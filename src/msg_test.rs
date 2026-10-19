// Regression and robustness tests for the runtime (decoder hardening, range checks,
// wire format examples from docs/WIRE_FORMAT.md).

#[cfg(test)]
mod helpers {
    use crate::*;

    /// Deterministic PRNG (xorshift64*), no external dependency.
    pub struct Rng(u64);
    impl Rng {
        pub fn new(seed: u64) -> Self { Rng(seed.max(1)) }
        pub fn next_u64(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x >> 12; x ^= x << 25; x ^= x >> 27;
            self.0 = x;
            x.wrapping_mul(0x2545F4914F6CDD1D)
        }
        pub fn below(&mut self, n: u64) -> u64 { if n == 0 { 0 } else { self.next_u64() % n } }
        pub fn bool(&mut self) -> bool { self.next_u64() & 1 == 1 }
        pub fn bytes(&mut self, n: usize) -> Vec<u8> { (0..n).map(|_| self.next_u64() as u8).collect() }
        /// signed value with |v| <= max_mag
        pub fn signed(&mut self, max_mag: u64) -> i64 {
            let m = self.below(max_mag.saturating_add(1)) as i64;
            if self.bool() { -m } else { m }
        }
    }

    /// Stream bits in stream order ('0'/'1'), first `bits` bits.
    pub fn bit_string(data: &[u8], bits: u64) -> String {
        (0..bits).map(|i| if (data[(i / 8) as usize] >> (i % 8)) & 1 == 1 { '1' } else { '0' }).collect()
    }
    /// Bytes from a stream bit string (spaces ignored).
    pub fn from_bit_string(s: &str) -> Vec<u8> {
        let bits: Vec<bool> = s.chars().filter(|c| !c.is_whitespace()).map(|c| c == '1').collect();
        let mut d = vec![0u8; bits.len().div_ceil(8)];
        for (i, b) in bits.iter().enumerate() { if *b { d[i / 8] |= 1 << (i % 8); } }
        d
    }
    pub fn ser_bits<T: BiserdiTrait>(v: &T) -> String {
        let (d, s) = serialize(v);
        bit_string(&d, s.total_bits)
    }

    /// Decodes `data` with `T::bit_deserialize`, checks that the reported size equals the
    /// consumed bits and that a decoded value re-encodes without error and idempotently.
    /// `roundtrip = false` for element types with `min_bits = 0` inside dynamic arrays (not
    /// allowed by the compiler, spec section 3): their re-encoding may violate the count cap.
    pub fn check_decode<T: BiserdiTrait + std::fmt::Debug>(data: &[u8], roundtrip: bool) -> Result<(), DecodeError> {
        let mut bides = Bides::from_slice(data);
        let r = T::bit_deserialize(1, &mut bides);
        let try_r = try_deserialize::<T>(data);
        match r {
            None => {
                let e = bides.error().expect("failed decode must record an error");
                assert_eq!(try_r.err(), Some(e));
                Err(e)
            }
            Some((v, bits)) => {
                assert_eq!(bits, bides.position_bits(), "reported size != consumed bits");
                assert!(bits <= (data.len() as u64) * 8);
                assert!(bits >= T::min_bits(), "min_bits {} > {}", T::min_bits(), bits);
                if let Some(m) = T::max_bits() { assert_eq!(m, bits); }
                assert!(bides.error().is_none());
                let (_, sizes) = try_r.expect("try_deserialize must agree");
                assert_eq!(sizes.total_bits, bits);
                let _ = format!("{:?}", v);
                if !roundtrip { return Ok(()); }
                // every decoded value is in range -> encodes without error
                let (d1, s1) = try_serialize(&v).expect("decoded value must be encodable");
                let (v2, s2) = try_deserialize::<T>(&d1).expect("re-encoded value must decode");
                assert_eq!(s1, s2);
                let (d2, _) = try_serialize(&v2).unwrap();
                assert_eq!(d1, d2, "encoding not idempotent for {:?}", v);
                Ok(())
            }
        }
    }

    /// All strict prefixes of a valid encoding fail with UnexpectedEnd.
    pub fn check_truncation<T: BiserdiTrait + std::fmt::Debug>(v: &T) {
        let (d, s) = try_serialize(v).unwrap();
        assert_eq!(d.len() as u64, s.total_bytes);
        let (vv, ss) = try_deserialize::<T>(&d).unwrap();
        // compare encodings (f32/f64 NaN != NaN)
        assert_eq!(try_serialize(&vv).unwrap().0, d);
        assert_eq!(ss, s);
        assert_eq!(try_serialize(&try_deserialize_exact::<T>(&d).unwrap()).unwrap().0, d);
        for l in 0..d.len() {
            assert_eq!(try_deserialize::<T>(&d[..l]).err(), Some(DecodeError::UnexpectedEnd),
                "prefix of {} of {} bytes decoded for {:?}", l, d.len(), v);
            assert!(deserialize::<T>(&d[..l].to_vec()).is_none());
        }
    }
}

#[cfg(test)]
mod types {
    use crate::*;
    use super::helpers::Rng;

    #[derive(BiserdiEnum, Debug, Clone, Copy, PartialEq, Default)]
    #[biserdi_enum_id_dynbits(2)]
    pub enum Color { #[default] Red, Green, Blue }

    #[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
    pub struct Inner { pub a: IntWithGivenBitSize<u8, 3>, pub b: bool }

    #[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
    pub struct Empty {}

    #[derive(BiserdiOneOf, Debug, Clone, PartialEq)]
    #[biserdi_enum_id_dynbits(2)]
    pub enum OO {
        I(Inner),
        U(DynInteger<u32, 32, 4>),
        S(BitisAString<4>),
        N(IntWithGivenBitSize<i16, 9>),
    }
    impl Default for OO { fn default() -> Self { OO::I(Inner::default()) } }

