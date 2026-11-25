use std::num::ParseIntError;

use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, block_padding::NoPadding};
use sha1::{Digest, Sha1};
use thiserror::Error;

use crate::{
    DolPatch,
    reader::{Reader, ReaderError, Writer},
};

type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;
type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

const COMMON_KEY: [u8; 16] = [
    0xeb, 0xe4, 0x2a, 0x22, 0x5e, 0x85, 0x93, 0xe4, 0x48, 0xd9, 0xc5, 0x45, 0x73, 0x81, 0xaa, 0xf7,
];

const WAD_MAGIC: [u8; 8] = [0x00, 0x00, 0x00, 0x20, 0x49, 0x73, 0x00, 0x00];
const WAD_PADDING: usize = 0x40;

mod content {
    pub const TMD: usize = 100;
    pub const TIK: usize = 101;
    pub const CERT_CHAIN: usize = 102;
}

mod tik {
    pub const TITLE_KEY: usize = 0x1bf;
    pub const TITLE_KEY_SIZE: usize = 0x10;
    pub const TITLE_ID: usize = 0x1dc;
    pub const TITLE_ID_SIZE: usize = 0x8;
    pub const PADDING: usize = 0x1f2;
}

mod tmd {
    pub const NUM_CONTENTS: usize = 0x1de;
    pub const CONTENT_SIZE: usize = 0x08;
    pub const CONTENT_HASH: usize = 0x10;
    pub const CMD_START: usize = 0x1e4;
    pub const CMD_SIZE: usize = 0x24;
    pub const TITLE_ID: usize = 0x18c;
    pub const REGION: usize = 0x19c;
    pub const PADDING: usize = 0x1d4;
}

mod imet {
    pub const CHANNEL_TITLE: usize = 0x9c;
    pub const CHANNEL_TITLE_LEN: usize = 0x54;
    pub const CHANNEL_TITLE_COUNT: usize = 10;
    pub const HASH: usize = 0x630;
    pub const HDR_START: usize = 0x40;
}

mod dol {
    pub const NUM_SEGMENTS: usize = 7;
    pub const SECTION_LOAD_ADDR: usize = 0x48;
    pub const SECTION_SIZE: usize = 0x90;
    pub const DATA_OFFSET: usize = 0x1c;
}

fn read_be16(d: &[u8], i: usize) -> u16 {
    u16::from_be_bytes(d[i..][..2].try_into().expect("should have 2 elements"))
}

fn read_be32(d: &[u8], i: usize) -> u32 {
    u32::from_be_bytes(d[i..][..4].try_into().expect("should have 4 elements"))
}

fn read_be64(d: &[u8], i: usize) -> u64 {
    u64::from_be_bytes(d[i..][..8].try_into().expect("should have 8 elements"))
}

fn write_be32(d: &mut [u8], i: usize, v: u32) {
    d[i..][..4].copy_from_slice(&v.to_be_bytes());
}

fn align_num(num: usize, align: usize) -> usize {
    num.div_ceil(align) * align
}

fn aes_cbc_dec(key: [u8; 16], iv: [u8; 16], data: &[u8]) -> Option<Vec<u8>> {
    Aes128CbcDec::new(&key.into(), &iv.into())
        .decrypt_padded_vec_mut::<NoPadding>(data)
        .ok()
}

fn aes_cbc_enc(key: [u8; 16], iv: [u8; 16], data: &[u8]) -> Vec<u8> {
    Aes128CbcEnc::new(&key.into(), &iv.into()).encrypt_padded_vec_mut::<NoPadding>(data)
}

fn sha1(data: &[u8]) -> [u8; 20] {
    Sha1::digest(data).into()
}

