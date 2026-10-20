//! CRC-8/AUTOSAR, CRC-16/IBM-3740 and the transmission CRC rule (not part of the bitis wire
//! format).
//!
//! - CRC-8/AUTOSAR: polynomial `0x2F`, init `0xFF`, xorout `0xFF`, no reflection; check
//!   value `crc8_autosar(b"123456789") == 0xDF`, `crc8_autosar(b"") == 0x00`.
//! - CRC-16/IBM-3740 (also called CRC-16/CCITT-FALSE): polynomial `0x1021`, init `0xFFFF`,
//!   xorout `0x0000`, no reflection; check value `crc16_ibm3740(b"123456789") == 0x29B1`,
//!   `crc16_ibm3740(b"") == 0xFFFF`.
//! - Transmission CRC (`carachangprotocol/FRAMING.md`, section 2): a frame `F` of
//!   `n = F.len()` bytes (header + payload as on the wire, without the CRC) gets a trailer of
//!   1 byte CRC-8/AUTOSAR for `n <= 14` and of 2 bytes CRC-16/IBM-3740, little-endian (low
//!   byte first, e.g. `0x29B1` goes on the wire as `B1 29`), for `n >= 15`. A received frame of `L` bytes (trailer included) therefore uses
//!   CRC-8 for `L <= 15` and CRC-16 for `L >= 17`; `L = 16` (and `L = 0`) is invalid. The
//!   rule itself allows `n = 0` (a frame consisting only of the CRC-8 trailer `0x00`); real
//!   frames always have a header, i.e. `n >= 1`. The receiver recomputes the CRC over
//!   header + payload and compares it with the trailer (with the little-endian trailer the
//!   "CRC over frame + trailer = 0" residue check does not apply).
//!
//! On the ESP32 the ROM gives CRC-16/IBM-3740 as `(uint16_t)~esp_rom_crc16_be(0, buf, len)`
//! (see the "CRC helpers" section of the README); there is no ROM function for
//! CRC-8/AUTOSAR (the ROM CRC-8 uses polynomial `0x07`).
//!
//! Implementation: byte-wise lookup with 256-entry tables (256 bytes and 512 bytes) computed
//! at compile time by a `const fn`, like the CRC-32 helper (the Rust runtime runs on hosts).

/// Polynomial of CRC-8/AUTOSAR.
const POLY8: u8 = 0x2F;
/// Polynomial of CRC-16/IBM-3740.
const POLY16: u16 = 0x1021;
/// Largest `n` (bytes without the CRC) that is protected by CRC-8/AUTOSAR.
const CRC8_MAX_N: usize = 14;

/// `TABLE8[i]` = CRC-8 register after shifting in the byte `i` (register 0).
const TABLE8: [u8; 256] = make_table8();
/// `TABLE16[i]` = CRC-16 register after shifting in the byte `i` (register 0).
const TABLE16: [u16; 256] = make_table16();

const fn make_table8() -> [u8; 256] {
    let mut table = [0u8; 256];
    let mut i = 0;
    while i < 256 {
        let mut c = i as u8;
        let mut k = 0;
        while k < 8 {
            c = if c & 0x80 != 0 { (c << 1) ^ POLY8 } else { c << 1 };
            k += 1;
        }
        table[i] = c;
        i += 1;
    }
    table
}

const fn make_table16() -> [u16; 256] {
    let mut table = [0u16; 256];
    let mut i = 0;
    while i < 256 {
        let mut c = (i as u16) << 8;
        let mut k = 0;
        while k < 8 {
            c = if c & 0x8000 != 0 { (c << 1) ^ POLY16 } else { c << 1 };
            k += 1;
        }
        table[i] = c;
        i += 1;
    }
    table
}

// *****************************************************************
// CRC-8/AUTOSAR

/// CRC-8/AUTOSAR of `data` (poly `0x2F`, init `0xFF`, xorout `0xFF`, not reflected).
///
/// ```
/// assert_eq!(bitis_lib::crc8_autosar(b"123456789"), 0xDF);
/// assert_eq!(bitis_lib::crc8_autosar(b""), 0x00);
/// ```
pub fn crc8_autosar(data: &[u8]) -> u8 {
    let mut c = Crc8Autosar::new();
    c.update(data);
    c.finish()
}

