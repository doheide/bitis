// #![feature(trace_macros)]
//
// trace_macros!(true);
//

pub mod msg_test;
pub mod lib_impl;

pub use lib_impl::berde::*;
pub use lib_impl::compiler::*;
pub use bitis_macros::{BiserdiMsg, BiserdiOneOf, BiserdiEnum};

pub fn serialize<T: BiserdiTrait>(data: &T) -> Vec<u8>{
    let mut ser = Biseri::new();

    data.bit_serialize(&mut ser);
    let (_bits, _bytes) = ser.finish_add_data().unwrap();
    // println!("bits: {}, bytes: {}", _bits, _bytes);

    ser.get_data().to_owned()
}
pub fn deserialize<T: BiserdiTrait>(data: &Vec<u8>) -> Option<(T, u64)> {
    let mut der = Bides::from_vec(data);

    T::bit_deserialize(1, &mut der)
}

#[cfg(test)]
mod msg_deserialization {
    use rstest::rstest;
    use super::*;


    #[derive(BiserdiMsg, Debug, Clone, PartialEq)]
    struct MsgLalaBase {
        a1: VarWithGivenBitSize<u16, 13>,
        b1: bool,
        b2: bool,
        f: f32,
    }

    #[derive(BiserdiMsg, Debug, Clone, PartialEq)]
    struct MsgLili {
        inner_msg: MsgLalaBase,
        b1: bool,
        b2: VarWithGivenBitSize<u8, 7>,
        signed: VarWithGivenBitSize<i8, 4>,
    }

    #[rstest]
    fn msg_simple_msg_serde() {
        let mut ser = Biseri::new();

        let m = MsgLalaBase { a1: 7345.into(), b1: true, b2: false, f: 12345.6789 };
        println!("m: {:?}", m);

        m.bit_serialize(&mut ser);
        let (bits, bytes) = ser.finish_add_data().unwrap();
        println!("bits: {}, bytes: {}", bits, bytes);

        // ***
        let mut der = Bides::from_biseri(&ser);

        let mm = MsgLalaBase::bit_deserialize(1, &mut der);
        assert!(mm.is_some());
        let mm = mm.unwrap();
        println!("mm: {:?}", mm);

        assert_eq!(m, mm.0);
    }

    #[rstest]
    fn msg_with_inner_msg_serde() {
        let mut ser = Biseri::new();

        let m = MsgLili {
            inner_msg: MsgLalaBase { a1: 7345.into(), b1: true, b2: false, f: 12345.6789 },
            b1: true,
            b2: 11.into(),
            signed: (-3).into()
        };
        println!("m: {:?}", m);

        m.bit_serialize(&mut ser);
        let (bits, bytes) = ser.finish_add_data().unwrap();
        println!("bits: {}, bytes: {}", bits, bytes);

        // ***
        let mut der = Bides::from_biseri(&ser);

        let mm = MsgLili::bit_deserialize(1, &mut der);
        assert!(mm.is_some());
        let mm = mm.unwrap();
        println!("mm: {:?}", mm);

        assert_eq!(m, mm.0);
    }

    #[derive(BiserdiOneOf, Debug, Clone, PartialEq)]
    #[biserdi_enum_id_dynbits(4)]
    enum OOLili {
        InnerMsg(MsgLalaBase),
        B1(bool),
        F2(f32),
        //#[biserdi_enum_id_bits(22)]
        Signed(VarWithGivenBitSize<i8, 6>),
    }

    fn oneof_test_serde(m: OOLili) {
        println!("m: {:?}", m);

        let mut ser = Biseri::new();

        m.bit_serialize(&mut ser);
        let (bits, bytes) = ser.finish_add_data().unwrap();
        println!("bits: {}, bytes: {}", bits, bytes);

        // ***
        let mut der = Bides::from_biseri(&ser);

        let mm = OOLili::bit_deserialize(1, &mut der);
        assert!(mm.is_some());
        let mm = mm.unwrap();
        println!("mm: {:?}", mm);

        assert_eq!(bits, mm.1);
        assert_eq!(m, mm.0);
    }
    #[rstest]
    fn oneof_msg_serde() {
        let m = OOLili::InnerMsg(MsgLalaBase { a1: 7345.into(), b1: true, b2: false, f: 12345.6789 });
        oneof_test_serde(m);
    }
    #[rstest]
    fn oneof_bool_serde() {
        let m = OOLili::B1(true);
        oneof_test_serde(m);
    }
    #[rstest]
    fn oneof_f32_serde() {
        let m = OOLili::F2(98765.54321);
        oneof_test_serde(m);
    }
    #[rstest]
    fn oneof_var_with_given_bit_size_serde() {
        let m = OOLili::Signed((-3).into());
        oneof_test_serde(m);
    }