#[derive(Error, Debug)]
pub enum WadError {
    #[error("unexpected wad header (expected: `{0:?}`, got: `{1:?}`)")]
    UnexpectedWadHeader(Vec<u8>, Vec<u8>),
    #[error("reader: {0}")]
    ReaderError(#[from] ReaderError),
    #[error("unable to decrypt {0}")]
    DecryptionError(String),
    #[error("content {0} hash mismatch. expected {1:#?}, got {2:#?}")]
    ContentHashMismatch(usize, Vec<u8>, Vec<u8>),
    #[error("invalid gzi patch line: {0}")]
    InvalidGzi(String),
    #[error("invalid gzi patch: content file not specified")]
    InvalidGziContent,
    #[error("invalid gzi patch: content file {0} not found")]
    InvalidGziContentIdx(u8),
    #[error("invalid gzi patch: index `{0}` out of bounds")]
    GziIndexBounds(usize),
    #[error("title id `{0}` too large")]
    InvalidTitleId(String),
    #[error("channel title `{0}` too large")]
    InvalidChannelTitle(String),
    #[error("dol `{0}` has no free text segments")]
    NoTextSegments(usize),
    #[error("invalid int: {0}")]
    IntError(#[from] ParseIntError),
    #[error("lz77 gzi option not supported")]
    UnsupportedLz77,
    #[error("wad content {0} doesn't exist")]
    MissingContent(usize),
    #[error("malformed content {0}")]
    MalformedContent(usize),
    #[error("malformed tmd")]
    MalformedTmd,
    #[error("malformed tik")]
    MalformedTik,
}

pub struct Wad {
    cert_chain: Vec<u8>,
    tik: Vec<u8>,
    tmd: Vec<u8>,
    title_key: [u8; 16],
    contents: Vec<Vec<u8>>,
    footer: Vec<u8>,
}

impl Wad {
    pub fn decode(data: &[u8]) -> Result<Self, WadError> {
        let mut r = Reader::new(data);
        let magic = r.take(8)?;
        if magic != WAD_MAGIC {
            return Err(WadError::UnexpectedWadHeader(
                WAD_MAGIC.to_vec(),
                magic.to_vec(),
            ));
        }

        let cert_len = r.u32()? as usize;
        r.skip(4);
        let tik_len = r.u32()? as usize;
        let tmd_len = r.u32()? as usize;
        let _content_len = r.u32()? as usize;
        let footer_size = r.u32()? as usize;

        r.align(WAD_PADDING);
        let cert_chain = r.take(cert_len)?.to_vec();
        r.align(WAD_PADDING);
        let tik = r.take(tik_len)?.to_vec();
        r.align(WAD_PADDING);
        let tmd = r.take(tmd_len)?.to_vec();
        r.align(WAD_PADDING);

        let title_key = Self::decrypt_title_key(&tik)?;
        let n = read_be16(&tmd, tmd::NUM_CONTENTS) as usize;

        if tmd.len() < tmd::CMD_SIZE + n * tmd::CMD_SIZE {
            return Err(WadError::MalformedTmd);
        }

        if tik.len() < tik::PADDING + size_of::<u16>() {
            return Err(WadError::MalformedTik);
        }

        let mut wad = Wad {
            cert_chain,
            tik,
            tmd,
            title_key,
            contents: vec![],
            footer: vec![],
        };

        for i in 0..n {
            let size = wad.content_size(i) as usize;

            let enc = r.take(align_num(size, 0x10))?;
            r.align(WAD_PADDING);
            let mut dec = aes_cbc_dec(wad.title_key, Self::content_iv(i), enc)
                .ok_or(WadError::DecryptionError(format!("content{}", i)))?;
            dec.truncate(size);

            let expected_hash = wad.content_hash(i);
            let actual_hash = sha1(&dec);
            if *expected_hash != actual_hash {
                return Err(WadError::ContentHashMismatch(
                    i,
                    expected_hash.into(),
                    actual_hash.into(),
                ));
            }

            wad.contents.push(dec);
        }

        r.align(WAD_PADDING);
        wad.footer = r.take(footer_size)?.into();

        Ok(wad)
    }

    pub fn get_content(&mut self, i: usize) -> Result<&mut Vec<u8>, WadError> {
        match i {
            content::TMD => Ok(&mut self.tmd),
            content::TIK => Ok(&mut self.tik),
            content::CERT_CHAIN => Ok(&mut self.cert_chain),
            _ => self.contents.get_mut(i).ok_or(WadError::MissingContent(i)),
        }
    }

    fn decrypt_title_key(ticket: &[u8]) -> Result<[u8; 16], WadError> {
        let mut title_key = ticket[tik::TITLE_KEY..tik::TITLE_KEY + tik::TITLE_KEY_SIZE].to_vec();
        let mut iv = vec![0; 16];
        iv[..tik::TITLE_ID_SIZE]
            .copy_from_slice(&ticket[tik::TITLE_ID..tik::TITLE_ID + tik::TITLE_ID_SIZE]);
        Aes128CbcDec::new(&COMMON_KEY.into(), iv.as_slice().into())
            .decrypt_padded_mut::<NoPadding>(&mut title_key)
            .map_err(|_| WadError::DecryptionError("title key".to_string()))?;
        Ok(title_key.try_into().expect("should have 16 elements"))
    }

    fn reencrypt_title_key(&mut self) {
        let mut iv = [0u8; tik::TITLE_KEY_SIZE];
        iv[..tik::TITLE_ID_SIZE].copy_from_slice(&self.tik[tik::TITLE_ID..][..tik::TITLE_ID_SIZE]);
        let enc = aes_cbc_enc(COMMON_KEY, iv, &self.title_key);
        self.tik[tik::TITLE_KEY..][..tik::TITLE_KEY_SIZE].copy_from_slice(&enc);
    }

    fn content_size(&self, i: usize) -> u64 {
        read_be64(
            &self.tmd,
            tmd::CMD_START + i * tmd::CMD_SIZE + tmd::CONTENT_SIZE,
        )
    }

    fn content_hash(&mut self, i: usize) -> &mut [u8] {
        let offset = tmd::CMD_START + i * tmd::CMD_SIZE + tmd::CONTENT_HASH;
        &mut self.tmd[offset..][..20]
    }

    fn content_iv(idx: usize) -> [u8; 16] {
        let i = (idx as u16).to_be_bytes();
        let mut iv = [0; 16];
        iv[..i.len()].copy_from_slice(&i);
        iv
    }

    fn fake_sign(data: &mut [u8], offset: usize) {
        data[0x04..0x104].fill(0);

        for i in 0..0xFFFFu16 {
            data[offset..][..size_of::<u16>()].copy_from_slice(&i.to_be_bytes());

            if sha1(&data[0x140..])[0] == 0 {
                break;
            }
        }
    }

    pub fn apply_gzi(&mut self, p: &[u8]) -> Result<(), WadError> {
        let mut cur_content: Option<u8> = None;

        let patch = String::from_utf8_lossy(p);

        for l in patch.lines() {
            if l.is_empty() || l.starts_with('#') {
                continue;
            }

            let [header, offset, data] = l.split_whitespace().collect::<Vec<_>>()[..] else {
                return Err(WadError::InvalidGzi(l.to_string()));
            };

            let cmd = header
                .get(0..2)
                .ok_or(WadError::InvalidGzi(l.to_string()))?;
            let size = header
                .get(2..4)
                .ok_or(WadError::InvalidGzi(l.to_string()))?;

            match cmd {
                "00" => {
                    cur_content.replace(u8::from_str_radix(data, 16)?);
                }
                // lz77 compression/decompression not implemented
                "01" | "02" => return Err(WadError::UnsupportedLz77),
                "03" => {
                    let size = size
                        .parse::<u8>()
                        .map_err(|_| WadError::InvalidGzi(l.to_string()))?
                        as usize;

                    let offset = u32::from_str_radix(offset, 16)
                        .map_err(|_| WadError::InvalidGzi(l.to_string()))?
                        as usize;
                    let data = match size {
                        1 => &u8::from_str_radix(data, 16)
                            .map_err(|_| WadError::InvalidGzi(l.to_string()))?
                            .to_be_bytes()[..],
                        2 => &u16::from_str_radix(data, 16)
                            .map_err(|_| WadError::InvalidGzi(l.to_string()))?
                            .to_be_bytes()[..],
                        4 => &u32::from_str_radix(data, 16)
                            .map_err(|_| WadError::InvalidGzi(l.to_string()))?
                            .to_be_bytes()[..],
                        _ => return Err(WadError::InvalidGzi(l.to_string())),
                    };

                    let idx = cur_content.ok_or(WadError::InvalidGziContent)?;
                    let content = self
                        .get_content(idx as usize)
                        .map_err(|_| WadError::InvalidGziContentIdx(idx))?;
                    if offset + size > content.len() {
                        return Err(WadError::GziIndexBounds(offset + size));
                    }

                    content[offset..offset + size].copy_from_slice(data);
                }
                _ => return Err(WadError::InvalidGzi(l.to_string())),
            }
        }

        Ok(())
    }

    pub fn apply_dol_patch(&mut self, patch: &DolPatch) -> Result<(), WadError> {
        let dol = self.get_content(patch.dol_num)?;
        let slot = (0..dol::NUM_SEGMENTS)
            .find(|&i| read_be32(dol, i * 4) == 0)
            .ok_or(WadError::NoTextSegments(patch.dol_num))?;

        let insert_at = read_be32(dol, dol::DATA_OFFSET) as usize;
        let size = align_num(patch.data.len(), 32);
        let mut section = patch.data.clone();
        section.resize(size, 0);
        dol.splice(insert_at..insert_at, section);

        for i in 0..11 {
            let off = dol::DATA_OFFSET + i * 4;
            let v = read_be32(dol, off);
            if v != 0 {
                write_be32(dol, off, v + size as u32);
            }
        }
        write_be32(dol, slot * 4, insert_at as u32);
        write_be32(dol, dol::SECTION_SIZE + slot * 4, size as u32);
        write_be32(dol, dol::SECTION_LOAD_ADDR + slot * 4, patch.load_addr);
        Ok(())
    }

    pub fn set_title_id(&mut self, title_id: [u8; 4]) -> Result<(), WadError> {
        self.tik[tik::TITLE_ID + 4..][..4].copy_from_slice(&title_id);
        self.tmd[tmd::TITLE_ID + 4..][..4].copy_from_slice(&title_id);
        Ok(())
    }

    pub fn set_channel_title(&mut self, title: &str) -> Result<(), WadError> {
        let units: Vec<u16> = title.encode_utf16().collect();
        if units.len() > imet::CHANNEL_TITLE_LEN / 2 {
            return Err(WadError::InvalidChannelTitle(title.to_string()));
        }

        let mut t = vec![0u8; imet::CHANNEL_TITLE_LEN];
        for (j, u) in units.iter().enumerate() {
            t[j * 2..][..2].copy_from_slice(&u.to_be_bytes());
        }

        let content0 = self.get_content(0)?;
        if content0.len() < imet::HASH + 16 {
            return Err(WadError::MalformedContent(0));
        }

        for i in 0..imet::CHANNEL_TITLE_COUNT {
            content0[imet::CHANNEL_TITLE + i * imet::CHANNEL_TITLE_LEN..]
                [..imet::CHANNEL_TITLE_LEN]
                .copy_from_slice(&t);
        }

        content0[imet::HASH..][..16].fill(0);
        let d = md5::compute(&content0[imet::HDR_START..imet::HASH + 16]);
        content0[imet::HASH..][..16].copy_from_slice(d.as_slice());

        Ok(())
    }

    pub fn set_region(&mut self, region: u16) {
        self.tmd[tmd::REGION..][..2].copy_from_slice(&region.to_be_bytes());
    }

    pub fn encode(&mut self) -> Vec<u8> {
        let mut w = Writer::new(
            self.cert_chain.len()
                + self.tik.len()
                + self.tmd.len()
                + self.contents.iter().map(|x| x.len()).sum::<usize>()
                + self.footer.len(),
        );

        self.reencrypt_title_key();
        Wad::fake_sign(&mut self.tik, tik::PADDING);

        w.write(&WAD_MAGIC);
        w.u32(self.cert_chain.len());
        w.u32(0);
        w.u32(self.tik.len());
        w.u32(self.tmd.len());

        let mut contents = vec![];

        for i in 0..read_be16(&self.tmd, tmd::NUM_CONTENTS) as usize {
            let size = self.contents[i].len();
            let hash = sha1(&self.contents[i]);
            let mut padded = self.contents[i].clone();
            padded.resize(align_num(size, 0x10), 0);
            let enc = aes_cbc_enc(self.title_key, Self::content_iv(i), &padded);
            self.content_hash(i).copy_from_slice(&hash);
            let off = tmd::CMD_START + i * tmd::CMD_SIZE + tmd::CONTENT_SIZE;
            self.tmd[off..][..8].copy_from_slice(&(size as u64).to_be_bytes());
            contents.push(enc);
        }

        w.u32(
            contents
                .iter()
                .map(|c| align_num(c.len(), WAD_PADDING))
                .sum(),
        );
        w.u32(self.footer.len());
        w.align(WAD_PADDING);

        Wad::fake_sign(&mut self.tmd, tmd::PADDING);

        w.write(&self.cert_chain);
        w.align(WAD_PADDING);
        w.write(&self.tik);
        w.align(WAD_PADDING);
        w.write(&self.tmd);
        w.align(WAD_PADDING);
        contents.iter().for_each(|c| {
            w.write(c);
            w.align(WAD_PADDING);
        });
        w.align(WAD_PADDING);
        w.write(&self.footer);
        w.align(WAD_PADDING);
        w.get()
    }
}
