
#[derive(Debug, Clone)]
pub struct Biseri {
    cur_cache_u8: u16,
    sub_byte_counter: u8,
    data_cache: Vec<u8>,
}

fn bits_for_and(x: u8) -> u16 {
    u16::MAX >> (u16::BITS as u8 - x)
}

#[allow(dead_code)]
trait BiserdiTrait : Sized {
    fn get_expected_size() -> u32;
    fn bit_serialize(self: &Self, total_bits: u64, biseri: &mut Biseri) -> bool;
    fn bit_deserialize(total_bits: u64, bides: &mut Bides) -> Option<Self>;
}

#[allow(dead_code)]
impl Biseri {
    pub fn new() -> Biseri {
        Biseri { cur_cache_u8: 0, data_cache: Vec::new(), sub_byte_counter: 0 }
    }

    pub fn data_size_bytes(&self) -> u64 {
        self.data_cache.len() as u64
    }
    pub fn get_data(&self) -> &Vec<u8> {
        &self.data_cache
    }

    fn add_data_base_u8(&mut self, cur_u8: &u8, total_bits: u64) -> u64 {
        if total_bits > 0 {
            let cur_u16 = cur_u8.clone() as u16;
            let shift_by = self.sub_byte_counter & 7;

            let cur_bit_size = std::cmp::min(8, total_bits as u8);
            // println!("cur_bit_size: {}", cur_bit_size);
            let cur_u16 = (cur_u16 & (bits_for_and(cur_bit_size))) << shift_by;
            self.cur_cache_u8 += cur_u16;

            self.sub_byte_counter += cur_bit_size;
            let total_bits = total_bits - cur_bit_size as u64;

            if self.sub_byte_counter >= 8 {
                self.sub_byte_counter -= 8;
                let u8_to_add = (self.cur_cache_u8 & 0xFF) as u8;
                self.data_cache.push(u8_to_add);
                self.cur_cache_u8 >>= 8;
            }
            total_bits
        }
        else { 0 }
    }

    pub fn add_data(&mut self, cur_data: Vec<u8>, total_bits: u64) -> bool {
        if (cur_data.len() as u64)*8  < total_bits {
            return false;
        }
        let mut cur_total_bits = total_bits;
        for cu8 in cur_data.iter() {
            cur_total_bits = self.add_data_base_u8(cu8, cur_total_bits);
        }
        true
    }

    pub fn finish_add_data(&mut self) -> (u64, u64) {
        let total_bits = ((self.data_cache.len() as u64) << 3) + self.sub_byte_counter as u64;
        if self.sub_byte_counter > 0 {
            let u8_to_add = (self.cur_cache_u8 & 0xFF) as u8;
            self.data_cache.push(u8_to_add);
        }

        (total_bits, self.data_cache.len() as u64)
    }

    /*pub fn add_data_type<T: ToBytes>(&mut self, val: &T, total_bits: u64) -> bool {
        self.add_data(val.clone().to_le_bytes().to_vec(), total_bits)
    }*/

    // pub fn add_data_bool(&mut self, val: &bool) -> bool {
    //     let val = if *val { 1_u8 } else { 0_u8 };
    //     self.add_data_base_u8(&val, 1);
    //     true
    // }
    // pub fn add_data_u8(&mut self, val: &u8, total_bits: u64) -> bool {
    //     self.add_data(val.clone().to_le_bytes().to_vec(), total_bits)
    // }
    // pub fn add_data_u16(&mut self, val: &u16, total_bits: u64) -> bool {
    //     self.add_data(val.clone().to_le_bytes().to_vec(), total_bits)
    // }
    // pub fn add_data_u32(&mut self, val: &u32, total_bits: u64) -> bool {
    //     self.add_data(val.clone().to_le_bytes().to_vec(), total_bits)
    // }
    // pub fn add_data_u64(&mut self, val: &u64, total_bits: u64) -> bool {
    //     self.add_data(val.clone().to_le_bytes().to_vec(), total_bits)
    // }
    // pub fn add_data_i8(&mut self, val: &i8, total_bits: u64) -> bool {
    //     self.add_data(val.clone().to_le_bytes().to_vec(), total_bits)
    // }
    // pub fn add_data_i16(&mut self, val: &i16, total_bits: u64) -> bool {
    //     self.add_data(val.clone().to_le_bytes().to_vec(), total_bits)
    // }
    // pub fn add_data_i32(&mut self, val: &i32, total_bits: u64) -> bool {
    //     self.add_data(val.clone().to_le_bytes().to_vec(), total_bits)
    // }
    // pub fn add_data_i64(&mut self, val: &i64, total_bits: u64) -> bool {
    //     self.add_data(val.clone().to_le_bytes().to_vec(), total_bits)
    // }
    // pub fn add_data_f32(&mut self, val: &f32) -> bool {
    //     self.add_data(val.clone().to_le_bytes().to_vec(), 32)
    // }
    // pub fn add_data_f64(&mut self, val: &f64) -> bool {
    //     self.add_data(val.clone().to_le_bytes().to_vec(), 64)
    // }
}