    #[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
    pub struct All {
        pub b: bool,
        pub u5: IntWithGivenBitSize<u8, 5>,
        pub i12: IntWithGivenBitSize<i16, 12>,
        pub i16f: IntWithGivenBitSize<i16, 16>,
        pub u64f: IntWithGivenBitSize<u64, 64>,
        pub du: DynInteger<u16, 12, 4>,
        pub di: DynInteger<i32, 20, 5>,
        pub fp: FixPrecisionMinMax<10, -50, 50>,
        pub f: f32,
        pub d: f64,
        pub s: BitisAString<4>,
        pub bin: Binary<3>,
        pub o: BitisOption<Inner>,
        pub fa: FixedArray<IntWithGivenBitSize<u8, 2>, 3>,
        pub da: DynArray<Inner, 3>,
        pub e: Color,
        pub oo: OO,
    }

    #[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
    pub struct Fixed {
        pub b: bool,
        pub u: IntWithGivenBitSize<u16, 11>,
        pub i: IntWithGivenBitSize<i32, 20>,
        pub fp: FixPrecisionMinMax<7, 0, 100>,
        pub f: f32,
        pub fa: FixedArray<Inner, 2>,
    }

    // versioned message (bitis-exp case "versioned_full_dyn")
    #[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
    pub struct WithDyn { pub d: DynInteger<u16, 8, 4> }
    #[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
    pub struct ExtV1 {}
    #[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
    pub struct Ext1 { pub v1: ExtV1, pub v2: WithDyn }
    #[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
    pub struct Base { pub b: bool }
    #[derive(bitis_macros::BiserdiMsgVersioned, Debug, Clone, PartialEq, Default)]
    pub struct Versioned { pub base: Base, pub ext: Ext1 }
    #[derive(bitis_macros::BiserdiMsgVersioned, Debug, Clone, PartialEq, Default)]
    pub struct VersionedOld { pub base: Base, pub ext: ExtOld }
    #[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
    pub struct ExtOld { pub v1: ExtV1 }

    fn ascii(rng: &mut Rng, max_len: u64) -> BitisAString<4> {
        let n = rng.below(max_len + 1) as usize;
        let s: String = (0..n).map(|_| (rng.below(0x80) as u8) as char).collect();
        BitisAString::try_from(s.as_str()).unwrap()
    }
    fn inner(rng: &mut Rng) -> Inner {
        Inner { a: IntWithGivenBitSize::try_new(rng.below(8) as u8).unwrap(), b: rng.bool() }
    }
    pub fn random_all(rng: &mut Rng) -> All {
        All {
            b: rng.bool(),
            u5: (rng.below(32) as u8).into(),
            i12: (rng.signed(4095) as i16).into(),
            i16f: (rng.signed(i16::MAX as u64) as i16).into(),
            u64f: rng.next_u64().into(),
            du: (rng.below(1 << 12) as u16).into(),
            di: (rng.signed((1 << 20) - 1) as i32).into(),
            fp: match rng.below(10) { 0 => f64::NAN.into(), 1 => 1000.0.into(), _ => (rng.below(10000) as f64 / 100.0 - 50.0).into() },
            f: f32::from_bits(rng.next_u64() as u32),
            d: (rng.next_u64() as f64) / 7.0,
            s: ascii(rng, 20),
            bin: { let n = rng.below(10) as usize; Binary::new(rng.bytes(n)) },
            o: if rng.bool() { BitisOption::new_some(inner(rng)) } else { BitisOption::new_none() },
            fa: [(rng.below(4) as u8).into(), (rng.below(4) as u8).into(), (rng.below(4) as u8).into()].into(),
            da: (0..rng.below(6)).map(|_| inner(rng)).collect::<Vec<_>>().into(),
            e: match rng.below(3) { 0 => Color::Red, 1 => Color::Green, _ => Color::Blue },
            oo: match rng.below(4) {
                0 => OO::I(inner(rng)),
                1 => OO::U((rng.next_u64() as u32).into()),
                2 => OO::S(ascii(rng, 5)),
                _ => OO::N((rng.signed(511) as i16).into()),
            },
        }
    }
}

// ****************************************************************************
#[cfg(test)]
mod decoding_safety {
    use crate::*;
    use super::helpers::*;
    use super::types::*;

    #[test]
    fn truncated_bool_u8_fails() {
        // bitis-exp case "truncated": {bool, u8} from 1 of 2 bytes decoded as Some(.., 2 bytes)
        #[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
        struct BoolU8 { b: bool, x: IntWithGivenBitSize<u16, 8> }
        let m = BoolU8 { b: true, x: 0xFF.into() };
        let (d, s) = serialize(&m);
        assert_eq!((d.len(), s.total_bits), (2, 9));
        assert!(deserialize::<BoolU8>(&vec![d[0]]).is_none());
        assert_eq!(try_deserialize::<BoolU8>(&d[..1]).err(), Some(DecodeError::UnexpectedEnd));
        assert_eq!(try_deserialize::<BoolU8>(&d).unwrap().0, m);
    }

    #[test]
    fn truncated_inputs_at_every_length() {
        let mut rng = Rng::new(42);
        for _ in 0..200 {
            check_truncation(&random_all(&mut rng));
        }
        check_truncation(&All::default());
        check_truncation(&Fixed { b: true, u: 2047.into(), i: (-524287).into(), fp: 33.0.into(), f: 1.5,
            fa: [Inner { a: 7.into(), b: true }, Inner::default()].into() });
        for d in [0_u16, 1, 15, 16, 255] {
            check_truncation(&Versioned { base: Base { b: true }, ext: Ext1 { v1: ExtV1 {}, v2: WithDyn { d: d.into() } } });
        }
        check_truncation(&DynArray::<Empty, 4>::from(vec![Empty {}, Empty {}]));
    }

    #[test]
    fn bit_level_truncation() {
        // reading beyond the data must fail also within the last byte
        let mut b = Bides::from_slice(&[0xFF]);
        assert_eq!(b.decode_data_base_u8(5), Some((0x1F, 0)));
        assert_eq!(b.decode_data_base_u8(4), None);
        assert_eq!(b.error(), Some(DecodeError::UnexpectedEnd));
        assert_eq!(b.decode_data_base_u8(3), Some((0x07, 0)));
        assert_eq!(b.remaining_bits(), 0);
        assert!(b.decode_data(1, 1).is_none());

        let mut b = Bides::from_slice(&[0xFF, 0x01]);
        assert!(b.try_skip_bits(17).is_none());
        assert_eq!(b.position_bits(), 0);
        b.skip_bits(17);
        assert_eq!(b.remaining_bits(), 0);
        b.reset_position();
        assert_eq!(b.error(), None);
        assert!(b.try_skip_bits(16).is_some());
    }