/// Incremental CRC-8/AUTOSAR: feed the data in arbitrary pieces with
/// [`Crc8Autosar::update`]; [`Crc8Autosar::finish`] gives the same value as [`crc8_autosar`]
/// over the concatenation.
///
/// ```
/// let mut c = bitis_lib::Crc8Autosar::new();
/// c.update(b"1234");
/// c.update(b"56789");
/// assert_eq!(c.finish(), bitis_lib::crc8_autosar(b"123456789"));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Crc8Autosar {
    /// CRC register (before the final xorout).
    state: u8,
}

impl Default for Crc8Autosar {
    fn default() -> Self { Self::new() }
}

impl Crc8Autosar {
    /// A new CRC over the empty input.
    pub const fn new() -> Self { Crc8Autosar { state: 0xFF } }

    /// Appends `data` to the input; can be called any number of times.
    pub fn update(&mut self, data: &[u8]) {
        let mut c = self.state;
        for &b in data {
            c = TABLE8[(c ^ b) as usize];
        }
        self.state = c;
    }

    /// The CRC of all data passed to [`update`](Self::update) since creation or the last
    /// [`reset`](Self::reset). Does not change the state (more data can be appended).
    pub const fn finish(&self) -> u8 { self.state ^ 0xFF }

    /// Starts over with the empty input.
    pub fn reset(&mut self) { self.state = 0xFF; }
}

// *****************************************************************
// CRC-16/IBM-3740

/// CRC-16/IBM-3740 (CRC-16/CCITT-FALSE) of `data` (poly `0x1021`, init `0xFFFF`, xorout
/// `0x0000`, not reflected).
///
/// ```
/// assert_eq!(bitis_lib::crc16_ibm3740(b"123456789"), 0x29B1);
/// assert_eq!(bitis_lib::crc16_ibm3740(b""), 0xFFFF);
/// ```
pub fn crc16_ibm3740(data: &[u8]) -> u16 {
    let mut c = Crc16Ibm3740::new();
    c.update(data);
    c.finish()
}

/// Incremental CRC-16/IBM-3740: feed the data in arbitrary pieces with
/// [`Crc16Ibm3740::update`]; [`Crc16Ibm3740::finish`] gives the same value as
/// [`crc16_ibm3740`] over the concatenation.
///
/// ```
/// let mut c = bitis_lib::Crc16Ibm3740::new();
/// c.update(b"1234");
/// c.update(b"56789");
/// assert_eq!(c.finish(), bitis_lib::crc16_ibm3740(b"123456789"));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Crc16Ibm3740 {
    /// CRC register (xorout is 0).
    state: u16,
}

impl Default for Crc16Ibm3740 {
    fn default() -> Self { Self::new() }
}

impl Crc16Ibm3740 {
    /// A new CRC over the empty input.
    pub const fn new() -> Self { Crc16Ibm3740 { state: 0xFFFF } }

    /// Appends `data` to the input; can be called any number of times.
    pub fn update(&mut self, data: &[u8]) {
        let mut c = self.state;
        for &b in data {
            c = TABLE16[((c >> 8) as u8 ^ b) as usize] ^ (c << 8);
        }
        self.state = c;
    }

    /// The CRC of all data passed to [`update`](Self::update) since creation or the last
    /// [`reset`](Self::reset). Does not change the state (more data can be appended).
    pub const fn finish(&self) -> u16 { self.state }

    /// Starts over with the empty input.
    pub fn reset(&mut self) { self.state = 0xFFFF; }
}

// *****************************************************************
// transmission CRC rule

/// Length of the transmission CRC trailer for a frame of `n` bytes (header + payload, the
/// CRC not counted): 1 (CRC-8/AUTOSAR) for `n <= 14`, 2 (CRC-16/IBM-3740) for `n >= 15`.
///
/// ```
/// assert_eq!(bitis_lib::transmission_crc_len(14), 1);
/// assert_eq!(bitis_lib::transmission_crc_len(15), 2);
/// ```
pub const fn transmission_crc_len(n: usize) -> usize {
    if n <= CRC8_MAX_N { 1 } else { 2 }
}

