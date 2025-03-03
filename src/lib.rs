// #![feature(trace_macros)]
//
// trace_macros!(true);
//

pub mod msg_test;
pub mod lib_impl;

pub use lib_impl::berde::*;
pub use lib_impl::compiler::*;

use bitis_macros::{BiserdiMsg, BiserdiOneOf};



#[cfg(test)]
mod msg_deserialization {
    use rstest::rstest;
    use super::*;

    #[derive(BiserdiMsg, Debug, Clone, PartialEq)]
    struct MsgLala {
        a1: VarWithGivenBitSize<u16, 13>,
        b1: bool,
        b2: bool,
        f: f32,
    }

    #[derive(BiserdiMsg, Debug, Clone, PartialEq)]
    struct MsgLili {
        inner_msg: MsgLala,
        b1: bool,
        b2: bool,
        signed: VarWithGivenBitSize<i8, 4>,
    }


    #[rstest]
    fn msg_simple_msg_serde() {
        let mut ser = Biseri::new();

        let m = MsgLala{a1: 7345.into(), b1: true, b2: false, f: 12345.6789};
        println!("m: {:?}", m);

        m.bit_serialize(&mut ser);
        let (bits, bytes) = ser.finish_add_data();
        println!("bits: {}, bytes: {}", bits, bytes);

        // ***
        let mut der = Bides::from_biseri(&ser);

        let mm = MsgLala::bit_deserialize(&mut der);
        assert!(mm.is_some());
        let mm = mm.unwrap();
        println!("mm: {:?}", mm);

        assert_eq!(m, mm.0);
    }

    #[rstest]
    fn msg_with_inner_msg_serde() {
        let mut ser = Biseri::new();

        let m = MsgLili{
            inner_msg: MsgLala{a1: 7345.into(), b1: true, b2: false, f: 12345.6789},
            b1: true, b2: false, signed: (-3).into()
        };
        println!("m: {:?}", m);

        m.bit_serialize(&mut ser);
        let (bits, bytes) = ser.finish_add_data();
        println!("bits: {}, bytes: {}", bits, bytes);

        // ***
        let mut der = Bides::from_biseri(&ser);

        let mm = MsgLili::bit_deserialize(&mut der);
        assert!(mm.is_some());
        let mm = mm.unwrap();
        println!("mm: {:?}", mm);

        assert_eq!(m, mm.0);
    }

    #[derive(BiserdiOneOf, Debug, Clone, PartialEq)]
    #[biserdi_enum_id_bits(4)]
    enum OOLili {
        InnerMsg(MsgLala),
        B1(bool),
        F2(f32),
        #[biserdi_enum_id_bits(22)]
        Signed(VarWithGivenBitSize<i8, 6>),
    }

    fn oneof_test_serde(m: OOLili) {
        println!("m: {:?}", m);

        let mut ser = Biseri::new();

        m.bit_serialize(&mut ser);
        let (bits, bytes) = ser.finish_add_data();
        println!("bits: {}, bytes: {}", bits, bytes);

        // ***
        let mut der = Bides::from_biseri(&ser);

        let mm = OOLili::bit_deserialize(&mut der);
        assert!(mm.is_some());
        let mm = mm.unwrap();
        println!("mm: {:?}", mm);

        assert_eq!(bits, mm.1);assert_eq!(m, mm.0);
    }
    #[rstest]
    fn oneof_msg_serde() {
        let m = OOLili::InnerMsg(MsgLala{a1: 7345.into(), b1: true, b2: false, f: 12345.6789});
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

}