    #[test]
    fn first_error_wins() {
        let mut b = Bides::new();
        b.set_error(DecodeError::InvalidValue);
        b.set_error(DecodeError::UnexpectedEnd);
        assert!(b.fail::<()>(DecodeError::TrailingData).is_none());
        assert_eq!(b.error(), Some(DecodeError::InvalidValue));
        let mut s = Biseri::new();
        assert_eq!(s.error(), None);
        s.set_error(EncodeError::ValueOutOfRange);
        assert_eq!(s.error(), Some(EncodeError::ValueOutOfRange));
        assert_eq!(DecodeError::UnexpectedEnd.to_string(), "unexpected end of input");
        let _e: Box<dyn std::error::Error> = Box::new(EncodeError::ValueOutOfRange);
    }

    #[test]
    fn trailing_data() {
        let m = Base { b: true };
        assert_eq!(try_deserialize::<Base>(&[0x01, 0xAA, 0xBB]).unwrap(), (m.clone(), BiserSizes { total_bits: 1, total_bytes: 1 }));
        assert_eq!(deserialize::<Base>(&vec![0x01, 0xAA]).unwrap().1.total_bytes, 1);
        assert_eq!(try_deserialize_exact::<Base>(&[0x01, 0xAA]).err(), Some(DecodeError::TrailingData));
        assert_eq!(try_deserialize_exact::<Base>(&[0x01]).unwrap(), m);
        // padding bits are not checked
        assert_eq!(try_deserialize_exact::<Base>(&[0xFF]).unwrap(), m);
        assert_eq!(try_deserialize_exact::<Empty>(&[]).unwrap(), Empty {});
        assert_eq!(try_deserialize_exact::<Empty>(&[0]).err(), Some(DecodeError::TrailingData));
    }

    #[test]
    fn count_caps() {
        // bitis-exp case "zst_dos": DynArray of empty messages with count 2^28
        let mut b = Biseri::new();
        DynInteger::<u32, 32, 8>::new(1 << 28).bit_serialize(&mut b);
        b.finish_add_data();
        let t = std::time::Instant::now();
        assert_eq!(try_deserialize::<DynArray<Empty, 8>>(b.get_data_ref()).err(), Some(DecodeError::UnexpectedEnd));
        assert!(t.elapsed() < std::time::Duration::from_secs(1));
        assert_eq!(try_deserialize::<Binary<8>>(b.get_data_ref()).err(), Some(DecodeError::UnexpectedEnd));
        assert_eq!(try_deserialize::<BitisAString<8>>(b.get_data_ref()).err(), Some(DecodeError::UnexpectedEnd));

        // count 3 of 4-bit elements needs 12 bits: rejected with 11 remaining, accepted with 12
        let enc = |n: u32, extra_bits: u64| {
            let mut b = Biseri::new();
            DynInteger::<u32, 32, 8>::new(n).bit_serialize(&mut b);
            b.add_data(&vec![0xFF; 8], extra_bits);
            b.finish_add_data();
            b.get_data()
        };
        // count 3 takes 10 bits -> total 22 bits = 3 bytes, remaining 14 >= 12
        assert_eq!(try_deserialize::<DynArray<IntWithGivenBitSize<u8, 4>, 8>>(&enc(3, 12)).unwrap().0.val.len(), 3);
        // 10 + 11 bits = 21 bits -> 3 bytes (3 padding bits) -> 14 remaining, ok as well
        assert!(try_deserialize::<DynArray<IntWithGivenBitSize<u8, 4>, 8>>(&enc(3, 11)).is_ok());
        // 10 + 1 = 11 bits -> 2 bytes -> 6 remaining < 12 -> fails before allocating
        assert_eq!(try_deserialize::<DynArray<IntWithGivenBitSize<u8, 4>, 8>>(&enc(3, 1)).err(), Some(DecodeError::UnexpectedEnd));
        // min_bits of the element type is used (Inner: 4 bits)
        assert_eq!(try_deserialize::<DynArray<Inner, 8>>(&enc(3, 1)).err(), Some(DecodeError::UnexpectedEnd));
    }

    fn fuzz_type<T: BiserdiTrait + std::fmt::Debug>(rng: &mut Rng, valid: &[Vec<u8>], iterations: usize) -> (usize, usize) {
        fuzz_type_ex::<T>(rng, valid, iterations, true)
    }
    fn fuzz_type_ex<T: BiserdiTrait + std::fmt::Debug>(rng: &mut Rng, valid: &[Vec<u8>], iterations: usize, roundtrip: bool) -> (usize, usize) {
        let mut ok = 0;
        let mut err = 0;
        for i in 0..iterations {
            let data = if valid.is_empty() || i % 2 == 0 {
                let n = rng.below(48) as usize;
                rng.bytes(n)
            } else {
                // mutate a valid encoding: flip bits, truncate or extend
                let mut d = valid[rng.below(valid.len() as u64) as usize].clone();
                for _ in 0..=rng.below(3) {
                    if d.is_empty() { break; }
                    let p = rng.below(d.len() as u64 * 8);
                    d[(p / 8) as usize] ^= 1 << (p % 8);
                }
                match rng.below(4) {
                    0 => { let l = rng.below(d.len() as u64 + 1) as usize; d.truncate(l); }
                    1 => { let n = rng.below(4) as usize; d.extend(rng.bytes(n)); }
                    _ => {}
                }
                d
            };
            match check_decode::<T>(&data, roundtrip) { Ok(()) => ok += 1, Err(_) => err += 1 }
        }
        (ok, err)
    }