    #[derive(BiserdiEnum, Debug, Clone, PartialEq)]
    #[biserdi_enum_id_dynbits(4)]
    #[allow(nonstandard_style)]
    enum EnumLele {
        One,
        Two,
        Three
    }
    #[derive(BiserdiMsg, Debug, Clone, PartialEq)]
    struct MsgLalaEnum {
        e1: EnumLele,
        b1: bool,
        b2: bool,
    }
    #[rstest]
    fn enum_msg_serde() {
        let mut ser = Biseri::new();

        let msg = MsgLalaEnum{
            e1: EnumLele::Two,
            b1: true,
            b2: false,
        };
        msg.bit_serialize(&mut ser);
        let (bits, bytes) = ser.finish_add_data().unwrap();
        println!("bits: {}, bytes: {}", bits, bytes);

        // ***
        let mut der = Bides::from_biseri(&ser);

        let mm = MsgLalaEnum::bit_deserialize(1, &mut der);
        assert!(mm.is_some());
        let mm = mm.unwrap();
        println!("mm: {:?}", mm);

        assert_eq!(bits, mm.1);
        assert_eq!(msg, mm.0);
    }

}

#[cfg(test)]
mod msg_deserialization_ver {
    use rstest::rstest;
    use super::*;
    pub use bitis_macros::{BiserdiMsgVersioned};

    #[derive(BiserdiMsg, Debug, Clone, PartialEq)]
    struct MsgLalaBase {
        a1: VarWithGivenBitSize<u16, 13>,
        b1: bool,
        b2: bool,
        f: f32,
    }

    // ***
    #[derive(BiserdiMsg, Debug, Clone, PartialEq)]
    struct MsgLalaV1 {}
    #[derive(BiserdiMsg, Debug, Clone, PartialEq)]
    struct MsgLalaExtV1 {
        v1: MsgLalaV1
    }
    #[derive(BiserdiMsgVersioned, Debug, Clone, PartialEq)]
    struct MsgLalaVersionedV1 {
        base: MsgLalaBase,
        ext: MsgLalaExtV1
    }
    #[derive(BiserdiMsg, Debug, Clone, PartialEq)]
    #[allow(nonstandard_style)]
    struct MsgLala_V2 {
        e1: VarWithGivenBitSize<i8, 4>,
        e2: bool,
    }
    #[derive(BiserdiMsg, Debug, Clone, PartialEq)]
    #[allow(nonstandard_style)]
    struct MsgLala_ExtV2 {
        v1: MsgLalaV1,
        v2: MsgLala_V2
    }
    #[derive(BiserdiMsgVersioned, Debug, Clone, PartialEq)]
    struct MsgLalaVersionedV2 {
        base: MsgLalaBase,
        ext: MsgLala_ExtV2
    }
    #[derive(BiserdiMsg, Debug, Clone, PartialEq)]
    struct MsgLalaV3 {}
    #[derive(BiserdiMsg, Debug, Clone, PartialEq)]
    #[allow(nonstandard_style)]
    struct MsgLala_ExtV3 {
        v1: MsgLalaV1,
        v2: MsgLala_V2,
        v3: MsgLalaV3
    }
    #[derive(BiserdiMsgVersioned, Debug, Clone, PartialEq)]
    struct MsgLalaVersionedV3 {
        base: MsgLalaBase,
        ext: MsgLala_ExtV3
    }
    #[derive(BiserdiMsg, Debug, Clone, PartialEq)]
    struct MsgLalaV4 {
        ee1: bool,
        ee2: bool,
        ee3: VarWithGivenBitSize<i16, 14>,
    }
    #[derive(BiserdiMsg, Debug, Clone, PartialEq)]
    struct MsgLalaExtV4 {
        v1: MsgLalaV1,
        v2: MsgLala_V2,
        v3: MsgLalaV3,
        v4: MsgLalaV4
    }
    #[derive(BiserdiMsgVersioned, Debug, Clone, PartialEq)]
    struct MsgLalaVersionedV4 {
        base: MsgLalaBase,
        ext: MsgLalaExtV4
    }
    #[rstest]
    // test backwards compability
    fn encode_v2_decode_v1() {
        let mut ser = Biseri::new();

        let m = MsgLalaVersionedV2 {
            base: MsgLalaBase { a1: 1234.into(), b1: true, b2: false, f: 9876.54321, },
            ext: MsgLala_ExtV2 { v1: MsgLalaV1 {}, v2: MsgLala_V2 { e1: (-6).into(), e2: true } }
        };
        m.bit_serialize(&mut ser);
        println!("m: {:?}", m);

        let (bits, bytes) = ser.finish_add_data().unwrap();
        println!("bits: {}, bytes: {}", bits, bytes);

        // ***
        let mut der = Bides::from_biseri(&ser.clone());

        let mm = MsgLalaVersionedV2::bit_deserialize(1, &mut der);
        println!("v2 - mm: {:?}", mm);
        assert!(mm.is_some());
        let mm = mm.unwrap();

        assert_eq!(bits, mm.1);
        assert_eq!(m, mm.0);

        // ***
        let mut der = Bides::from_biseri(&ser);

        let mm = MsgLalaVersionedV1::bit_deserialize(1, &mut der);
        println!("v1 - mm: {:?}", mm);
        assert!(mm.is_some());
        let mm = mm.unwrap();

        assert_eq!(bits, mm.1);
        assert_eq!(m.base, mm.0.base);
    }
    #[rstest]
    fn encode_v3_decode_v2_and_v1() {
        let mut ser = Biseri::new();

        let m = MsgLalaVersionedV4 {
            base: MsgLalaBase { a1: 1234.into(), b1: true, b2: false, f: 9876.54321, },
            ext: MsgLalaExtV4 {
                v1: MsgLalaV1 {},
                v2: MsgLala_V2 { e1: (-6).into(), e2: true },
                v3: MsgLalaV3 {},
                v4: MsgLalaV4 { ee1: false, ee2: true, ee3: (-5678).into(), }
            }
        };

        m.bit_serialize(&mut ser);
        println!("m: {:?}", m);

        let (bits, bytes) = ser.finish_add_data().unwrap();
        println!("bits: {}, bytes: {}", bits, bytes);

        // ***
        let mut der = Bides::from_biseri(&ser.clone());

        let mm = MsgLalaVersionedV4::bit_deserialize(1, &mut der);
        println!("v3 - mm: {:?}", mm);
        assert!(mm.is_some());
        let mm = mm.unwrap();

        assert_eq!(bits, mm.1);
        assert_eq!(m, mm.0);

        // ***
        let mut der = Bides::from_biseri(&ser.clone());

        let mm = MsgLalaVersionedV2::bit_deserialize(1, &mut der);
        println!("v2 - mm: {:?}", mm);
        assert!(mm.is_some());
        let mm = mm.unwrap();

        assert_eq!(bits, mm.1);
        assert_eq!(m.base, mm.0.base);
        assert_eq!(m.ext.v2, mm.0.ext.v2);

        // ***
        let mut der = Bides::from_biseri(&ser);

        let mm = MsgLalaVersionedV1::bit_deserialize(1, &mut der);
        println!("v1 - mm: {:?}", mm);
        assert!(mm.is_some());
        let mm = mm.unwrap();

        assert_eq!(bits, mm.1);
        assert_eq!(m.base, mm.0.base);
    }