#[derive(Debug, Clone)]
pub struct Bides {
    cur_read_pos: u64,
    sub_byte_counter: u8,
    data_cache: Vec<u8>,
}
#[allow(dead_code)]
impl Bides {
    pub fn from_vec(data: &Vec<u8>) -> Bides {
        Bides{sub_byte_counter: 0, data_cache: data.clone(), cur_read_pos: 0}
    }
    pub fn from_biseri(biseri: &Biseri) -> Bides {
        Bides{sub_byte_counter: 0, data_cache: biseri.get_data().clone(), cur_read_pos: 0}
    }

    pub fn decode_data_base_u8(&mut self, total_bits: u64) -> Option<(u8, u64)> {
        if self.cur_read_pos as usize >= self.data_cache.len() { return None}
        let mut cur_u16: u16 = self.data_cache[self.cur_read_pos as usize] as u16;
        if (self.cur_read_pos + 1 < self.data_cache.len() as u64) && (self.sub_byte_counter > 0) {
            cur_u16 += (self.data_cache[(self.cur_read_pos + 1) as usize] as u16) << 8;
        }

        let cur_used_bits = std::cmp::min(total_bits as u8, 8);

        // println!("d cur_used_bits={} (total_bits={})", cur_used_bits, total_bits);

        let d = ((cur_u16 >> self.sub_byte_counter) & bits_for_and(cur_used_bits)) as u8;

        self.sub_byte_counter += cur_used_bits;
        if self.sub_byte_counter >= 8 {
            self.sub_byte_counter -= 8;
            self.cur_read_pos += 1;
        }

        Some((d, total_bits - cur_used_bits as u64))
    }

    pub fn decode_data(&mut self, total_bits: u64, expected_bytes: u32) -> Option<Vec<u8>> {
        if self.cur_read_pos >= self.data_cache.len() as u64 {
            return None;
        }
        let mut cur_total_bits = total_bits;
        let mut dv = Vec::new();
        while cur_total_bits > 0 {
            let d;
            (d, cur_total_bits) = match self.decode_data_base_u8(cur_total_bits) {
                Some(d) => d, None => { return None; }
            };
            dv.push(d);
        }
        let num_add = expected_bytes - dv.len() as u32;
        for _ in 0..num_add { dv.push(0); }
        Some(dv)
    }