    #[test]
    fn fuzz_random_buffers() {
        let mut rng = Rng::new(0x5eed);
        let valid_all: Vec<Vec<u8>> = (0..64).map(|_| serialize(&random_all(&mut rng)).0).collect();
        let valid_v: Vec<Vec<u8>> = [0_u16, 3, 255].iter().map(|d|
            serialize(&Versioned { base: Base { b: true }, ext: Ext1 { v1: ExtV1 {}, v2: WithDyn { d: (*d).into() } } }).0).collect();

        let mut total = 0;
        let (o, e) = fuzz_type::<All>(&mut rng, &valid_all, 40_000); total += o + e;
        assert!(o > 0 && e > 0);
        let (o, e) = fuzz_type::<Fixed>(&mut rng, &[], 10_000); total += o + e;
        assert!(o > 0 && e > 0);
        let (o, e) = fuzz_type::<OO>(&mut rng, &[], 10_000); total += o + e;
        assert!(o > 0 && e > 0);
        let (o, e) = fuzz_type::<Versioned>(&mut rng, &valid_v, 10_000); total += o + e;
        assert!(o > 0 && e > 0);
        let (o, e) = fuzz_type::<VersionedOld>(&mut rng, &valid_v, 5_000); total += o + e;
        assert!(o > 0 && e > 0);
        fuzz_type_ex::<DynArray<Empty, 2>>(&mut rng, &[], 5_000, false);
        fuzz_type::<DynArray<BitisAString<2>, 2>>(&mut rng, &[], 5_000);
        fuzz_type::<DynInteger<i64, 64, 3>>(&mut rng, &[], 5_000);
        fuzz_type::<IntWithGivenBitSize<i16, 16>>(&mut rng, &[], 5_000);
        fuzz_type::<FixPrecisionMinMax<35, -180, 180>>(&mut rng, &[], 5_000);
        total += 25_000;
        assert_eq!(total, 100_000);
    }

    #[test]
    fn roundtrip_random_values() {
        let mut rng = Rng::new(7);
        for _ in 0..2000 {
            let v = random_all(&mut rng);
            let (d, s) = try_serialize(&v).unwrap();
            let (vv, ss) = try_deserialize::<All>(&d).unwrap();
            assert_eq!(s, ss);
            // f32 NaN != NaN: compare encodings instead
            assert_eq!(try_serialize(&vv).unwrap().0, d);
            assert!(s.total_bits >= All::min_bits());
        }
    }
}

// ****************************************************************************
#[cfg(test)]
mod integers {
    use crate::*;
    use super::helpers::*;

    #[test]
    fn min_max_constants() {
        assert_eq!(IntWithGivenBitSize::<u16, 3>::MIN, 0);
        assert_eq!(IntWithGivenBitSize::<u16, 3>::MAX, 7);
        assert_eq!(IntWithGivenBitSize::<i8, 4>::MIN, -15);
        assert_eq!(IntWithGivenBitSize::<i8, 4>::MAX, 15);
        assert_eq!(IntWithGivenBitSize::<i16, 16>::MIN, -32767);
        assert_eq!(IntWithGivenBitSize::<i16, 16>::MAX, 32767);
        assert_eq!(IntWithGivenBitSize::<i16, 15>::MAX, 32767);
        assert_eq!(IntWithGivenBitSize::<i16, 14>::MIN, -16383);
        assert_eq!(IntWithGivenBitSize::<i64, 64>::MIN, -i64::MAX);
        assert_eq!(IntWithGivenBitSize::<u64, 64>::MAX, u64::MAX);
        assert_eq!(IntWithGivenBitSize::<u32, 27>::MAX, (1 << 27) - 1);
        assert_eq!(DynInteger::<u16, 8, 4>::MAX, 255);
        assert_eq!(DynInteger::<u16, 8, 4>::MIN, 0);
        assert_eq!(DynInteger::<i32, 32, 4>::MIN, -i32::MAX);
        assert_eq!(DynInteger::<i32, 20, 4>::MAX, (1 << 20) - 1);
        assert_eq!(DynInteger::<u64, 64, 8>::MAX, u64::MAX);
    }

    fn rt<T: BiserdiTrait + PartialEq + std::fmt::Debug>(v: T) {
        let (d, _) = try_serialize(&v).unwrap();
        assert_eq!(try_deserialize::<T>(&d).unwrap().0, v);
    }

    #[test]
    fn min_max_roundtrip() {
        type I16 = IntWithGivenBitSize<i16, 16>;
        type I5 = IntWithGivenBitSize<i16, 5>;
        type U3 = IntWithGivenBitSize<u16, 3>;
        type I64 = IntWithGivenBitSize<i64, 64>;
        type U64 = IntWithGivenBitSize<u64, 64>;
        type DI = DynInteger<i64, 64, 8>;
        type DU = DynInteger<u16, 8, 4>;
        rt(I16::new(I16::MIN)); rt(I16::new(I16::MAX));
        rt(I5::new(I5::MIN)); rt(I5::new(I5::MAX));
        rt(U3::new(U3::MAX));
        rt(I64::new(I64::MIN)); rt(I64::new(I64::MAX));
        rt(U64::new(U64::MAX));
        rt(DI::new(DI::MIN)); rt(DI::new(DI::MAX));
        rt(DU::new(DU::MAX));
        rt(DynInteger::<u64, 64, 8>::new(u64::MAX));

        assert!(I16::try_new(i16::MIN).is_err());
        assert_eq!(I16::try_new(-32767).unwrap().val, -32767);
        assert_eq!(U3::try_new(8).err(), Some(EncodeError::ValueOutOfRange));
        assert!(U3::try_new(7).is_ok());
        assert!(I5::try_new(-32).is_err());
        assert!(I5::try_new(-31).is_ok());
        assert!(DU::try_new(256).is_err());
        assert!(DI::try_new(i64::MIN).is_err());
    }

