//! CRC-32 helper (not part of the bitis wire format).
//!
//! CRC-32/ISO-HDLC: reflected polynomial `0xEDB88320`, init `0xFFFFFFFF`, xorout
//! `0xFFFFFFFF`, reflected input and output. This is the CRC of zlib / Python
//! `zlib.crc32`, Ethernet, PNG and gzip; check value `crc32(b"123456789") == 0xCBF43926`,
//! `crc32(b"") == 0`.
//!
//! On the ESP32 the ROM function gives the same value with
//! `esp_rom_crc32_le(0, buf, len)` (incrementally: `crc = esp_rom_crc32_le(crc, buf, len)`
//! starting with `crc = 0`), see the "CRC-32 helper" section of the README.
//!
//! Implementation: byte-wise lookup with a 256-entry table (1 KiB) computed at compile
//! time by a `const fn`. The Rust runtime runs on hosts (tools, Python bindings), where
//! 1 KiB of read-only data is negligible and the byte-wise table is about twice as fast as
//! a 16-entry nibble table.

/// The 256-entry lookup table, `TABLE[i]` = CRC register after shifting in the byte `i`.
const TABLE: [u32; 256] = make_table();

const fn make_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        let mut c = i as u32;
        let mut k = 0;
        while k < 8 {
            c = if c & 1 != 0 { (c >> 1) ^ 0xEDB8_8320 } else { c >> 1 };
            k += 1;
        }
        table[i] = c;
        i += 1;
    }
    table
}

/// CRC-32/ISO-HDLC of `data` (same value as zlib / Python `zlib.crc32`).
///
/// ```
/// assert_eq!(bitis_lib::crc32(b"123456789"), 0xCBF4_3926);
/// assert_eq!(bitis_lib::crc32(b""), 0);
/// ```
pub fn crc32(data: &[u8]) -> u32 {
    let mut c = Crc32::new();
    c.update(data);
    c.finish()
}

/// Incremental CRC-32/ISO-HDLC: feed the data in arbitrary pieces with [`Crc32::update`];
/// [`Crc32::finish`] gives the same value as [`crc32`] over the concatenation.
///
/// ```
/// let mut c = bitis_lib::Crc32::new();
/// c.update(b"1234");
/// c.update(b"56789");
/// assert_eq!(c.finish(), bitis_lib::crc32(b"123456789"));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Crc32 {
    /// CRC register (before the final xorout).
    state: u32,
}

impl Default for Crc32 {
    fn default() -> Self { Self::new() }
}

impl Crc32 {
    /// A new CRC over the empty input.
    pub const fn new() -> Self { Crc32 { state: 0xFFFF_FFFF } }

    /// Appends `data` to the input; can be called any number of times.
    pub fn update(&mut self, data: &[u8]) {
        let mut c = self.state;
        for &b in data {
            c = TABLE[((c ^ b as u32) & 0xFF) as usize] ^ (c >> 8);
        }
        self.state = c;
    }

    /// The CRC of all data passed to [`update`](Self::update) since creation or the last
    /// [`reset`](Self::reset). Does not change the state (more data can be appended).
    pub const fn finish(&self) -> u32 { self.state ^ 0xFFFF_FFFF }

    /// Starts over with the empty input.
    pub fn reset(&mut self) { self.state = 0xFFFF_FFFF; }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Bit-wise reference implementation (no table).
    fn crc32_bitwise(data: &[u8]) -> u32 {
        let mut c = 0xFFFF_FFFFu32;
        for &b in data {
            c ^= b as u32;
            for _ in 0..8 {
                c = if c & 1 != 0 { (c >> 1) ^ 0xEDB8_8320 } else { c >> 1 };
            }
        }
        !c
    }

    #[test]
    fn check_value() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
        assert_eq!(crc32(b""), 0);
        assert_eq!(Crc32::new().finish(), 0);
        assert_eq!(Crc32::default(), Crc32::new());
    }

    #[test]
    fn table() {
        assert_eq!(TABLE[0], 0);
        assert_eq!(TABLE[1], 0x7707_3096);
        assert_eq!(TABLE[255], 0x2D02_EF8D);
    }

    #[test]
    fn matches_bitwise_reference() {
        let data: Vec<u8> = (0..1024u32).map(|i| (i.wrapping_mul(2_654_435_761) >> 13) as u8).collect();
        for n in [0, 1, 2, 3, 7, 8, 255, 256, 1024] {
            assert_eq!(crc32(&data[..n]), crc32_bitwise(&data[..n]), "n = {n}");
        }
    }

    #[test]
    fn incremental_and_reset() {
        let data = b"The quick brown fox jumps over the lazy dog";
        let expected = crc32(data);
        for i in 0..=data.len() {
            for j in i..=data.len() {
                let mut c = Crc32::new();
                c.update(&data[..i]);
                c.update(&data[i..j]);
                assert_eq!(c.finish(), crc32(&data[..j]));
                c.update(&data[j..]);
                assert_eq!(c.finish(), expected);
            }
        }
        let mut c = Crc32::new();
        c.update(b"garbage");
        c.reset();
        c.update(b"123456789");
        assert_eq!(c.finish(), 0xCBF4_3926);
    }
}