    // pub fn decode_data_bool(&mut self) -> Option<bool> {
    //     let vec = self.decode_data(1)?;
    //     Some(if vec[0] == 0 { false } else { true })
    // }
    // // pub fn decode_data_u8(&mut self, total_bits: u64) -> Option<u8> {
    //     Some(u8::from_le_bytes(self.decode_data(total_bits)?.try_into().ok()?))
    // }
    // pub fn decode_data_u16(&mut self, total_bits: u64) -> Option<u16> {
    //     Some(u16::from_le_bytes(self.decode_data(total_bits)?.try_into().ok()?))
    // }
    // pub fn decode_data_u32(&mut self, total_bits: u64) -> Option<u32> {
    //     Some(u32::from_le_bytes(self.decode_data(total_bits)?.try_into().ok()?))
    // }
    // pub fn decode_data_u64(&mut self, total_bits: u64) -> Option<u64> {
    //     Some(u64::from_le_bytes(self.decode_data(total_bits)?.try_into().ok()?))
    // }
    // pub fn decode_data_i8(&mut self, total_bits: u64) -> Option<i8> {
    //     Some(i8::from_le_bytes(self.decode_data(total_bits)?.try_into().ok()?))
    // }
    // pub fn decode_data_i16(&mut self, total_bits: u64) -> Option<i16> {
    //     Some(i16::from_le_bytes(self.decode_data(total_bits)?.try_into().ok()?))
    // }
    // pub fn decode_data_i32(&mut self, total_bits: u64) -> Option<i32> {
    //     Some(i32::from_le_bytes(self.decode_data(total_bits)?.try_into().ok()?))
    // }
    // pub fn decode_data_i64(&mut self, total_bits: u64) -> Option<i64> {
    //     Some(i64::from_le_bytes(self.decode_data(total_bits)?.try_into().ok()?))
    // }
    // pub fn decode_data_f32(&mut self) -> Option<f32> {
    //     Some(f32::from_le_bytes(self.decode_data(32)?.try_into().ok()?))
    // }
    // pub fn decode_data_f64(&mut self) -> Option<f64> {
    //     Some(f64::from_le_bytes(self.decode_data(64)?.try_into().ok()?))
    // }
}

macro_rules! impl_biserdi_trait {
    ($type:ty, $num_bytes: expr) => {
        impl BiserdiTrait for $type {
            fn bit_serialize(self: &Self, total_bits: u64, biseri: &mut Biseri) -> bool {
                biseri.add_data(self.clone().to_le_bytes().to_vec(), total_bits) }
            fn bit_deserialize(total_bits: u64, bides: &mut Bides) -> Option<Self> {
                Some(Self::from_le_bytes(bides.decode_data(total_bits, Self::get_expected_size())?.try_into().ok()?)) }
            fn get_expected_size() -> u32 { return $num_bytes; }
        }
    };
}


/*impl BiserdiTrait for u8 {
    fn bit_serialize(val: &Self, total_bits: u64, biseri: &mut Biseri) -> bool {
        biseri.add_data(val.clone().to_le_bytes().to_vec(), total_bits)
    }
    fn bit_deserialize(total_bits: u64, bides: &mut Bides) -> Option<Self> {
        Some(Self::from_le_bytes(bides.decode_data(total_bits)?.try_into().ok()?))
    }
}*/
impl BiserdiTrait for bool {
    fn get_expected_size() -> u32 { 1 }
    fn bit_serialize(self: &Self, total_bits: u64, biseri: &mut Biseri) -> bool {
        if total_bits != 1 { return false; }
        let val = if *self { 1_u8 } else { 0_u8 };
        biseri.add_data_base_u8(&val, 1);
        true
    }
    fn bit_deserialize(total_bits: u64, bides: &mut Bides) -> Option<Self> {
        if total_bits != 1 { return None; }
        let vec = bides.decode_data(1, Self::get_expected_size())?;
        Some(if vec[0] == 0 { false } else { true })
    }
}
impl_biserdi_trait!(u8, u8::BITS>>3);
impl_biserdi_trait!(u16, u16::BITS>>3);
impl_biserdi_trait!(u32, u32::BITS>>3);
impl_biserdi_trait!(u64, u64::BITS>>3);
impl_biserdi_trait!(i8, i8::BITS>>3);
impl_biserdi_trait!(i16, i16::BITS>>3);
impl_biserdi_trait!(i32, i32::BITS>>3);
impl_biserdi_trait!(i64, i64::BITS>>3);
impl_biserdi_trait!(f32, 32>>3);
impl_biserdi_trait!(f64, 64>>3);

#[cfg(test)]
mod bitis_base_serialization_deserialization {
    use rstest::rstest;
    use crate::lib_berde::{Bides, Biseri};
    use super::*;

    #[rstest]
    fn add_one_u8() {
        let mut b = Biseri::new();
        let d = 0b10101010_u8;

        b.add_data_base_u8(&d, 8);

        assert_eq!(b.clone().data_cache.len(), 1);
        assert_eq!(b.sub_byte_counter, 0);

        let r = b.finish_add_data();
        assert_eq!(r, (8, 1));
    }