    #[test]
    fn out_of_range_encode() {
        // bitis-exp case "range": uint_3 = 9 -> silently 1
        let v = IntWithGivenBitSize::<u16, 3>::new(9);
        assert_eq!(try_serialize(&v).err(), Some(EncodeError::ValueOutOfRange));
        let (d, _) = serialize(&v);
        assert_eq!(deserialize::<IntWithGivenBitSize<u16, 3>>(&d).unwrap().0.val, 1);
        let mut s = Biseri::new();
        assert_eq!(v.bit_serialize(&mut s), Some(3));
        assert_eq!(s.error(), Some(EncodeError::ValueOutOfRange));

        let v = IntWithGivenBitSize::<i16, 3>::new(-9);
        assert_eq!(try_serialize(&v).err(), Some(EncodeError::ValueOutOfRange));
        assert_eq!(deserialize::<IntWithGivenBitSize<i16, 3>>(&serialize(&v).0).unwrap().0.val, -1);

        // inside a message: error propagates, legacy output unchanged in size
        #[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
        struct M { a: IntWithGivenBitSize<u8, 7>, b: bool }
        let m = M { a: 128.into(), b: true };
        assert_eq!(try_serialize(&m).err(), Some(EncodeError::ValueOutOfRange));
        assert_eq!(serialize(&m).1.total_bits, 8);
        assert_eq!(deserialize::<M>(&serialize(&m).0).unwrap().0, M { a: 0.into(), b: true });
    }

    #[test]
    fn signed_min_no_panic() {
        // bitis-exp cases "imin_ser", "imin_ser_dyn", "imin_de"
        let v = IntWithGivenBitSize::<i16, 16>::new(i16::MIN);
        assert_eq!(try_serialize(&v).err(), Some(EncodeError::ValueOutOfRange));
        let (d, _) = serialize(&v);
        assert_eq!(try_deserialize::<IntWithGivenBitSize<i16, 16>>(&d).err(), Some(DecodeError::InvalidValue));

        let v = DynInteger::<i64, 64, 8>::new(i64::MIN);
        assert_eq!(try_serialize(&v).err(), Some(EncodeError::ValueOutOfRange));
        let (d, _) = serialize(&v);
        assert_eq!(try_deserialize::<DynInteger<i64, 64, 8>>(&d).err(), Some(DecodeError::InvalidValue));

        for t in [i8::MIN as i64, i16::MIN as i64, i32::MIN as i64, i64::MIN] {
            let _ = serialize(&IntWithGivenBitSize::<i64, 64>::new(t));
            let _ = serialize(&DynInteger::<i64, 64, 7>::new(t));
        }
        let _ = serialize(&IntWithGivenBitSize::<i8, 8>::new(i8::MIN));
        let _ = serialize(&DynInteger::<i32, 32, 5>::new(i32::MIN));

        // sign=1, magnitude=0x8000 for int_16
        assert_eq!(try_deserialize::<IntWithGivenBitSize<i16, 16>>(&[0x01, 0x00, 0x01]).err(), Some(DecodeError::InvalidValue));
        // sign=0, magnitude=0xFFFF for int_16
        assert_eq!(try_deserialize::<IntWithGivenBitSize<i16, 16>>(&[0xFE, 0xFF, 0x01]).err(), Some(DecodeError::InvalidValue));
        // sign=0, magnitude=0x7FFF ok
        assert_eq!(try_deserialize::<IntWithGivenBitSize<i16, 16>>(&[0xFE, 0xFF, 0x00]).unwrap().0.val, 32767);
        // int_64: magnitude 2^63 invalid
        let mut b = Biseri::new();
        true.bit_serialize(&mut b);
        (1_u64 << 63).bit_serialize(64, &mut b);
        b.finish_add_data();
        assert_eq!(try_deserialize::<IntWithGivenBitSize<i64, 64>>(b.get_data_ref()).err(), Some(DecodeError::InvalidValue));
    }

    #[test]
    fn signed_wire_format_and_negative_zero() {
        // spec 2.3: sign bit then magnitude
        assert_eq!(ser_bits(&IntWithGivenBitSize::<i16, 5>::new(-3)), "1 11000".replace(' ', ""));
        assert_eq!(ser_bits(&IntWithGivenBitSize::<i16, 5>::new(3)), "011000");
        assert_eq!(ser_bits(&IntWithGivenBitSize::<u16, 3>::new(5)), "101");
        // negative zero -> 0
        let d = from_bit_string("1 00000");
        assert_eq!(try_deserialize::<IntWithGivenBitSize<i16, 5>>(&d).unwrap().0.val, 0);
        let d = from_bit_string("1 0");
        assert_eq!(try_deserialize::<DynInteger<i32, 32, 3>>(&d).unwrap().0.val, 0);
    }