/// Transmission CRC of `frame` (header + payload, without the CRC): CRC-8/AUTOSAR (as
/// `u16`) for `frame.len() <= 14`, CRC-16/IBM-3740 otherwise. The trailer is the low byte
/// (1-byte trailer) or the value little-endian (2-byte trailer, `to_le_bytes()`), see
/// [`transmission_crc_len`].
pub fn transmission_crc(frame: &[u8]) -> u16 {
    if frame.len() <= CRC8_MAX_N {
        crc8_autosar(frame) as u16
    } else {
        crc16_ibm3740(frame)
    }
}

/// Appends the transmission CRC trailer (1 or 2 bytes, see [`transmission_crc_len`]) to
/// `frame` (header + payload, without the CRC).
///
/// ```
/// let mut frame = vec![0x02, 0x01]; // header (size 1) + payload
/// bitis_lib::append_transmission_crc(&mut frame);
/// assert_eq!(frame, [0x02, 0x01, 0x95]);
/// assert!(bitis_lib::verify_transmission_crc(&frame));
/// ```
pub fn append_transmission_crc(frame: &mut Vec<u8>) {
    let crc = transmission_crc(frame);
    if frame.len() <= CRC8_MAX_N {
        frame.push(crc as u8);
    } else {
        frame.extend_from_slice(&crc.to_le_bytes());
    }
}

/// `frame` (header + payload) followed by its transmission CRC trailer.
pub fn with_transmission_crc(frame: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(frame.len() + 2);
    out.extend_from_slice(frame);
    append_transmission_crc(&mut out);
    out
}

/// Checks the transmission CRC of a received frame (trailer included) and returns the frame
/// without the trailer, or `None` if the length is invalid (0 or 16) or the CRC does not
/// match.
///
/// ```
/// assert_eq!(bitis_lib::strip_transmission_crc(&[0x02, 0x01, 0x95]), Some(&[0x02, 0x01][..]));
/// assert_eq!(bitis_lib::strip_transmission_crc(&[0x02, 0x01, 0x96]), None);
/// assert_eq!(bitis_lib::strip_transmission_crc(&[0u8; 16]), None);
/// ```
pub fn strip_transmission_crc(data: &[u8]) -> Option<&[u8]> {
    let l = data.len();
    if l == 0 || l == CRC8_MAX_N + 2 {
        return None;
    }
    if l <= CRC8_MAX_N + 1 {
        let (frame, trailer) = data.split_at(l - 1);
        (crc8_autosar(frame) == trailer[0]).then_some(frame)
    } else {
        let (frame, trailer) = data.split_at(l - 2);
        (crc16_ibm3740(frame).to_le_bytes() == trailer).then_some(frame)
    }
}

