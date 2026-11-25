pub mod romc;
pub mod u8;
pub mod wad;

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