    #[test]
    fn dyn_int_wire_format() {
        // spec 2.4 example: 10 as uint_32d3 -> 1 010 1 100 0
        assert_eq!(ser_bits(&DynInteger::<u32, 32, 3>::new(10)), "101011000");
        assert_eq!(ser_bits(&DynInteger::<u32, 32, 3>::new(0)), "0");
        assert_eq!(ser_bits(&DynInteger::<u32, 32, 3>::new(1)), "1 100 0".replace(' ', ""));
        // 2^k - 1 and 2^k with k = Y
        assert_eq!(ser_bits(&DynInteger::<u32, 32, 3>::new(7)), "1 111 0".replace(' ', ""));
        assert_eq!(ser_bits(&DynInteger::<u32, 32, 3>::new(8)), "1 000 1 100 0".replace(' ', ""));
        // MAX_BITS used up: no continuation bit after the last chunk
        assert_eq!(ser_bits(&DynInteger::<u16, 8, 4>::new(15)), "1 1111 0".replace(' ', ""));
        assert_eq!(ser_bits(&DynInteger::<u16, 8, 4>::new(16)), "1 0000 1 1000".replace(' ', ""));
        assert_eq!(ser_bits(&DynInteger::<u16, 8, 4>::new(255)), "1 1111 1 1111".replace(' ', ""));
        // last chunk smaller than Y
        assert_eq!(ser_bits(&DynInteger::<u16, 7, 3>::new(127)), "1 111 1 111 1 1".replace(' ', ""));
        assert_eq!(ser_bits(&DynInteger::<u16, 7, 3>::new(64)), "1 000 1 000 1 1".replace(' ', ""));
        // max value with 64 bits: 1 + 64 + 7 continuation bits
        assert_eq!(ser_bits(&DynInteger::<u64, 64, 8>::new(u64::MAX)), "1".repeat(72));
        assert_eq!(ser_bits(&DynInteger::<u64, 64, 64>::new(u64::MAX)), "1".repeat(65));
        // spec 2.5: signed, zero takes 2 bits
        assert_eq!(ser_bits(&DynInteger::<i32, 32, 3>::new(0)), "00");
        assert_eq!(ser_bits(&DynInteger::<i32, 32, 3>::new(-10)), "1101011000");

        // decoding: reported size equals the written size (off-by-one when MAX_BITS used up)
        for v in [0_u16, 1, 15, 16, 127, 128, 255] {
            let (d, s) = serialize(&DynInteger::<u16, 8, 4>::new(v));
            let mut b = Bides::from_slice(&d);
            let (dv, bits) = DynInteger::<u16, 8, 4>::bit_deserialize(1, &mut b).unwrap();
            assert_eq!((dv.val, bits), (v, s.total_bits));
            assert_eq!(b.position_bits(), bits);
        }
        // non-minimal encodings are accepted
        assert_eq!(try_deserialize::<DynInteger<u32, 32, 3>>(&from_bit_string("1 010 1 000 0")).unwrap().0.val, 2);
        // magnitude outside the base type range
        assert_eq!(try_deserialize::<DynInteger<i16, 16, 4>>(&from_bit_string("0 1 1111 1 1111 1 1111 1 1111")).err(),
            Some(DecodeError::InvalidValue));
        assert_eq!(try_deserialize::<DynInteger<i16, 16, 4>>(&from_bit_string("0 1 1111 1 1111 1 1111 1 1110")).unwrap().0.val,
            i16::MAX);
    }

    #[test]
    fn dyn_int_value_exceeding_max_bits_does_not_hang() {
        // bitis-exp case "dyn_hang"
        let v = DynInteger::<u16, 8, 4>::new(300);
        assert_eq!(try_serialize(&v).err(), Some(EncodeError::ValueOutOfRange));
        let (d, s) = serialize(&v);
        assert_eq!(s.total_bits, 10);
        assert_eq!(deserialize::<DynInteger<u16, 8, 4>>(&d).unwrap().0.val, 300 & 255);
        // masked to zero
        assert_eq!(ser_bits(&DynInteger::<u16, 8, 4>::new(256)), "0");
    }

    #[test]
    fn wide_primitive_values() {
        // values written with more bits than 64: zero bits above bit 63
        let mut b = Biseri::new();
        assert_eq!(5_u64.bit_serialize(70, &mut b), Some(70));
        assert_eq!(b.error(), None);
        let s = b.finish_add_data().unwrap();
        assert_eq!(s.total_bits, 70);
        let mut d = Bides::from_slice(b.get_data_ref());
        assert_eq!(u64::bit_deserialize(1, 70, &mut d), Some((5, 70)));
        let mut d = Bides::from_slice(&[0xFF; 9]);
        assert_eq!(u64::bit_deserialize(1, 70, &mut d), None);
        assert_eq!(d.error(), Some(DecodeError::InvalidValue));
    }

    #[test]
    fn get_bits_num() {
        assert_eq!(<u32 as IntegerBaseFunctions>::get_bits_num(), 32);
        assert_eq!(<u64 as IntegerBaseFunctions>::get_bits_num(), 64);
        assert_eq!(<i16 as IntegerBaseFunctions>::get_bits_num(), 16);
        assert_eq!(i64::MIN.get_absolute_uint(), 1 << 63);
        assert_eq!(i8::MIN.switch_sign_if_possible(), i8::MIN);
    }
}

// ****************************************************************************
#[cfg(test)]
mod strings_enums {
    use crate::*;
    use super::types::*;

    #[test]
    fn non_ascii_decode_rejected() {
        // bitis-exp case "astr": length 1 followed by 0xE9
        let mut b = Biseri::new();
        DynInteger::<u32, 32, 4>::new(1).bit_serialize(&mut b);
        0xE9_u8.bit_serialize(8, &mut b);
        b.finish_add_data();
        assert_eq!(try_deserialize::<BitisAString<4>>(b.get_data_ref()).err(), Some(DecodeError::InvalidValue));
        assert!(deserialize::<BitisAString<4>>(&b.get_data()).is_none());
        // binary accepts it
        assert_eq!(try_deserialize::<Binary<4>>(b.get_data_ref()).unwrap().0.val, vec![0xE9]);
    }

    #[test]
    fn ascii_conversions() {
        let s = BitisAString::<4>::try_from("Hello").unwrap();
        assert_eq!(s.get_string(), "Hello");
        assert_eq!(format!("{:?}", s), "'Hello'");
        assert_eq!(format!("{}", s), "'Hello' |string:4");
        assert_eq!(BitisAString::<4>::try_from("Grüße").err(), Some(EncodeError::ValueOutOfRange));
        assert_eq!(BitisAString::<4>::try_from_string("Grüße".to_string()).err(), Some(EncodeError::ValueOutOfRange));
        assert_eq!(BitisAString::<4>::try_from_string("abc".to_string()).unwrap().get_string(), "abc");
        // legacy: non-ASCII becomes the empty string
        let s: BitisAString<4> = String::from("Grüße").into();
        assert_eq!(s.get_string(), "");
        let (d, _) = try_serialize(&BitisAString::<4>::try_from("abc").unwrap()).unwrap();
        assert_eq!(try_deserialize::<BitisAString<4>>(&d).unwrap().0.get_string(), "abc");
    }