/// `true` if `data` (frame with trailer) has a valid length and a matching transmission
/// CRC, see [`strip_transmission_crc`].
pub fn verify_transmission_crc(data: &[u8]) -> bool {
    strip_transmission_crc(data).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn crc8_bitwise(data: &[u8]) -> u8 {
        let mut c = 0xFFu8;
        for &b in data {
            c ^= b;
            for _ in 0..8 {
                c = if c & 0x80 != 0 { (c << 1) ^ 0x2F } else { c << 1 };
            }
        }
        c ^ 0xFF
    }

    fn crc16_bitwise(data: &[u8]) -> u16 {
        let mut c = 0xFFFFu16;
        for &b in data {
            c ^= (b as u16) << 8;
            for _ in 0..8 {
                c = if c & 0x8000 != 0 { (c << 1) ^ 0x1021 } else { c << 1 };
            }
        }
        c
    }

    #[test]
    fn check_values() {
        assert_eq!(crc8_autosar(b"123456789"), 0xDF);
        assert_eq!(crc8_autosar(b""), 0x00);
        assert_eq!(Crc8Autosar::new().finish(), 0);
        assert_eq!(Crc8Autosar::default(), Crc8Autosar::new());
        assert_eq!(crc16_ibm3740(b"123456789"), 0x29B1);
        assert_eq!(crc16_ibm3740(b""), 0xFFFF);
        assert_eq!(Crc16Ibm3740::new().finish(), 0xFFFF);
        assert_eq!(Crc16Ibm3740::default(), Crc16Ibm3740::new());
    }

    #[test]
    fn tables() {
        assert_eq!(TABLE8[0], 0);
        assert_eq!(TABLE8[1], 0x2F);
        assert_eq!(TABLE16[0], 0);
        assert_eq!(TABLE16[1], 0x1021);
        assert_eq!(TABLE16[255], 0x1EF0);
    }

    #[test]
    fn matches_bitwise_reference() {
        let data: Vec<u8> = (0..1024u32).map(|i| (i.wrapping_mul(2_654_435_761) >> 13) as u8).collect();
        for n in [0, 1, 2, 3, 7, 8, 14, 15, 255, 256, 1024] {
            assert_eq!(crc8_autosar(&data[..n]), crc8_bitwise(&data[..n]), "n = {n}");
            assert_eq!(crc16_ibm3740(&data[..n]), crc16_bitwise(&data[..n]), "n = {n}");
        }
    }

    #[test]
    fn incremental_and_reset() {
        let data = b"The quick brown fox jumps over the lazy dog";
        let (e8, e16) = (crc8_autosar(data), crc16_ibm3740(data));
        for i in 0..=data.len() {
            for j in i..=data.len() {
                let mut c8 = Crc8Autosar::new();
                let mut c16 = Crc16Ibm3740::new();
                c8.update(&data[..i]);
                c16.update(&data[..i]);
                c8.update(&data[i..j]);
                c16.update(&data[i..j]);
                assert_eq!(c8.finish(), crc8_autosar(&data[..j]));
                assert_eq!(c16.finish(), crc16_ibm3740(&data[..j]));
                c8.update(&data[j..]);
                c16.update(&data[j..]);
                assert_eq!((c8.finish(), c16.finish()), (e8, e16));
            }
        }
        let mut c8 = Crc8Autosar::new();
        let mut c16 = Crc16Ibm3740::new();
        c8.update(b"garbage");
        c16.update(b"garbage");
        c8.reset();
        c16.reset();
        c8.update(b"123456789");
        c16.update(b"123456789");
        assert_eq!((c8.finish(), c16.finish()), (0xDF, 0x29B1));
    }

    #[test]
    fn transmission_boundaries() {
        assert_eq!(transmission_crc_len(0), 1);
        assert_eq!(transmission_crc_len(14), 1);
        assert_eq!(transmission_crc_len(15), 2);
        assert_eq!(transmission_crc_len(2047 + 2), 2);
        for n in 0..40usize {
            let frame: Vec<u8> = (0..n).map(|i| (i * 37 + 1) as u8).collect();
            let full = with_transmission_crc(&frame);
            assert_eq!(full.len(), n + transmission_crc_len(n));
            assert_ne!(full.len(), 16, "L = 16 never occurs");
            assert_eq!(strip_transmission_crc(&full), Some(&frame[..]));
            assert!(verify_transmission_crc(&full));
            if n >= 15 {
                assert_eq!(full[n..], crc16_ibm3740(&frame).to_le_bytes(), "little-endian, n = {n}");
            }
            for bit in 0..full.len() * 8 {
                let mut bad = full.clone();
                bad[bit / 8] ^= 1 << (bit % 8);
                assert!(!verify_transmission_crc(&bad), "n = {n}, bit {bit}");
            }
        }
        assert!(!verify_transmission_crc(&[]));
        assert_eq!(strip_transmission_crc(&[0x00]), Some(&[][..]));
        // L = 16 is invalid even with a CRC-16 trailer that would match its 14 bytes
        let mut l16: Vec<u8> = (0..14u8).collect();
        let c = crc16_ibm3740(&l16);
        l16.extend_from_slice(&c.to_le_bytes());
        assert_eq!(l16.len(), 16);
        assert!(!verify_transmission_crc(&l16));
    }
}
