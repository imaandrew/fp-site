use serde::Deserialize;
use thiserror::Error;
use tsify::{Ts, Tsify};
use wasm_bindgen::prelude::*;
use wii::{romc::Romc, u8::U8Error, wad::WadError};

use crate::{n64::VCDiffDecoderError, wii::{u8::U8Archive, wad::Wad}, wiiu::WiiUError};
extern crate console_error_panic_hook;

#[derive(Error, Debug)]
pub enum PatchError {
    #[error("invalid settings: {0}")]
    Settings(#[from] serde_wasm_bindgen::Error),
    #[error("n64: {0}")]
    N64(#[from] VCDiffDecoderError),
    #[error("wad: {0}")]
    Wad(#[from] WadError),
    #[error("u8 archive: {0}")]
    U8(#[from] U8Error),
    #[error("wii u: {0}")]
    WiiU(#[from] WiiUError),
}

pub mod n64;
pub mod wii;
pub mod wiiu;

fn n64_decode(rom: &[u8], patch: &[u8]) -> Result<Vec<u8>, PatchError> {
    let mut x = n64::decode::VCDiffDecoder::new(patch, rom);
    Ok(x.decode()?)
}

#[derive(Tsify, Deserialize)]
pub struct N64PatcherSettings {
    #[serde(with = "serde_bytes")]
    #[tsify(type = "Uint8Array")]
    pub input: Vec<u8>,
    #[serde(with = "serde_bytes")]
    #[tsify(type = "Uint8Array")]
    pub rom_patch: Vec<u8>,
}

#[derive(Tsify, Deserialize)]
pub struct WiiPatcherSettings {
    #[serde(with = "serde_bytes")]
    #[tsify(type = "Uint8Array")]
    pub input: Vec<u8>,
    #[serde(with = "serde_bytes")]
    #[tsify(type = "Uint8Array")]
    pub rom_patch: Vec<u8>,
    #[serde(with = "serde_bytes")]
    #[tsify(type = "Uint8Array")]
    pub gzi_patch: Vec<u8>,
    pub channel_id: String,
    pub title: String,
    pub dol_patch: DolPatch,
}

#[derive(Tsify, Deserialize)]
pub struct DolPatch {
    pub dol_num: usize,
    pub load_addr: u32,
    #[serde(with = "serde_bytes")]
    #[tsify(type = "Uint8Array")]
    pub data: Vec<u8>,
}

#[derive(Tsify, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResultType {
    Zip,
    Tar,
}

#[derive(Tsify, Deserialize)]
pub struct WiiUPatcherSettings {
    #[serde(with = "serde_bytes")]
    #[tsify(type = "Uint8Array")]
    input: Vec<u8>,
    #[serde(with = "serde_bytes")]
    #[tsify(type = "Uint8Array")]
    rom_patch: Vec<u8>,
    enable_dark_filter: bool,
    enable_widescreen: bool,
    name: String,
    #[serde(with = "serde_bytes")]
    #[tsify(type = "Uint8Array")]
    config: Vec<u8>,
    result_type: ResultType,
    #[serde(with = "serde_bytes")]
    #[tsify(type = "Uint8Array")]
    frame_layout: Vec<u8>,
}

#[derive(Tsify, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PatcherSettings {
    N64(N64PatcherSettings),
    Wii(WiiPatcherSettings),
    WiiU(WiiUPatcherSettings)
}

#[wasm_bindgen]
pub fn patch(s: Ts<PatcherSettings>) -> Result<Vec<u8>, JsError> {
    console_error_panic_hook::set_once();
    match s.to_rust()? {
        PatcherSettings::N64(s) => Ok(patch_n64(&s)?),
        PatcherSettings::Wii(s) => Ok(patch_wii(&s)?),
        PatcherSettings::WiiU(mut s) => Ok(patch_wiiu(&mut s)?),
    }
}

fn patch_n64(s: &N64PatcherSettings) -> Result<Vec<u8>, PatchError> {
    n64_decode(&s.input, &s.rom_patch)
}

fn patch_wii(s: &WiiPatcherSettings) -> Result<Vec<u8>, PatchError> {
    let mut wad = Wad::decode(&s.input)?;
    let mut arc = U8Archive::new(wad.get_content(5).unwrap())?;
    let mut n = arc.open("./romc")?;
    let rom = Romc::decode(arc.read(&n)?);
    let patched_rom = n64_decode(&rom, &s.rom_patch)?;
    arc.replace_file(&mut n, &Romc::encode(&patched_rom))?;
    wad.set_title_id(&s.channel_id.as_bytes()[..4])?;
    wad.set_channel_title(&s.title)?;
    wad.set_region(3);
    wad.apply_gzi(&String::from_utf8(s.gzi_patch.clone()).unwrap())?;
    wad.apply_dol_patch(&s.dol_patch)?;
    Ok(wad.encode())
}

fn patch_wiiu(s: &mut WiiUPatcherSettings) -> Result<Vec<u8>, PatchError> {
    Ok(wiiu::patch(s)?)
}