    #[test]
    fn unknown_discriminant() {
        // bitis-exp case "enum_unknown"
        for idx in [4_u32, 5, 1000] {
            let mut b = Biseri::new();
            DynInteger::<u32, 32, 2>::new(idx).bit_serialize(&mut b);
            b.add_data(&vec![0; 8], 64);
            b.finish_add_data();
            assert_eq!(try_deserialize::<Color>(b.get_data_ref()).err(), Some(DecodeError::InvalidDiscriminant));
            assert_eq!(try_deserialize::<OO>(b.get_data_ref()).err(), Some(DecodeError::InvalidDiscriminant));
            assert!(deserialize::<Color>(&b.get_data()).is_none());
        }
        assert_eq!(try_deserialize::<Color>(&[0b0111]).err(), Some(DecodeError::InvalidDiscriminant));
        let (d, _) = serialize(&Color::Blue);
        assert_eq!(try_deserialize::<Color>(&d).unwrap().0, Color::Blue);
        // truncated discriminant
        assert_eq!(try_deserialize::<Color>(&[]).err(), Some(DecodeError::UnexpectedEnd));
    }

    #[test]
    fn min_max_bits() {
        assert_eq!(IntWithGivenBitSize::<i16, 5>::min_bits(), 6);
        assert_eq!(IntWithGivenBitSize::<i16, 5>::max_bits(), Some(6));
        assert_eq!(IntWithGivenBitSize::<u16, 5>::min_bits(), 5);
        assert_eq!(DynInteger::<i32, 32, 4>::min_bits(), 2);
        assert_eq!(DynInteger::<u32, 32, 4>::min_bits(), 1);
        assert_eq!(DynInteger::<u32, 32, 4>::max_bits(), None);
        assert_eq!(FixPrecisionMinMax::<10, 0, 1>::max_bits(), Some(10));
        assert_eq!(<bool as BiserdiTrait>::max_bits(), Some(1));
        assert_eq!(<f64 as BiserdiTrait>::max_bits(), Some(64));
        assert_eq!(FixedArray::<IntWithGivenBitSize<i8, 3>, 4>::max_bits(), Some(16));
        assert_eq!(BitisOption::<bool>::max_bits(), None);
        // oneof: 1 + min over variants (Inner 4, U 1, S 1, N 10)
        assert_eq!(OO::min_bits(), 2);
        assert_eq!(OO::max_bits(), None);
        assert_eq!(Color::min_bits(), 1);
        assert_eq!(Inner::min_bits(), 4);
        assert_eq!(Inner::max_bits(), Some(4));
        assert_eq!(Fixed::max_bits(), Some(1 + 11 + 21 + 7 + 32 + 8));
        assert_eq!(Fixed::min_bits(), Fixed::max_bits().unwrap());
        assert_eq!(serialize(&Fixed::default()).1.total_bits, Fixed::max_bits().unwrap());
        assert_eq!(Empty::max_bits(), Some(0));
        assert_eq!(All::max_bits(), None);
        assert_eq!(Versioned::min_bits(), 2);

        #[derive(BiserdiOneOf, Debug, Clone, PartialEq)]
        #[biserdi_enum_id_dynbits(3)]
        enum OOEmpty {}
        assert_eq!(OOEmpty::min_bits(), 1);
        assert_eq!(try_deserialize::<OOEmpty>(&[0]).err(), Some(DecodeError::InvalidDiscriminant));

        #[derive(BiserdiOneOf, Debug, Clone, PartialEq)]
        #[biserdi_enum_id_dynbits(3)]
        enum OOInts { A(IntWithGivenBitSize<i8, 5>), B(IntWithGivenBitSize<u8, 3>) }
        assert_eq!(OOInts::min_bits(), 4);
    }

    #[test]
    fn display_no_panic() {
        assert_eq!(format!("{}", DynInteger::<u16, 8, 4>::new(5)), "5 |8d4");
        let _ = format!("{} {:?}", All::default(), All::default());
    }
}

// ****************************************************************************
#[cfg(test)]
mod fixed_point {
    use crate::*;

    type F = FixPrecisionMinMax<8, -2, 3>;

    fn raw<const N: u8, const MI: i64, const MA: i64>(v: &FixPrecisionMinMax<N, MI, MA>) -> u64 {
        let (d, _) = try_serialize(v).unwrap();
        let mut b = [0_u8; 8];
        b[..d.len()].copy_from_slice(&d);
        u64::from_le_bytes(b)
    }

    #[test]
    fn edges() {
        assert_eq!(raw(&F::new(-2.0)), 1);
        assert_eq!(raw(&F::new(3.0)), 254);
        assert_eq!(raw(&F::new(f64::NAN)), 0);
        assert_eq!(raw(&F::new(f64::INFINITY)), 255);
        assert_eq!(raw(&F::new(f64::NEG_INFINITY)), 0);
        assert_eq!(raw(&F::new(3.0001)), 255);
        assert_eq!(raw(&F::new(-2.0001)), 0);
        assert!(F::new(f64::NAN).is_underflow());
        assert_eq!(F::new(f64::NAN).val, FixPrecisionVal::Underflow);
        assert!(F::new(f64::INFINITY).is_overflow());
        assert!(F::new(f64::NEG_INFINITY).is_underflow());
        assert_eq!(F::new(3.0).value(), Some(3.0));
        assert_eq!(F::new(-2.0).value(), Some(-2.0));
        assert_eq!(F::new(4.0).value(), None);
        assert!(!F::new(1.0).is_overflow() && !F::new(1.0).is_underflow());

        // directly constructed states are normalized
        let nan = F { val: FixPrecisionVal::Value(f64::NAN) };
        assert!(nan.is_underflow());
        assert_eq!(nan.value(), None);
        assert_eq!(raw(&nan), 0);
        let big = F { val: FixPrecisionVal::Value(100.0) };
        assert!(big.is_overflow());
        assert_eq!(raw(&big), 255);

        // decoding of raw values
        assert_eq!(try_deserialize::<F>(&[0]).unwrap().0.val, FixPrecisionVal::Underflow);
        assert_eq!(try_deserialize::<F>(&[255]).unwrap().0.val, FixPrecisionVal::Overflow);
        assert_eq!(try_deserialize::<F>(&[254]).unwrap().0.value(), Some(3.0));
        assert_eq!(try_deserialize::<F>(&[1]).unwrap().0.value(), Some(-2.0));

        // legacy sentinels
        assert_eq!(F::new(-5.0).val_into(), -3.0);
        assert_eq!(F::new(5.0).val_into(), 4.0);
        let x: f64 = F::new(5.0).into();
        assert_eq!(x, 4.0);
    }

