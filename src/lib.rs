pub mod msg_test;
pub mod lib_impl;

pub use lib_impl::berde::*;
pub use lib_impl::compiler::*;


#[cfg(test)]
mod msg_deserialization {
    use rstest::rstest;
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct MsgLala {
        a1: u16,
        b1: bool,
        b2: bool,
        f: f32,
    }
    impl BiserdiTrait for MsgLala {
        fn bit_serialize(self: &Self, biseri: &mut Biseri) -> Option<()> {
            self.a1.bit_serialize(13, biseri)?;
            self.b1.bit_serialize(biseri)?;
            self.b2.bit_serialize(biseri)?;
            self.f.bit_serialize(biseri)?;
            Some(())
        }
        fn bit_deserialize(bides: &mut Bides) -> Option<Self> {
            Some(Self{
                a1: u16::bit_deserialize(13, bides)?,
                b1: bool::bit_deserialize(bides)?,
                b2: bool::bit_deserialize(bides)?,
                f: f32::bit_deserialize(bides)?
            })
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    struct MsgLili {
        inner_msg: MsgLala,
        b1: bool,
        b2: bool,
        signed: i8,
    }
    impl BiserdiTrait for MsgLili {
        fn bit_serialize(self: &Self, biseri: &mut Biseri) -> Option<()> {
            self.inner_msg.bit_serialize(biseri)?;
            self.b1.bit_serialize(biseri)?;
            self.b2.bit_serialize(biseri)?;
            self.signed.bit_serialize(4, biseri)?;
            Some(())
        }
        fn bit_deserialize(bides: &mut Bides) -> Option<Self> {
            Some(Self{
                inner_msg: MsgLala::bit_deserialize(bides)?,
                b1: bool::bit_deserialize(bides)?,
                b2: bool::bit_deserialize(bides)?,
                signed: i8::bit_deserialize(4, bides)?
            })
        }
    }


    #[rstest]
    fn msg_simple_msg_serde() {
        let mut ser = Biseri::new();

        let m = MsgLala{a1: 7345, b1: true, b2: false, f: 12345.6789};
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

        assert_eq!(m, mm);
    }
    #[rstest]
    fn msg_with_inner_msg_serde() {
        let mut ser = Biseri::new();

        let m = MsgLili{
            inner_msg: MsgLala{a1: 7345, b1: true, b2: false, f: 12345.6789},
            b1: true, b2: false, signed: -3
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

        assert_eq!(m, mm);
    }

}
