// #![feature(trace_macros)]
//
// trace_macros!(true);
//

pub mod msg_test;
pub mod lib_impl;

pub use lib_impl::berde::*;
pub use lib_impl::compiler::*;
pub use bitis_macros::{BiserdiMsg, BiserdiOneOf, BiserdiEnum};
pub use std::result;


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



#[derive(Debug)]
pub enum MessageManagerError {
    InvalidMessage,
}
pub trait MessageWithHeaderTrait : Default {
    fn deserialize_header(&mut self, data: &mut Bides) -> Result<Option<(usize, usize)>, MessageManagerError>;
    fn deserialize_payload(&mut self, data: &mut Bides) -> Option<usize>;
}
#[derive(Debug, Clone)]
pub struct MessageManager<MWH> {
    pub bides: Bides,
    pub data_unused: Vec<u8>,
    pub header_successfully_read: bool,

    pub payload_size: usize,
    pub msg_with_header: MWH,
}
#[allow(unused)]
impl<MWH: MessageWithHeaderTrait + Default> MessageManager<MWH> {
    fn create() -> Self {
        Self{bides: Bides::new(), data_unused: Vec::new(), header_successfully_read: false,
            payload_size: 0, msg_with_header: Default::default(), }
    }

    fn append_data_and_try_deserialize(&mut self, data: &Vec<u8>) -> Result<Option<usize>, MessageManagerError> {
        self.bides.append_data(data);

        if !self.header_successfully_read {
            if self.bides.data_cache.len() == 0 { return Ok(None); }

            let (p, b) = match self.msg_with_header.deserialize_header(&mut self.bides)? {
                None => { return Ok(None); } Some(v) => { v }
            };
            self.payload_size = p;
            self.header_successfully_read = true;
            self.bides.data_cache.drain(..b);
            self.bides.reset_position();
        }
        if self.header_successfully_read {
            if self.bides.data_cache.len() < self.payload_size { return Ok(None); }
            if self.bides.data_cache.len() > self.payload_size {
                self.data_unused.extend(self.bides.data_cache[self.payload_size..].to_vec());
                self.bides.data_cache.truncate(self.payload_size);
            }

            match self.msg_with_header.deserialize_payload(&mut self.bides) {
                Some(s) => Ok(Some(s)), None => Ok(None) 
            }
        }
        else { Ok(None) }
    }
}


#[cfg(test)]
mod msg_header {
    use rstest::rstest;
    use super::*;

    #[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
    struct MsgHeaderByteTwo {
        size_high: VarWithGivenBitSize<u8, 6>,
        must_be_one: VarWithGivenBitSize<u8, 2>,
    }
    #[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
    struct MsgHeader {
        size_low: VarWithGivenBitSize<u8, 6>,
        must_be_one: VarWithGivenBitSize<u8, 1>,
        byte_two: Option<MsgHeaderByteTwo>
    }
    #[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
    struct MsgLalaBase {
        a1: VarWithGivenBitSize<u16, 13>,
        b1: bool,
        b2: bool,
        f: f32,
    }

    #[derive(Debug, Clone, PartialEq, Default)]
    struct MsgWH{
        header: MsgHeader,
        payload: MsgLalaBase,
    }
    impl MessageWithHeaderTrait for MsgWH {
        fn deserialize_header(&mut self, bides: &mut Bides) -> Result<Option<(usize, usize)>, MessageManagerError> {
            let bit_size = match MsgHeader::bit_deserialize(0, bides) {
                None => {
                    bides.reset_position();
                    return Ok(None)
                }
                Some((h, r)) => {
                    self.header = h;
                    r
                }
            };
            if self.header.must_be_one.val != 1 { return Err(MessageManagerError::InvalidMessage) }
            if let Some(v) = self.header.byte_two.clone() {
                if v.must_be_one.val != 1 { return Err(MessageManagerError::InvalidMessage) }
            }
            Ok(Some(
                (
                    (self.header.size_low.val as usize) +
                        if let Some(bt) = self.header.byte_two.clone()
                        { (bt.size_high.val as usize) << 6 } else { 0 },
                    (bit_size >> 3) as usize
                )
            ))
        }

        fn deserialize_payload(&mut self, bides: &mut Bides) -> Option<usize> {
            match MsgLalaBase::bit_deserialize(0, bides) {
                None => { bides.reset_position(); None }
                Some((p, s)) => {
                    self.payload = p;
                    Some(s as usize)
                }
            }
        }
    }
    #[test]
    fn msg_manager_test() {
        let payload = MsgLalaBase{ a1: 1234.into(), b1: false, b2: true, f: 12.34 };
        let mut s1 = Biseri::new();
        payload.bit_serialize(&mut s1);
        s1.finish_add_data().unwrap();
        let s = s1.get_data().len();

        let header = MsgHeader{
            size_low: ((s & ((1<<6)-1)) as u8).into(),
            must_be_one: 1.into(),
            byte_two: if (s>>6) > 0 {
                Some(MsgHeaderByteTwo{size_high: ((s>>6) as u8).into(), must_be_one: 1.into() }) }
            else { None }
        };
        let mut s2 = Biseri::new();
        header.bit_serialize(&mut s2);
        s2.finish_add_data().unwrap();
        let mut data = s2.get_data();
        data.extend(s1.get_data());

        let mut mm = MessageManager::<MsgWH>::create();
        let r = mm.append_data_and_try_deserialize(&vec![data[0]]);
        assert!(r.is_ok());
        let r = r.unwrap();
        assert!(r.is_none());

        let r = mm.append_data_and_try_deserialize(&data[1..].to_vec());
        assert!(r.is_ok());
        let r = r.unwrap();
        assert!(r.is_some());
        
        println!("{:#?}", mm);
        assert_eq!(mm.msg_with_header.header, header);
        assert_eq!(mm.msg_with_header.payload, payload);
    }
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