    #[test]
    fn rounding_half_away_from_zero() {
        // fp_3[0,10]: S = 5, R = 10, t = v / 10 * 5 + 1
        type G = FixPrecisionMinMax<3, 0, 10>;
        type H = FixPrecisionMinMax<3, -10, 0>;
        for (v, t_exp, r_exp) in [(1.0, 1.5, 2_u64), (3.0, 2.5, 3), (5.0, 3.5, 4), (9.0, 5.5, 6)] {
            let t = (v - 0.0) / 10.0 * 5.0 + 1.0;
            assert_eq!(t, t_exp, "precondition: t exactly .5");
            assert_eq!(raw(&G::new(v)), r_exp, "v={}", v);
        }
        // negative range: v - min = 3 -> t = 2.5 -> 3
        assert_eq!(raw(&H::new(-7.0)), 3);
        assert_eq!(raw(&H::new(-9.0)), 2);
        assert_eq!(raw(&H::new(-10.0)), 1);
        assert_eq!(raw(&H::new(0.0)), 6);
    }

    #[test]
    fn roundtrip_quantized() {
        type L = FixPrecisionMinMax<35, -180, 180>;
        for v in [-180.0, -179.999999, -0.000001, 0.0, 13.4050, 179.9999999, 180.0] {
            let x = L::new(v);
            let (d, _) = try_serialize(&x).unwrap();
            let y = try_deserialize::<L>(&d).unwrap().0;
            assert_eq!(x, y);
            assert!((y.value().unwrap() - v).abs() < 360.0 / ((1_u64 << 35) as f64));
        }
    }
}

// ****************************************************************************
#[cfg(test)]
mod versioned {
    use crate::*;
    use super::types::*;

    #[test]
    fn full_dyn_in_ext() {
        // bitis-exp case "versioned_full_dyn": ext with dyn int using all MAX_BITS
        for d in [0_u16, 15, 16, 255] {
            let m = Versioned { base: Base { b: true }, ext: Ext1 { v1: ExtV1 {}, v2: WithDyn { d: d.into() } } };
            let (data, s) = try_serialize(&m).unwrap();
            let (mm, ss) = try_deserialize::<Versioned>(&data).unwrap();
            assert_eq!(mm, m);
            assert_eq!(ss, s);
            // old decoder skips the unknown part
            let (old, so) = try_deserialize::<VersionedOld>(&data).unwrap();
            assert_eq!(old.base, m.base);
            assert_eq!(so, s);
        }
    }

    #[test]
    fn oversized_ext_size() {
        // bitis-exp case "versioned_overflow": ext size u64::MAX
        let mut b = Biseri::new();
        true.bit_serialize(&mut b);
        DynInteger::<u64, 64, 4>::new(u64::MAX).bit_serialize(&mut b);
        false.bit_serialize(&mut b);
        false.bit_serialize(&mut b);
        b.finish_add_data();
        assert_eq!(try_deserialize::<Versioned>(b.get_data_ref()).err(), Some(DecodeError::UnexpectedEnd));

        // bitis-exp case "vskip": ext size larger than the remaining data
        let mut b = Biseri::new();
        true.bit_serialize(&mut b);
        DynInteger::<u64, 64, 4>::new(100).bit_serialize(&mut b);
        false.bit_serialize(&mut b);
        b.finish_add_data();
        assert_eq!(try_deserialize::<VersionedOld>(b.get_data_ref()).err(), Some(DecodeError::UnexpectedEnd));
    }

    #[test]
    fn ext_larger_than_announced() {
        // ext announced with 1 bit, but Ext1 needs at least 1 bit plus its dyn int
        let mut b = Biseri::new();
        true.bit_serialize(&mut b);
        DynInteger::<u64, 64, 4>::new(1).bit_serialize(&mut b);
        b.add_data(&vec![0xFF; 2], 16);
        b.finish_add_data();
        assert_eq!(try_deserialize::<Versioned>(b.get_data_ref()).err(), Some(DecodeError::InvalidValue));
    }

    #[test]
    fn encode_error_in_ext_propagates() {
        let m = Versioned { base: Base { b: true }, ext: Ext1 { v1: ExtV1 {}, v2: WithDyn { d: 300.into() } } };
        assert_eq!(try_serialize(&m).err(), Some(EncodeError::ValueOutOfRange));
    }
}

// ****************************************************************************
// The derive macros must not depend on glob imports or on names at the use site.
#[cfg(test)]
mod macro_hygiene {
    #![allow(dead_code)]
    // shadowing names that the generated code must not pick up
    struct Option;
    struct Biseri;
    struct Bides;
    struct DynInteger;
    trait BiserdiTrait {}

    #[derive(crate::BiserdiEnum, Debug, Clone, PartialEq)]
    #[biserdi_enum_id_dynbits(3)]
    enum E { A, B }

    #[derive(crate::BiserdiMsg, Debug, Clone, PartialEq)]
    struct M { a: crate::IntWithGivenBitSize<u8, 3>, e: E, o: OO }

    #[derive(crate::BiserdiOneOf, Debug, Clone, PartialEq)]
    #[biserdi_enum_id_dynbits(2)]
    enum OO { X(bool), Y(crate::DynInteger<u16, 10, 3>) }

    #[derive(bitis_macros::BiserdiMsgVersioned, Debug, Clone, PartialEq)]
    struct V { base: M, ext: M }

    #[test]
    fn derive_without_imports() {
        let m = M { a: 5.into(), e: E::B, o: OO::Y(700.into()) };
        let (d, _) = crate::try_serialize(&m).unwrap();
        assert_eq!(crate::try_deserialize_exact::<M>(&d).unwrap(), m);
        let v = V { base: m.clone(), ext: m };
        let (d, _) = crate::try_serialize(&v).unwrap();
        assert_eq!(crate::try_deserialize_exact::<V>(&d).unwrap(), v);
        assert_eq!(format!("{}", E::A), "A");
    }
}
