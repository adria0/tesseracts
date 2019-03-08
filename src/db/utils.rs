/// get u64 as litte endian
pub fn u64_to_le(v: u64) -> [u8; 8] {
    [
        ((v >> 56) & 0xff) as u8,
        ((v >> 48) & 0xff) as u8,
        ((v >> 40) & 0xff) as u8,
        ((v >> 32) & 0xff) as u8,
        ((v >> 24) & 0xff) as u8,
        ((v >> 16) & 0xff) as u8,
        ((v >> 8) & 0xff) as u8,
        ((v     ) & 0xff) as u8,
    ]
}

/// get u64 from litte endian
pub fn le_to_u64(v: [u8; 8]) -> u64 {
    u64::from(v[7])
    + (u64::from(v[6]) << 8 )
    + (u64::from(v[5]) << 16)
    + (u64::from(v[4]) << 24)
    + (u64::from(v[3]) << 32)
    + (u64::from(v[2]) << 40)
    + (u64::from(v[1]) << 48)
    + (u64::from(v[0]) << 56)
}

/// get u64 from litte endian slice
pub fn u64_from_slice(v: &[u8]) -> u64 {
    let mut le = [0; 8];
    le[..].copy_from_slice(v);
    le_to_u64(le)
}

