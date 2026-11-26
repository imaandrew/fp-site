use crate::n64::VCDiffDecoderError;
use crate::{Region, WiiUPatcherSettings, n64_decode};
use aho_corasick::{AhoCorasick, BuildError, MatchKind};
use std::collections::HashSet;
use std::io::{self, Cursor, Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::str;
use tar::{Archive, Builder, Header};
use thiserror::Error;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, result::ZipError, write::ZipWriter};

#[derive(Error, Debug)]
pub enum WiiUError {
    #[error("unknown archive format")]
    UnknownArchive,
    #[error(transparent)]
    Zip(#[from] ZipError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    InvalidStr(#[from] std::str::Utf8Error),
    #[error(transparent)]
    AhoCorasick(#[from] BuildError),
    #[error(transparent)]
    Patcher(#[from] VCDiffDecoderError),
    #[error("Invalid path: {0}")]
    InvalidPath(PathBuf),
}

const ZIP_MAGIC: [u8; 4] = [0x50, 0x4b, 0x03, 0x04];
const TAR_MAGIC: [u8; 4] = [0x75, 0x73, 0x74, 0x61];
const TAR_OFFSET: u64 = 0x101;

#[derive(PartialEq)]
enum ArchiveFormat {
    Unknown,
    Zip,
    Tar,
}

pub fn patch(s: &mut WiiUPatcherSettings) -> Result<Vec<u8>, WiiUError> {
    let replace_list: HashSet<&'static str> = HashSet::from([
        "content/rom/UNMQE0.785",
        "content/rom/Unmqj0.716",
        "code/app.xml",
        "meta/meta.xml",
        "content/FrameLayout.arc",
    ]);

    let title_id = format!(
        "000500007080{:02X}{:02X}",
        if s.region == Region::Us { b'U' } else { b'J' },
        (s.enable_dark_filter as u8) << 4 | (s.enable_widescreen as u8) << 5
    );
    let group_id = format!("0000{}", &title_id[10..14]);
    let patterns = &[
        "NACE",
        "NAEJ",
        "Paper Mario",
        "マリオストーリー",
        "0005000010199700",
        "0005000010199600",
        "00001997",
        "00001996",
    ];
    let replacements = &[
        "FPUS", "FPJP", "fp-us", "fp-jp", &title_id, &title_id, &group_id, &group_id,
    ];

    let ac = AhoCorasick::builder()
        .match_kind(MatchKind::LeftmostLongest)
        .build(patterns)?;

    let replace_file = |p: &str, d: Vec<u8>| -> Result<Vec<u8>, WiiUError> {
        match p {
            "content/rom/UNMQE0.785" | "content/rom/Unmqj0.716" => {
                Ok(n64_decode(&d, &s.rom_patch)?)
            }
            "content/config/UNMQE0.z64.ini" | "content/config/UNMQJ0.z64.ini" => {
                Ok(std::mem::take(&mut s.config))
            }
            "code/app.xml" | "meta/meta.xml" => Ok(ac.replace_all_bytes(&d, replacements)),
            "content/FrameLayout.arc" => Ok(std::mem::take(&mut s.frame_layout)),
            _ => unreachable!(),
        }
    };

    let mut c = Cursor::new(&s.input);
    let mut sig = vec![0; 4];
    c.read_exact(&mut sig)?;
    let mut fmt = ArchiveFormat::Unknown;
    if sig == ZIP_MAGIC {
        fmt = ArchiveFormat::Zip;
    } else {
        c.seek(std::io::SeekFrom::Start(TAR_OFFSET))?;
        c.read_exact(&mut sig)?;
        if sig == TAR_MAGIC {
            fmt = ArchiveFormat::Tar;
        }
    }

    if fmt == ArchiveFormat::Unknown {
        return Err(WiiUError::UnknownArchive);
    }
    c.seek(std::io::SeekFrom::Start(0))?;

    let output = vec![];
    Ok(match fmt {
        ArchiveFormat::Zip => edit_zip(c, Cursor::new(output), &replace_list, replace_file)?,
        ArchiveFormat::Tar => edit_tar(c, Cursor::new(output), &replace_list, replace_file)?,
        ArchiveFormat::Unknown => unreachable!(),
    }
    .into_inner())
}

fn edit_zip<R: Read + io::Seek, W: Write + io::Seek>(
    data: R,
    out: W,
    replace: &HashSet<&'static str>,
    mut edit: impl FnMut(&str, Vec<u8>) -> Result<Vec<u8>, WiiUError>,
) -> Result<W, WiiUError> {
    let mut input = ZipArchive::new(data)?;
    let mut output = ZipWriter::new(out);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

    for i in 0..input.len() {
        let mut f = input.by_index(i)?;
        if !replace.contains(&*f.name()?) {
            output.raw_copy_file(f)?;
            continue;
        }

        let mut contents = Vec::with_capacity(f.size() as usize);
        f.read_to_end(&mut contents)?;
        output.start_file(&f.name()?, options)?;
        output.write_all(&edit(&f.name()?, contents)?)?;
    }

    Ok(output.finish()?)
}

fn edit_tar<R: Read, W: Write>(
    data: R,
    out: W,
    replace: &HashSet<&'static str>,
    mut edit: impl FnMut(&str, Vec<u8>) -> Result<Vec<u8>, WiiUError>,
) -> Result<W, WiiUError> {
    let mut input = Archive::new(data);
    let mut output = Builder::new(out);

    for e in input.entries()? {
        let mut e = e?;
        let path = e
            .path()?
            .into_owned()
            .into_string()
            .map_err(WiiUError::InvalidPath)?;
        let mut header = Header::new_gnu();
        if !replace.contains(path.as_str()) {
            header.set_size(e.size());
            header.set_cksum();
            output.append_data(&mut header, Path::new(&path), e)?;
            continue;
        }

        let mut contents = Vec::with_capacity(e.size() as usize);
        e.read_to_end(&mut contents)?;
        let p = edit(&path, contents)?;
        header.set_size(p.len() as u64);
        header.set_cksum();
        output.append_data(&mut header, path, &*p)?;
    }

    Ok(output.into_inner()?)
}