    #[rstest]
    #[case::ot_uint4(4, 1, 4, 1)]
    #[case::tt_uint4(4, 2, 8, 1)]
    #[case::ot_uint6(6, 1, 6, 1)]
    #[case::tt_uint6(6, 2, 12, 2)]
    #[case::trt_uint6(6, 3, 18, 3)]
    #[case::ft_uint6(6, 4, 24, 3)]
    fn add_one_var(#[case] bitsize: u64, #[case] repeated: u8, #[case] final_bitsize: u64, #[case] num_bytes: u64) {
        let mut b = Biseri::new();
        let d = 0b10101010_u8;

        for _i in 0..repeated {
            b.add_data_base_u8(&d, bitsize);
        }

        let r = b.finish_add_data();
        assert_eq!(r, (final_bitsize, num_bytes));
    }

    #[rstest]
    #[case::ok(5, 4, 5)]
    #[case::expected_overflow(0b10011, 4, 3)]
    fn serialize_and_deserialize_base(#[case] val_in: u8, #[case] bitsize: u64, #[case] val_out: u8) {
        let mut ser = Biseri::new();

        ser.add_data_base_u8(&val_in, bitsize);
        ser.finish_add_data();

        let mut des = Bides::from_biseri(&ser);

        assert_eq!(des.data_cache, ser.data_cache);

        let r = des.decode_data_base_u8(bitsize);
        assert!(r.is_some());
        let (dd, bs) = r.unwrap();
        assert_eq!(bs, 0);

        println!("val_in: {val_in} vs. dd: {dd} (expected: {val_out}");
        assert_eq!(val_out, dd);
    }

    #[rstest]
    #[case::ok(3*256+5, 16, 3*256+5)]
    #[case::ok(3*256+5, 12, 3*256+5)]
    #[case::ok(3*256+5, 9, 1*256+5)]
    fn serialize_and_deserialize_u16_single(#[case] val_in: u16, #[case] bitsize: u64, #[case] val_out: u16) {
        let val_in_vec = val_in.to_le_bytes().clone();

        let mut ser = Biseri::new();

        let mut total_size = bitsize;
        val_in_vec.clone().iter().for_each(|x| {
            total_size = ser.add_data_base_u8(&x, bitsize);
        });
        ser.finish_add_data();

        println!("ser.cache: {:?}", ser.data_cache);

        assert_eq!(ser.data_cache.len(), 2);

        let mut des = Bides::from_biseri(&ser);

        assert_eq!(des.data_cache, ser.data_cache);

        let mut dd = Vec::new();
        let mut total_size = bitsize;
        while total_size > 0 {
            let ddd;
            let r = des.decode_data_base_u8(total_size);
            assert!(r.is_some());
            (ddd, total_size) = r.unwrap();
            dd.push(ddd);
        };

        let ddv = u16::from_le_bytes(dd.clone().try_into().unwrap());
        println!("val_in: {val_in} ({val_in_vec:?}) vs. ddv: {ddv:?} ({dd:?}) (expected: {val_out})");
        assert_eq!(val_out, ddv);
    }

    fn add_two_u16_fixed(ser: &mut Biseri, bits: u64) -> (u64, u64) {
        let d: u16 = 3;

        ser.add_data(d.clone().to_le_bytes().to_vec(), bits);
        ser.add_data(d.clone().to_le_bytes().to_vec(), bits);
        ser.finish_add_data()
    }
    #[rstest]
    fn serialize_u16_fixed_full() {
        let mut ser = Biseri::new();

        let (lbits, lbytes) = add_two_u16_fixed(&mut ser, 16);

        assert_eq!(ser.data_cache.len(), 4);
        assert_eq!(lbytes, 4);
        assert_eq!(lbits, 2 * 16);

        assert_eq!(ser.data_cache[0], 3);
        assert_eq!(ser.data_cache[1], 0);
        assert_eq!(ser.data_cache[2], 3);
        assert_eq!(ser.data_cache[3], 0);
    }

    #[rstest]
    fn serialize_u16_fixed_12b() {
        let mut ser = Biseri::new();

        let (lbits, lbytes) = add_two_u16_fixed(&mut ser, 12);

        assert_eq!(ser.data_cache.len(), 3);
        assert_eq!(lbytes, 3);
        assert_eq!(lbits, 2 * 12);

        assert_eq!(ser.data_cache[0], 3);
        assert_eq!(ser.data_cache[1], 3 << 4);
        assert_eq!(ser.data_cache[2], 0);
    }