    #[derive(BiserdiMsgVersioned, Debug, Clone, PartialEq)]
    enum MsgLalaVersionEnum {
        V1(MsgLalaVersionedV1),
        V2(MsgLalaVersionedV2),
        V3(MsgLalaVersionedV3),
        V4(MsgLalaVersionedV4),
    }
    #[rstest]
    fn encode_v3_decode_v2_and_v4() {
        let mut ser = Biseri::new();

        let mv = MsgLalaVersionedV3 {
            base: MsgLalaBase { a1: 1234.into(), b1: true, b2: false, f: 9876.54321, },
            ext: MsgLala_ExtV3 {
                v1: MsgLalaV1 {},
                v2: MsgLala_V2 { e1: (-6).into(), e2: true },
                v3: MsgLalaV3 {},
            }
        };
        let m = MsgLalaVersionEnum::V3(mv.clone());

        m.bit_serialize(&mut ser);
        println!("m: {:?}", m);

        let (bits, bytes) = ser.finish_add_data().unwrap();
        println!("bits: {}, bytes: {}", bits, bytes);

        // ***
        let mut der = Bides::from_biseri(&ser.clone());

        let mm = MsgLalaVersionEnum::bit_deserialize(2, &mut der);
        println!("v2 - mm: {:?}", mm);
        assert!(mm.is_some());
        let mm = mm.unwrap();

        let mmm = match mm.0.clone() {
            MsgLalaVersionEnum::V2(v) => v, _ => { panic!("Wrong enum, expected v2") }
        };


        assert_eq!(bits, mm.1);
        assert_eq!(mv.base, mmm.base);
        assert_eq!(mv.ext.v1, mmm.ext.v1);
        assert_eq!(mv.ext.v2, mmm.ext.v2);

        // ***
        let mut der = Bides::from_biseri(&ser.clone());

        let mm = MsgLalaVersionEnum::bit_deserialize(3, &mut der);
        println!("v4 - mm: {:?}", mm);
        assert!(mm.is_some());
        let mm = mm.unwrap();

        let mmm = match mm.0.clone() {
            MsgLalaVersionEnum::V3(v) => v, _ => { panic!("Wrong enum, expected v3") }
        };

        assert_eq!(bits, mm.1);
        assert_eq!(mv.base, mmm.base);
        assert_eq!(mv.ext.v2, mmm.ext.v2);
        assert_eq!(mv.ext.v3, mmm.ext.v3);
    }
}