    #[rstest]
    #[case::bitsize_16(16)]
    #[case::bitsize_14(14)]
    #[case::bitsize_12(12)]
    fn ser_and_deserialize_u16_fixed(#[case] bits: u64) {
        let mut ser = Biseri::new();

        let (_lbits, _lbytes) = add_two_u16_fixed(&mut ser, bits);

        let mut des = Bides::from_biseri(&ser);

        assert_eq!(des.data_cache, ser.data_cache);

        let d1 = des.decode_data(bits, 1);
        assert!(d1.is_some());
        let d2 = des.decode_data(bits, 1);
        assert!(d2.is_some());

        let d1 = d1.unwrap();
        let d2 = d2.unwrap();

        assert_eq!(d1[0], 3);
        assert_eq!(d1[1], 0);
        assert_eq!(d2[0], 3);
        assert_eq!(d2[1], 0);
    }

    #[rstest]
    fn de_and_serialize_various_unsigned() {
        // ***********************************************************
        let mut ser = Biseri::new();
        let v1: u8 = 5;
        v1.bit_serialize(6, &mut ser);
        // ser.add_data_u8(&v1, 6);
        let v2: u16 = 15;
        v2.bit_serialize(14, &mut ser);
        // ser.add_data_u16(&v2, 14);
        let v3: u32 = 55;
        v3.bit_serialize(22, &mut ser);
        // ser.add_data_u32(&v3, 30);
        let (bits, bytes) = ser.finish_add_data();
        println!("bits: {}, bytes: {}", bits, bytes);

        // // ***********************************************************
        let mut des = Bides::from_biseri(&ser);
        let vo1 = u8::bit_deserialize(6, &mut des);
        // let vo1 = des.decode_data_u8(6);
        let vo2 = u16::bit_deserialize(14, &mut des);
        // let vo2 = des.decode_data_u16(14);
        let vo3 = u32::bit_deserialize(22, &mut des);
        // let vo3 = des.decode_data_u32(30);

        println!("v1: {}, v2: {}, v3: {} vs vo1: {:?}, vo2: {:?}, vo3: {:?}", v1, v2, v3, vo1, vo2, vo3);

        // ***********************************************************
        assert!(vo1.is_some());
        assert_eq!(v1, vo1.unwrap());

        assert!(vo2.is_some());
        assert_eq!(v2, vo2.unwrap());

        assert!(vo3.is_some());
        assert_eq!(v3, vo3.unwrap());
    }

    #[rstest]
    fn de_and_serialize_various_float() {
        // // ***********************************************************
        // let mut ser = Biseri::new();
        // let v1 = 56.78;
        // ser.add_data_f32(&v1);
        // let v2 = 5;
        // ser.add_data_u8(&v2, 5);
        // let v3 = true;
        // ser.add_data_bool(&v3);
        // let v4 = false;
        // ser.add_data_bool(&v4);
        // ser.add_data_f32(&v1);
        // let (bits, bytes) = ser.finish_add_data();
        //
        // println!("bits: {}, bytes: {}", bits, bytes);
        //
        // // ***********************************************************
        // let mut des = Bides::from_biseri(&ser);
        // let vo1 = des.decode_data_f32();
        // let vo2 = des.decode_data_u8(5);
        // let vo3 = des.decode_data_bool();
        // let vo4 = des.decode_data_bool();
        // let vo5 = des.decode_data_f32();
        //
        // println!("vo1: {:?}, vo2: {:?}, vo3: {:?}, vo4: {:?}, vo4: {:?}", vo1, vo2, vo3, vo4, vo5);
        //
        // // ***********************************************************
        // assert!(vo1.is_some());
        // assert_eq!(v1, vo1.unwrap());
        //
        // assert!(vo2.is_some());
        // assert_eq!(v2, vo2.unwrap());
        //
        // assert!(vo3.is_some());
        // assert_eq!(v3, vo3.unwrap());
        //
        // assert!(vo4.is_some());
        // assert_eq!(v4, vo4.unwrap());
        //
        // assert!(vo5.is_some());
        // assert_eq!(v1, vo5.unwrap());
    }
}

