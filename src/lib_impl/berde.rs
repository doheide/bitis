use std::ops::{AddAssign, Neg, Shl, Shr};
use std::convert::{TryFrom, From, TryInto};
use std::fmt::{Debug, Display};
use array_init::map_array_init;
use ascii::{AsAsciiStr, AsciiString};

// ***
pub trait IntegerBaseFunctions {
    const IS_SIGNED: bool;
    fn get_absolute_uint(self: &Self) -> u64;
    fn is_val_negative(self: &Self) -> bool;
    fn switch_sign_if_possible(self: &Self) -> Self;
    fn get_zero() -> Self;
    fn get_bits_num() -> u8;
}
macro_rules! impl_integer_signed {
    ($type:ty, $num_bits: expr) => {
        impl IntegerBaseFunctions for $type {
            const IS_SIGNED: bool = true;
            fn get_absolute_uint(self: &Self) -> u64 {
                if *self < 0 { self.clone().neg() as u64 }
                else { self.clone() as u64 }
            }
            fn is_val_negative(self: &Self) -> bool { *self < 0 }
            fn switch_sign_if_possible(self: &Self) -> Self { return self.clone().neg() }
            fn get_zero() -> Self { 0 as $type }
            fn get_bits_num() -> u8 { $num_bits as u8 }
        }
} }
macro_rules! impl_integer_unsigned {
    ($type:ty, $num_bits: expr) => {
        impl IntegerBaseFunctions for $type {
            const IS_SIGNED: bool = false;
            fn get_absolute_uint(self: &Self) -> u64 { self.clone() as u64 }
            fn is_val_negative(self: &Self) -> bool { false }
            fn switch_sign_if_possible(self: &Self) -> Self { return self.clone() }
            fn get_zero() -> Self { 0 as $type }
            fn get_bits_num() -> u8 { $num_bits as u8 }
        }
} }
impl_integer_signed!(i8, 8);
impl_integer_signed!(i16, 16);
impl_integer_signed!(i32, 32);
impl_integer_signed!(i64, 64);
impl_integer_unsigned!(u8, 8);
impl_integer_unsigned!(u16, 16);
impl_integer_unsigned!(u32, 16);
impl_integer_unsigned!(u64, 16);

// ***
pub trait ValFromInto<T> {
    fn val_into(self: &Self) -> T;
    fn val_from(val: &T) -> Self;
}

// ***
#[derive(Debug, Clone)]
pub struct Biseri {
    cur_cache_u8: u16,
    sub_byte_counter: u8,
    data_cache: Vec<u8>,
    final_total_bits: u64,
}

fn bits_for_and(x: u8) -> u16 {
    u16::MAX >> (u16::BITS as u8 - x)
}

#[allow(dead_code)]
pub trait BiserdiTraitVarBitSize : Sized {
    fn bit_serialize(self: &Self, total_bits: u64, biseri: &mut Biseri) -> Option<u64>;
    fn bit_deserialize(version_id: u16, total_bits: u64, bides: &mut Bides) -> Option<(Self, u64)>;
}
#[allow(dead_code)]
pub trait BiserdiTrait: Sized {
    fn bit_serialize(self: &Self, biseri: &mut Biseri) -> Option<u64>;
    fn bit_deserialize(version_id: u16, bides: &mut Bides) -> Option<(Self, u64)>;
    fn min_bits() -> u64;
}

#[derive(Debug, Clone)]
pub struct BiserSizes {
    pub total_bits: u64,
    pub total_bytes: u64
}
#[allow(dead_code)]
impl Biseri {
    pub fn new() -> Biseri {
        Biseri { cur_cache_u8: 0, data_cache: Vec::new(), sub_byte_counter: 0, final_total_bits: 0 }
    }

    pub fn data_size_bytes(&self) -> u64 {
        self.data_cache.len() as u64
    }
    pub fn get_data_ref(&self) -> &Vec<u8> {
        &self.data_cache
    }
    pub fn get_data(&self) -> Vec<u8> {
        self.data_cache.clone()
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

    pub fn add_data(&mut self, cur_data: &Vec<u8>, total_bits: u64) -> Option<u64> {
        // if (cur_data.len() as u64)*8 > total_bits {
        //     // return None;
        //     println!("")
        // }
        // if (cur_data.len() as u64)*8  < total_bits {
        //     return None;
        // }
        let mut cur_total_bits = total_bits;
        for cu8 in cur_data.iter() {
            cur_total_bits = self.add_data_base_u8(cu8, cur_total_bits);
        }
        Some(total_bits)
    }

    pub fn add_biseri_data(&mut self, data: &Biseri) -> Option<u64> {
        self.add_data(&data.data_cache, data.final_total_bits)
        // self.add_data(&data.data_cache, ((data.data_cache.len() << 3) +
        //     data.sub_byte_counter as usize) as u64)
    }
    pub fn finish_add_data(&mut self) -> Option<BiserSizes> {
        if self.final_total_bits > 0 {
            None
        }
        else {
            let total_bits = ((self.data_cache.len() as u64) << 3) + self.sub_byte_counter as u64;
            if self.sub_byte_counter > 0 {
                let u8_to_add = (self.cur_cache_u8 & 0xFF) as u8;
                self.data_cache.push(u8_to_add);
            }

            self.final_total_bits = total_bits;
            Some(BiserSizes{total_bits, total_bytes: self.data_cache.len() as u64})
        }
    }
}

#[derive(Debug, Clone)]
pub struct Bides {
    cur_read_pos: u64,
    sub_byte_counter: u8,
    pub data_cache: Vec<u8>,
}
#[allow(dead_code)]
impl Bides {
    pub fn new() -> Bides { Bides { cur_read_pos: 0, sub_byte_counter: 0, data_cache: Vec::new() } }
    pub fn from_vec(data: &Vec<u8>) -> Bides {
        Bides{sub_byte_counter: 0, data_cache: data.clone(), cur_read_pos: 0}
    }
    pub fn from_biseri(biseri: &Biseri) -> Bides {
        Bides{sub_byte_counter: 0, data_cache: biseri.get_data().clone(), cur_read_pos: 0}
    }

    pub fn append_data(&mut self, data: &Vec<u8>) {
        self.data_cache.extend(data);
    }

    pub fn reset_position(&mut self) {
        self.sub_byte_counter = 0;
        self.cur_read_pos = 0;
    }
    pub fn decode_data_base_u8(&mut self, total_bits: u64) -> Option<(u8, u64)> {
        if self.cur_read_pos as usize >= self.data_cache.len() { return None }
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

    pub fn skip_bits(&mut self, bits: u64) {
        let bits_total_pos = bits + self.sub_byte_counter as u64;
        self.sub_byte_counter = (bits_total_pos.clone() & 7) as u8;
        self.cur_read_pos += bits_total_pos >> 3;
    }
}

macro_rules! impl_biserdi_var_bitsize_trait {
    ($type:ty, $num_bytes: expr) => {
        impl BiserdiTraitVarBitSize for $type {
            fn bit_serialize(self: &Self, total_bits: u64, biseri: &mut Biseri) -> Option<u64> {
                if Self::IS_SIGNED { self.is_val_negative().bit_serialize(biseri)?; }
                let v = self.get_absolute_uint();
                let vv = &v.to_le_bytes().to_vec();
                let bits = biseri.add_data(vv, total_bits)?;

                let bits_with_sign = if Self::IS_SIGNED { bits+1 } else { bits };
                Some(bits_with_sign)
            }
            fn bit_deserialize(version_id: u16, total_bits: u64, bides: &mut Bides) -> Option<(Self, u64)> {
                let is_neg = if Self::IS_SIGNED {
                    bool::bit_deserialize(version_id, bides)?.0 }
                else { false };

                let mut v = Self::from_le_bytes(
                    bides.decode_data(total_bits, $num_bytes)?.try_into().ok()?);
                if is_neg { v = v.switch_sign_if_possible() }

                let bits_with_sign = if Self::IS_SIGNED { total_bits + 1 } else { total_bits };
                Some((v, bits_with_sign))
            }
        }
    };
}
macro_rules! impl_biserdi {
    ($type:ty, $num_bits: expr) => {
        impl BiserdiTrait for $type {
            fn bit_serialize(self: &Self, biseri: &mut Biseri) -> Option<u64> {
                Some(biseri.add_data(&self.clone().to_le_bytes().to_vec(), ($num_bits))?)
            }
            fn bit_deserialize(_version_id: u16, bides: &mut Bides) -> Option<(Self, u64)> {
                Some((Self::from_le_bytes(bides.decode_data(
                    ($num_bits), std::cmp::max((($num_bits)>>3),1))?.try_into().ok()?),
                $num_bits))
            }
            fn min_bits() -> u64 { $num_bits as u64 }
        }
    };
}

// ****************************************************************************
pub fn call_min_bits<T: BiserdiTrait>() -> u64 { T::min_bits() }
pub fn call_default<T: Default>() -> T { T::default() }

// ****************************************************************************
// bool
impl BiserdiTrait for bool {
    fn bit_serialize(self: &Self, biseri: &mut Biseri) -> Option<u64> {
        let val = if *self { 1_u8 } else { 0_u8 };
        biseri.add_data_base_u8(&val, 1);
        Some(1)
    }
    fn bit_deserialize(_version_id: u16, bides: &mut Bides) -> Option<(Self, u64)> {
        let vec = bides.decode_data(1, 1)?;
        Some((if vec[0] == 0 { false } else { true }, 1))
    }
    fn min_bits() -> u64 { 1 }
}
impl ValFromInto<bool> for bool {
    fn val_into(self: &Self) -> bool { self.clone() }
    fn val_from(val: &bool) -> Self { val.clone() }
}

// ****************************************************************************
// integer
impl_biserdi_var_bitsize_trait!(u8, u8::BITS>>3);
impl_biserdi_var_bitsize_trait!(u16, u16::BITS>>3);
impl_biserdi_var_bitsize_trait!(u32, u32::BITS>>3);
impl_biserdi_var_bitsize_trait!(u64, u64::BITS>>3);
impl_biserdi_var_bitsize_trait!(i8, i8::BITS>>3);
impl_biserdi_var_bitsize_trait!(i16, i16::BITS>>3);
impl_biserdi_var_bitsize_trait!(i32, i32::BITS>>3);
impl_biserdi_var_bitsize_trait!(i64, i64::BITS>>3);

// ****************************************************************************
// floats
impl_biserdi!(f32, 32);
impl_biserdi!(f64, 64);

impl ValFromInto<f32> for f32 {
    fn val_into(self: &Self) -> f32 { self.clone() }
    fn val_from(val: &f32) -> Self { val.clone() }
}
impl ValFromInto<f64> for f64 {
    fn val_into(self: &Self) -> f64 { self.clone() }
    fn val_from(val: &f64) -> Self { val.clone() }
}

// ****************************************************************************
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntWithGivenBitSize<T: Clone + IntegerBaseFunctions + Sized + BiserdiTraitVarBitSize + Default + Display + Debug, const NUM_BITS: u64> {
    pub val: T
}
impl<T: Clone + IntegerBaseFunctions + Sized + BiserdiTraitVarBitSize + Default + Display + Debug, const NUM_BITS: u64> IntWithGivenBitSize<T, NUM_BITS> {
    pub fn new(v: T) -> Self { IntWithGivenBitSize { val: v } }
}
impl<T: Clone + IntegerBaseFunctions + Sized + BiserdiTraitVarBitSize + Default + Display + Debug, const NUM_BITS: u64> From<T> for IntWithGivenBitSize<T, NUM_BITS> {
    fn from(val: T) -> Self {
        IntWithGivenBitSize::<T, NUM_BITS>::new(val)
    }
}
impl<T: Clone + IntegerBaseFunctions + Sized + BiserdiTraitVarBitSize + Default + Display + Debug, const NUM_BITS: u64> Display for IntWithGivenBitSize<T, NUM_BITS> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} |bits:{}", self.val, NUM_BITS)
    } }
impl<T: Clone + IntegerBaseFunctions + Sized + BiserdiTraitVarBitSize + Default + Display + Debug, const NUM_BITS: u64> BiserdiTrait for IntWithGivenBitSize<T, NUM_BITS> {
    fn bit_serialize(self: &Self, biseri: &mut Biseri) -> Option<u64> {
        self.val.bit_serialize(NUM_BITS, biseri)
    }
    fn bit_deserialize(version_id: u16, bides: &mut Bides) -> Option<(Self, u64)> {
        let v = T::bit_deserialize(version_id, NUM_BITS, bides)?;
        Some((Self{val: v.0}, v.1))
    }
    fn min_bits() -> u64 { NUM_BITS }
}
impl<T: Clone + IntegerBaseFunctions + Sized + BiserdiTraitVarBitSize + Default + Display + Debug, const NUM_BITS: u64> Default for IntWithGivenBitSize<T, NUM_BITS> {
    fn default() -> Self { Self{val: Default::default()} }
}
impl<T: Clone + IntegerBaseFunctions + Sized + BiserdiTraitVarBitSize + Default + Display + Debug, const NUM_BITS: u64> ValFromInto<T> for IntWithGivenBitSize<T, NUM_BITS> {
    fn val_into(&self) -> T { self.val.clone() }
    fn val_from(val: &T) -> Self { Self{ val: val.clone() } }
}

// ****************************************************************************
#[derive(Debug, Clone, PartialEq, Eq, Default, Copy)]
pub struct DynInteger<
    T: Sized + Copy + BiserdiTraitVarBitSize + AddAssign + Shl<Output = T> + Shr + Ord + PartialEq //+ TryFrom<u64>
    + IntegerBaseFunctions + Default, const MAX_BITS: u8, const BIT_PACKS: u8> { pub val: T }
#[allow(dead_code)]
impl<T: Display + Sized + Copy + BiserdiTraitVarBitSize + AddAssign + Shl<Output = T> + Shr + Ord + PartialEq + TryFrom<u64>
+ IntegerBaseFunctions + Default, const MAX_BITS: u8, const BIT_PACKS: u8> DynInteger<T, MAX_BITS, BIT_PACKS> {
    const MAX_BITS: u8 = MAX_BITS;
    const BIT_PACKS: u8 = BIT_PACKS;
    pub fn new(v: T) -> Self{
        DynInteger{val: v}
    }
}
impl<T: Display + Sized + Copy + BiserdiTraitVarBitSize + AddAssign + Shl<Output = T> + Shr + Ord + PartialEq + TryFrom<u64>
+ IntegerBaseFunctions + Default, const MAX_BITS: u8, const BIT_PACKS: u8> BiserdiTrait for DynInteger<T, MAX_BITS, BIT_PACKS> {
    fn bit_serialize(self: &Self, biseri: &mut Biseri) -> Option<u64> {
        // let br = self.bits_required();
        // let dyn_sections = br / Self::DYN_SIZE;
        let mut bit_size: u64 = 1;
        let mut val_work = self.val.get_absolute_uint();

        if T::IS_SIGNED { self.val.is_val_negative().bit_serialize(biseri)?; bit_size +=1; }

        (val_work != 0).bit_serialize(biseri);
        let mut max_bits_num = MAX_BITS;
        // println!("max_bits_num: {}, val_work: {}", max_bits_num, val_work);
        while val_work > 0 {
            let cur_bitsize = std::cmp::min(max_bits_num, Self::BIT_PACKS);

            val_work.bit_serialize(u64::from(cur_bitsize), biseri)?;
            val_work >>= cur_bitsize;
            bit_size += cur_bitsize as u64;

            max_bits_num -= cur_bitsize;
            // println!("max_bits_num: {}, val_work: {}", max_bits_num, val_work);
            if max_bits_num > 0 {
                let further_data = val_work > 0;
                further_data.bit_serialize(biseri);
                bit_size += 1;
            }
        }
        Some(bit_size)
    }
    fn bit_deserialize(version_id: u16, bides: &mut Bides) -> Option<(Self, u64)> {
        let mut cur_shift: u64 = 0;
        let mut v: u64 = 0;
        let mut negative_sign = false;

        let mut bit_size = 1;
        if T::IS_SIGNED {
            negative_sign = bool::bit_deserialize(version_id, bides)?.0; bit_size += 1; }
        let mut further_data = bool::bit_deserialize(version_id, bides)?.0;
        let mut max_bits_num = MAX_BITS;
        while further_data {
            let cur_bitsize = std::cmp::min(max_bits_num, Self::BIT_PACKS);

            let vt = u64::bit_deserialize(version_id, cur_bitsize as u64, bides)?;
            bit_size += vt.1 + 1;
            v += vt.0 << cur_shift;
            cur_shift += u64::from(cur_bitsize);

            max_bits_num -= cur_bitsize;

            // println!("max_bits_num: {}, bit_size: {}", max_bits_num, bit_size);

            if max_bits_num > 0 {
                further_data = bool::bit_deserialize(version_id, bides)?.0;
            }
            else { further_data = false }
        }
        let mut vv= T::try_from(v).ok()?;
        if negative_sign {
            vv = vv.switch_sign_if_possible();
        }
        Some((Self{val: vv}, bit_size))
    }
    fn min_bits() -> u64 { 1 }
}
impl<T: Display + Sized + Copy + BiserdiTraitVarBitSize + AddAssign + Shl<Output = T> + Shr + Ord + PartialEq + TryFrom<u64>
+ IntegerBaseFunctions + Default, const MAX_BITS: u8, const BIT_PACKS: u8> Display for DynInteger<T, MAX_BITS, BIT_PACKS> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} |{}d{}]", self.val, MAX_BITS, BIT_PACKS)
    } }
impl<T: Display + Sized + Copy + BiserdiTraitVarBitSize + AddAssign + Shl<Output = T> + Shr + Ord + PartialEq + TryFrom<u64>
+ Default + IntegerBaseFunctions, const MAX_BITS: u8, const BIT_PACKS: u8> From<T> for DynInteger<T, MAX_BITS, BIT_PACKS> {
    fn from(val: T) -> Self { DynInteger::<T, MAX_BITS, BIT_PACKS>::new(val) }
}
impl<T: Display + Sized + Copy + BiserdiTraitVarBitSize + AddAssign + Shl<Output = T> + Shr + Ord + PartialEq + TryFrom<u64>
+ Default + IntegerBaseFunctions, const MAX_BITS: u8, const BIT_PACKS: u8> ValFromInto<T> for DynInteger<T, MAX_BITS, BIT_PACKS> {
    fn val_into(self: &Self) -> T { self.val.clone() }
    fn val_from(val: &T) -> Self { DynInteger::from(val.clone()) }
}

// ****************************************************************************
#[derive(Debug, Clone, Default)]
pub enum FixPrecisionVal {
    #[default]
    Overflow,
    Value(f64),
    Underflow
}
#[derive(Debug, Clone, Default)]
pub struct FixPrecisionMinMax<const NUM_BITS: u8, const MIN_IVALUE: i64, const MAX_IVALUE: i64> {
    pub val: FixPrecisionVal
}
impl<const NUM_BITS: u8, const MIN_IVALUE: i64, const MAX_IVALUE: i64> FixPrecisionMinMax<NUM_BITS, MIN_IVALUE, MAX_IVALUE> {
    const MIN_VALUE: f64 = MIN_IVALUE as f64;
    const MAX_VALUE: f64 = MAX_IVALUE as f64;
    const RANGE_VALUE: f64 = Self::MAX_VALUE - Self::MIN_VALUE;
    const MAX_INT_VALUE_FOR_BITS: u64 = (1_u64<<NUM_BITS) - 1_u64;
    const MAX_VALUE_FOR_BITS: f64 = (Self::MAX_INT_VALUE_FOR_BITS - 2_u64) as f64;

    pub fn new(val: f64) -> Self {
        let mut obj = Self{val: FixPrecisionVal::Underflow};
        obj.set(val);
        obj
    }
    pub fn set(&mut self, val: f64) {
        self.val = Self::u64_to_val(Self::val_to_u64(&Self::val_to_enum(val)));
    }
    fn val_to_enum(val: f64) -> FixPrecisionVal {
        if val > Self::MAX_VALUE { FixPrecisionVal::Overflow }
        else if val < Self::MIN_VALUE { FixPrecisionVal::Underflow }
        else { FixPrecisionVal::Value(val) }
    }
    fn val_to_u64(v: &FixPrecisionVal) -> u64 {
        let vu = match v {
            // i = (v-Min) / (Max-Min) * 253 + 1
            FixPrecisionVal::Value(v) => {
                let vv= (v - Self::MIN_VALUE) / Self::RANGE_VALUE * Self::MAX_VALUE_FOR_BITS + 1.0;
                vv.round() as u64
            },
            FixPrecisionVal::Underflow => 0,
            FixPrecisionVal::Overflow => Self::MAX_INT_VALUE_FOR_BITS
        };
        // println!("vu: {}", vu);
        vu
    }
    fn u64_to_val(v: u64) -> FixPrecisionVal {
        if v == 0 { FixPrecisionVal::Underflow }
        else if v == Self::MAX_INT_VALUE_FOR_BITS { FixPrecisionVal::Overflow }
        else {
            FixPrecisionVal::Value(((v-1) as f64) / Self::MAX_VALUE_FOR_BITS
                * Self::RANGE_VALUE + Self::MIN_VALUE)
        }
    }
}
impl<const NUM_BITS: u8, const MIN_IVALUE: i64, const MAX_IVALUE: i64> BiserdiTrait for FixPrecisionMinMax<NUM_BITS, MIN_IVALUE, MAX_IVALUE> {
    fn bit_serialize(self: &Self, biseri: &mut Biseri) -> Option<u64> {
        let v = Self::val_to_u64(&self.val);
        v.bit_serialize(NUM_BITS as u64, biseri)
    }
    fn bit_deserialize(version_id: u16, bides: &mut Bides) -> Option<(Self, u64)> {
        // v = (i-1)/253 * (Max-Min) + Min
        let (v, bits) = u64::bit_deserialize(version_id, NUM_BITS as u64, bides)?;
        let vv = Self::u64_to_val(v);
        Some((Self{val: vv}, bits))
    }
    fn min_bits() -> u64 { NUM_BITS as u64 }
}
impl<const NUM_BITS: u8, const MIN_IVALUE: i64, const MAX_IVALUE: i64> Display for FixPrecisionMinMax<NUM_BITS, MIN_IVALUE, MAX_IVALUE> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.val {
            FixPrecisionVal::Overflow => write!(f, "Overflow |dynbits:{}", NUM_BITS),
            FixPrecisionVal::Underflow => write!(f, "Underflow |dynbits:{}", NUM_BITS),
            FixPrecisionVal::Value(v) => write!(f, "{} |dynbits:{}", v, NUM_BITS)
        }
    } }
impl<const NUM_BITS: u8, const MIN_IVALUE: i64, const MAX_IVALUE: i64> From<f32> for FixPrecisionMinMax<NUM_BITS, MIN_IVALUE, MAX_IVALUE> {
    fn from(value: f32) -> Self { Self::new(value.into()) }
}
impl<const NUM_BITS: u8, const MIN_IVALUE: i64, const MAX_IVALUE: i64> From<f64> for FixPrecisionMinMax<NUM_BITS, MIN_IVALUE, MAX_IVALUE> {
    fn from(value: f64) -> Self { Self::new(value.into()) }
}
impl<const NUM_BITS: u8, const MIN_IVALUE: i64, const MAX_IVALUE: i64> PartialEq for FixPrecisionMinMax<NUM_BITS, MIN_IVALUE, MAX_IVALUE> {
    fn eq(&self, other: &Self) -> bool {
        Self::val_to_u64(&self.val) == Self::val_to_u64(&other.val)
    }
    fn ne(&self, other: &Self) -> bool { !self.eq(other) }
}
impl<const NUM_BITS: u8, const MIN_IVALUE: i64, const MAX_IVALUE: i64> Into<f64> for FixPrecisionMinMax<NUM_BITS, MIN_IVALUE, MAX_IVALUE> {
    fn into(self) -> f64 {
        match self.val {
            FixPrecisionVal::Value(v) => { v },
            FixPrecisionVal::Underflow => { Self::MIN_VALUE - 1.0 }
            FixPrecisionVal::Overflow => { Self::MAX_VALUE + 1.0 }
        }
    }
}
impl<const NUM_BITS: u8, const MIN_IVALUE: i64, const MAX_IVALUE: i64> ValFromInto<f64> for FixPrecisionMinMax<NUM_BITS, MIN_IVALUE, MAX_IVALUE> {
    fn val_into(&self) -> f64 {
        match self.val {
            FixPrecisionVal::Overflow => { Self::MAX_VALUE + 1.0 },
            FixPrecisionVal::Value(v) => { v },
            FixPrecisionVal::Underflow =>  { Self::MIN_VALUE - 1.0 },
        }
    }
    fn val_from(val: &f64) -> Self {
        Self::new(val.clone())
    }
}

// ****************************************************************************
#[derive(Debug, Clone, PartialEq)]
pub struct Binary<const DYNSIZEBITS: u8> {
    pub val: Vec<u8>
}
impl<const DYNSIZEBITS: u8> Binary<DYNSIZEBITS> {
    pub fn new(data: Vec<u8>) -> Self {
        Self{ val: data }
    }
    pub fn empty() -> Self {
        Self{val: Vec::new()}
    }
}
impl<const DYNSIZEBITS: u8> BiserdiTrait for Binary<DYNSIZEBITS> {
    fn bit_serialize(self: &Self, biseri: &mut Biseri) -> Option<u64> {
        let mut s = 0;
        s += DynInteger::<u32, 32, DYNSIZEBITS>::new(self.val.len() as u32).bit_serialize(biseri)?;
        for d in self.val.iter() { s += d.bit_serialize(8, biseri)?; };
        Some(s)
    }
    fn bit_deserialize(version_id: u16, bides: &mut Bides) -> Option<(Self, u64)> {
        let mut s = 0;
        let (v, cs) =
            DynInteger::<u32, 32, DYNSIZEBITS>::bit_deserialize(version_id, bides)?;
        let mut data = Vec::with_capacity(cs as usize);
        for _ci in 0..v.val {
            let (vi, si) = u8::bit_deserialize(version_id, 8, bides)?;
            s += si;
            data.push(vi);
        }
        Some((Self{ val: data }, s+cs))
    }
    fn min_bits() -> u64 { 1 }
}
impl<const N: u8> std::fmt::Display for Binary< N> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let hex = self.val.iter().map(|b| format!("{:02x}", b).to_string()).collect::<Vec<String>>().join(" ");
        write!(f, "{:x?} |dynbits:{}", hex, N)
    }
}
impl<const N: u8> Default for Binary<N> {
    fn default() -> Self { Self{val: Vec::new()} }
}

// *****
#[derive(Clone, PartialEq)]
pub struct BitisAString<const DYNSIZEBITS: u8> {
    val: Binary<DYNSIZEBITS>
}
impl<const DYNSIZEBITS: u8> BitisAString<DYNSIZEBITS> {
    pub fn from_str(data: AsciiString) -> Self {
        Self{ val: Binary::new(data.as_bytes().into()) }
    }
    pub fn empty() -> Self {
        Self{val: Binary::empty()}
    }
    pub fn set(&mut self, data: AsciiString) {
        self.val = Binary::new(data.as_bytes().into())
    }
    pub fn get (&self) -> AsciiString {
        AsciiString::from_ascii(self.val.val.clone()).unwrap()
    }
    pub fn get_string (&self) -> String {
        AsciiString::from_ascii(self.val.val.clone()).unwrap().to_string()
    }
}
impl<const DYNSIZEBITS: u8> BiserdiTrait for BitisAString<DYNSIZEBITS> {
    fn bit_serialize(self: &Self, biseri: &mut Biseri) -> Option<u64> {
        self.val.bit_serialize(biseri)
    }
    fn bit_deserialize(version_id: u16, bides: &mut Bides) -> Option<(Self, u64)> {
        let (v, s) = Binary::bit_deserialize(version_id, bides)?;
        Some((BitisAString{val: v}, s))
    }
    fn min_bits() -> u64 { Binary::<DYNSIZEBITS>::min_bits() }
}
impl<const DYNSIZEBITS: u8> std::fmt::Display for BitisAString<DYNSIZEBITS> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "'{}' |string:{}", self.get(), DYNSIZEBITS)
    }
}
impl<const DYNSIZEBITS: u8> Debug for BitisAString<DYNSIZEBITS> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "'{}'", self.get())
    }
}
impl<const DYNSIZEBITS: u8> Default for BitisAString<DYNSIZEBITS> {
    fn default() -> Self { Self::empty() }
}
impl<const DYNSIZEBITS: u8> From<ascii::AsciiString> for BitisAString<DYNSIZEBITS> {
    fn from(data: ascii::AsciiString) -> Self { Self::from_str(data) }
}
impl<const DYNSIZEBITS: u8> From<String> for BitisAString<DYNSIZEBITS> {
    fn from(value: String) -> Self { Self::val_from(&value) }
}
impl<const DYNSIZEBITS: u8> ValFromInto<String> for BitisAString<DYNSIZEBITS> {
    fn val_into(self: &Self) -> String { self.get().to_string() }
    fn val_from(val: &String) -> Self {
        match val.clone().as_ascii_str() {
            Ok(val) => Self::from_str(val.into()),
            Err(_) => Self::empty(),
        }
    }
}


// ****************************************************************************
// Option
#[derive(Clone, Debug, PartialEq)]
pub struct BitisOption<T> {
    pub val: Option<T>
}
impl<T> BitisOption<T> where T: BiserdiTrait + Default {
    pub fn new_some(val: T) -> BitisOption<T> { BitisOption { val: Some(val) } }
    pub fn new_none() -> BitisOption<T> { BitisOption { val: None } }
}
impl<T> BiserdiTrait for BitisOption<T> where T: BiserdiTrait + Default {
    fn bit_serialize(self: &Self, biseri: &mut Biseri) -> Option<u64> {
        let mut size = 1;
        match &self.val {
            None => { false.bit_serialize(biseri)?; },
            Some(v) => {
                true.bit_serialize(biseri)?;
                size += v.bit_serialize(biseri)?;
            }
        }
        Some(size)
    }
    fn bit_deserialize(version_id: u16, bides: &mut Bides) -> Option<(Self, u64)> {
        let mut size = 1;

        let (is_set, _) = bool::bit_deserialize(version_id, bides)?;
        let v = if is_set {
            let vv = T::bit_deserialize(version_id, bides)?;
            size += vv.1.clone();
            Some(vv.0)
        }
        else { None };

        Some((BitisOption{val: v}, size))
    }
    fn min_bits() -> u64 { 1 }
}
impl<T, TT> From<Option<TT>> for BitisOption<T> where T: BiserdiTrait + Default, TT: Into<T> {
    fn from(value: Option<TT>) -> Self {
        match value {
            Some(v) => Self { val: Some(v.into()) },
            None => Self { val: None }
        }
    }
}
impl<T> From<T> for BitisOption<T> where T: BiserdiTrait + Default {
    fn from(val: T) -> Self {
        Self{ val: Some(val) }
    } }
impl<T> Into<Option<T>> for BitisOption<T> {
    fn into(self) -> Option<T> { self.val }
}
impl<T> std::fmt::Display for BitisOption<T> where T: BiserdiTrait + Default + Display {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.val {
            None => { write!(f, "None [opt]") },
            Some(v) => { write!(f, "{} [opt]", v) }
        }
    } }
impl<T> Default for BitisOption<T> where T: BiserdiTrait + Default {
    fn default() -> Self { Self{val: None} }
}
impl<T, U> ValFromInto<Option<U>> for BitisOption<T> where T: BiserdiTrait + Default + Clone + ValFromInto<U> {
    fn val_into(&self) -> Option<U> { match self.val.clone() { Some(v) => Some(v.val_into()), None => None } }
    fn val_from(v: &Option<U>) -> Self {
        match v {
            Some(v) => Self{ val: Some(T::val_from(v)) },
            None => Self{ val: None }
        }
    }
}

// ****************************************************************************
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixedArray<T: Sized + BiserdiTrait + Default + Debug, const N: usize> { pub val: [T; N] }
impl<T, const N: usize> BiserdiTrait for FixedArray<T, N> where T: Sized + Clone + BiserdiTrait + Default + std::fmt::Debug {
    fn bit_serialize(self: &Self, biseri: &mut Biseri) -> Option<u64> {
        let mut s = 0;
        for i in 0..N { s += self.val[i].bit_serialize(biseri)?; }
        Some(s)
    }
    fn bit_deserialize(version_id: u16, bides: &mut Bides) -> Option<(Self, u64)> {
        //let mut v: [T; N] = vec![T::default(); N].try_into::<[T; N]>().unwrap_err_unchecked();
        let mut v: [T; N] = array_init::array_init(|_| T::default());
        let mut bits = 0;
        let mut cur_bits;
        for i in 0..N { (v[i], cur_bits) = T::bit_deserialize(version_id, bides)?; bits += cur_bits; }
        Some((Self{ val: v}, bits))
    }
    fn min_bits() -> u64 { (N as u64) * T::min_bits() }
}
impl<T: Sized + Clone + BiserdiTrait + Default + Debug, const N: usize> Display for FixedArray<T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let das: Vec<String> = self.val.iter().map(|v| format!("{:?}", v)).collect();
        write!(f, "[{}]", das.join(", "))
    } }
impl<T: Sized + BiserdiTrait + Default + Debug, const N: usize> Default for FixedArray<T, N>  {
    fn default() -> Self {
        Self{val: array_init::array_init(|_| T::default())}
    }
}
impl<T: Sized + BiserdiTrait + Default + Debug + Clone, const N: usize> Into<[T; N]> for FixedArray<T, N> {
    fn into(self) -> [T; N] { map_array_init(&self.val, |v| v.clone()) }
}
impl<T: Sized + Clone + BiserdiTrait + Default + Debug, const N: usize> From<[T; N]> for FixedArray<T, N> {
    fn from(v: [T;N]) -> Self { Self{ val: map_array_init(&v, |v| v.clone()) } }
}
impl<T: Sized + Clone + BiserdiTrait + Default + Debug + ValFromInto<U>, const N: usize, U> ValFromInto<[U; N]> for FixedArray<T, N> {
    fn val_into(&self) -> [U; N] { map_array_init(&self.val, |v| v.clone().val_into()) }
    fn val_from(v: &[U; N]) -> Self { Self{val: map_array_init(&v, |v| T::val_from(v).into())} }
}

// ****************************************************************************
#[derive(Debug, Clone, PartialEq)]
pub struct DynArray<T, const DYNSIZEBITS: u8> where T: BiserdiTrait + Default { pub val: Vec<T> }
impl<T, const DYNSIZEBITS: u8> BiserdiTrait for DynArray<T, DYNSIZEBITS> where T: BiserdiTrait + Default + Clone {
    fn bit_serialize(self: &Self, biseri: &mut Biseri) -> Option<u64> {
        let mut s = 0;
        s += DynInteger::<u32, 32, DYNSIZEBITS>::new(self.val.len() as u32).bit_serialize(biseri)?;
        for d in self.val.iter() { s += d.bit_serialize(biseri)?; };
        Some(s)
    }
    fn bit_deserialize(version_id: u16, bides: &mut Bides) -> Option<(Self, u64)> {
        let mut s = 0;
        let (v, cs) =
            DynInteger::<u32, 32, DYNSIZEBITS>::bit_deserialize(version_id, bides)?;
        let mut data = Vec::with_capacity(cs as usize);
        for _ci in 0..v.val {
            let (vi, si) = T::bit_deserialize(version_id, bides)?;
            s += si;
            data.push(vi);
        }
        Some((Self{ val: data }, s+cs))
    }
    fn min_bits() -> u64 { 1 }
}
impl<T: Sized + BiserdiTrait + Default + Debug + Clone, const DYNSIZEBITS: u8, const N: usize> From<[T; N]> for DynArray<T, DYNSIZEBITS> {
    fn from(val: [T;N]) -> Self {
        DynArray{val: val.to_vec()}
    } }
impl<T: Sized + BiserdiTrait + Default + Debug + Clone, const DYNSIZEBITS: u8> From<Vec<T>> for DynArray<T, DYNSIZEBITS> {
    fn from(val: Vec<T>) -> Self {
        DynArray{val: val}
    } }
impl<T: Sized + BiserdiTrait + Display + Debug + Default + Clone, const DYNSIZEBITS: u8> std::fmt::Display for DynArray<T, DYNSIZEBITS> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let das: Vec<String> = self.val.iter().map(|v| v.to_string()).collect();
        write!(f, "[{} |dynbits:{}]", das.join(", "), DYNSIZEBITS)
    } }
impl<T: Sized + BiserdiTrait + Default + Debug + Clone, const DYNSIZEBITS: u8> Default for DynArray<T, DYNSIZEBITS> {
    fn default() -> Self {
        Self{val: Vec::new()}
    }
}
impl<T: Sized + BiserdiTrait + Default + Debug, const DYNSIZEBITS: u8> Into<Vec<T>> for DynArray<T, DYNSIZEBITS> {
    fn into(self) -> Vec<T> { self.val }
}
impl<T: Sized + BiserdiTrait + Default + Debug +  ValFromInto<U>, const DYNSIZEBITS: u8, U> ValFromInto<Vec<U>> for DynArray<T, DYNSIZEBITS> {
    fn val_into(&self) -> Vec<U> { self.val.iter().map(|v| v.val_into()).collect() }
    fn val_from(val: &Vec<U>) -> Self { Self { val: val.iter().map(|v| { T::val_from(v).into() }).collect() } }
}



#[cfg(test)]
mod bitis_base_serialization_deserialization {
    use rstest::rstest;
    use crate::lib_impl::berde::{Bides, Biseri};
    use crate::{deserialize, serialize};
    use super::*;

    #[rstest]
    fn add_one_u8() {
        let mut b = Biseri::new();
        let d = 0b10101010_u8;

        b.add_data_base_u8(&d, 8);

        assert_eq!(b.clone().data_cache.len(), 1);
        assert_eq!(b.sub_byte_counter, 0);

        let r = b.finish_add_data().unwrap();
        assert_eq!((r.total_bits, r.total_bytes), (8, 1));
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

        let r = b.finish_add_data().unwrap();
        assert_eq!((r.total_bits, r.total_bytes), (final_bitsize, num_bytes));
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

    fn add_two_u16_fixed(ser: &mut Biseri, bits: u64) -> BiserSizes {
        let d: u16 = 3;

        ser.add_data(&d.to_le_bytes().to_vec(), bits);
        ser.add_data(&d.to_le_bytes().to_vec(), bits);
        ser.finish_add_data().unwrap()
    }
    #[rstest]
    fn serialize_u16_fixed_full() {
        let mut ser = Biseri::new();

        let r = add_two_u16_fixed(&mut ser, 16);
        let (lbits, lbytes) = (r.total_bits, r.total_bytes);

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

        let r = add_two_u16_fixed(&mut ser, 12);
        let (lbits, lbytes) = (r.total_bits, r.total_bytes);

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

        let _ = add_two_u16_fixed(&mut ser, bits);

        let mut des = Bides::from_biseri(&ser);

        assert_eq!(des.data_cache, ser.data_cache);

        let d1 = des.decode_data(bits, 2);
        assert!(d1.is_some());
        let d2 = des.decode_data(bits, 2);
        assert!(d2.is_some());

        let d1 = d1.unwrap();
        let d2 = d2.unwrap();

        assert_eq!(d1[0], 3);
        assert_eq!(d1[1], 0);
        assert_eq!(d2[0], 3);
        assert_eq!(d2[1], 0);
    }

    #[rstest]
    fn ser_and_deserialize_i16_fixed() {
        let mut ser = Biseri::new();

        let v: i8 = -11;
        v.bit_serialize(5, &mut ser);

        let r = ser.finish_add_data().unwrap();
        let (bits, bytes) = (r.total_bits, r.total_bytes);

        println!("bits: {}, bytes: {}", bits, bytes);

        let mut des = Bides::from_biseri(&ser);

        let vv = i8::bit_deserialize(1,5, &mut des);
        println!("v: {}, vv: {:?}", v, vv);

        assert!(vv.is_some());

        let vv = vv.unwrap();
        println!("bits_des: {}", vv.1);
        assert_eq!(v, vv.0);
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
        let r = ser.finish_add_data().unwrap();
        let (bits, bytes) = (r.total_bits, r.total_bytes);

        println!("bits: {}, bytes: {}", bits, bytes);

        // // ***********************************************************
        let mut des = Bides::from_biseri(&ser);
        let vo1 = u8::bit_deserialize(1,6, &mut des);
        // let vo1 = des.decode_data_u8(6);
        let vo2 = u16::bit_deserialize(1,14, &mut des);
        // let vo2 = des.decode_data_u16(14);
        let vo3 = u32::bit_deserialize(1,22, &mut des);
        // let vo3 = des.decode_data_u32(30);

        println!("v1: {}, v2: {}, v3: {} vs vo1: {:?}, vo2: {:?}, vo3: {:?}", v1, v2, v3, vo1, vo2, vo3);

        // ***********************************************************
        assert!(vo1.is_some());
        assert_eq!(v1, vo1.unwrap().0);

        assert!(vo2.is_some());
        assert_eq!(v2, vo2.unwrap().0);

        assert!(vo3.is_some());
        assert_eq!(v3, vo3.unwrap().0);
    }

    #[rstest]
    fn de_and_serialize_various_float() {
        // ***********************************************************
        let mut ser = Biseri::new();
        let v1: f32 = 56.78;
        v1.bit_serialize(&mut ser);
        // ser.add_data_f32(&v1);
        let v2: u8 = 5;
        v2.bit_serialize(5, &mut ser);
        // ser.add_data_u8(&v2, 5);
        let v3: bool = true;
        v3.bit_serialize(&mut ser);
        // ser.add_data_bool(&v3);
        let v4: bool = false;
        v4.bit_serialize(&mut ser);
        // ser.add_data_bool(&v4);
        v1.bit_serialize(&mut ser);
        // ser.add_data_f32(&v1);
        let r = ser.finish_add_data().unwrap();
        let (bits, bytes) = (r.total_bits, r.total_bytes);

        println!("bits: {}, bytes: {}", bits, bytes);

        // ***********************************************************
        let mut des = Bides::from_biseri(&ser);
        let vo1 = f32::bit_deserialize(1, &mut des);
        let vo2 = u8::bit_deserialize(1,5, &mut des);
        let vo3 = bool::bit_deserialize(1,&mut des);
        let vo4 = bool::bit_deserialize(1,&mut des);
        let vo5 = f32::bit_deserialize(1,&mut des);
        // let vo1 = des.decode_data_f32();
        // let vo2 = des.decode_data_u8(5);
        // let vo3 = des.decode_data_bool();
        // let vo4 = des.decode_data_bool();
        // let vo5 = des.decode_data_f32();

        println!("vo1: {:?}, vo2: {:?}, vo3: {:?}, vo4: {:?}, vo4: {:?}", vo1, vo2, vo3, vo4, vo5);

        // ***********************************************************
        assert!(vo1.is_some());
        assert_eq!(v1, vo1.unwrap().0);

        assert!(vo2.is_some());
        assert_eq!(v2, vo2.unwrap().0);

        assert!(vo3.is_some());
        assert_eq!(v3, vo3.unwrap().0);

        assert!(vo4.is_some());
        assert_eq!(v4, vo4.unwrap().0);

        assert!(vo5.is_some());
        assert_eq!(v1, vo5.unwrap().0);
    }

    #[rstest]
    fn serialize_and_deserialize_array_uint() {
        let mut ser = Biseri::new();

        let v: FixedArray<IntWithGivenBitSize<u16, 5>, 4> = [11.into(), 12.into(), 22.into(), 23.into()].into();
        v.bit_serialize(&mut ser);
        let r = ser.finish_add_data().unwrap();
        let (bits, bytes) = (r.total_bits, r.total_bytes);

        println!("bits: {}, bytes: {}", bits, bytes);

        let mut des = Bides::from_biseri(&ser);
        let vv = FixedArray::<IntWithGivenBitSize<u16, 5>, 4>::bit_deserialize(1, &mut des);

        assert!(vv.is_some());
        let vv = vv.unwrap().0;

        assert_eq!(v.val[0], vv.val[0]);
        assert_eq!(v.val[1], vv.val[1]);
        assert_eq!(v.val[2], vv.val[2]);
        assert_eq!(v.val[3], vv.val[3]);
    }

    #[rstest]
    fn serialize_and_deserialize_array_bool() {
        let mut ser = Biseri::new();

        let v= FixedArray::from([true, true, false, true]);
        v.bit_serialize(&mut ser);
        let r = ser.finish_add_data().unwrap();
        let (bits, bytes) = (r.total_bits, r.total_bytes);
        println!("bits: {}, bytes: {}", bits, bytes);

        let mut des = Bides::from_biseri(&ser);
        let vv = FixedArray::<bool, 4>::bit_deserialize(1,&mut des);

        assert!(vv.is_some());
        let vv = vv.unwrap().0;

        assert_eq!(v.val[0], vv.val[0]);
        assert_eq!(v.val[1], vv.val[1]);
        assert_eq!(v.val[2], vv.val[2]);
        assert_eq!(v.val[3], vv.val[3]);
    }
    #[rstest]
    fn serialize_and_deserialize_array_f64() {
        let mut ser = Biseri::new();

        let v = FixedArray::from([1.1, 1.2, 22.34, 123456.78]);
        v.bit_serialize(&mut ser);
        let r = ser.finish_add_data().unwrap();
        let (bits, bytes) = (r.total_bits, r.total_bytes);
        println!("bits: {}, bytes: {}", bits, bytes);

        let mut des = Bides::from_biseri(&ser);
        let vv = FixedArray::<f64, 4>::bit_deserialize(1,&mut des);

        assert!(vv.is_some());
        let vv = vv.unwrap().0;

        assert_eq!(v.val[0], vv.val[0]);
        assert_eq!(v.val[1], vv.val[1]);
        assert_eq!(v.val[2], vv.val[2]);
        assert_eq!(v.val[3], vv.val[3]);
    }

    #[rstest]
    #[case::val_0(0, 1, 1)]
    #[case::val_1(1, 5, 1)]
    #[case::val_10(10, 9, 2)]
    fn serialize_dyn_int_u32_3(#[case] val: u32, #[case] ex_bits: u64, #[case] ex_bytes: u64) {
        // #[test]
        // fn serialize_dyn_int_u32_3() {
        //     let val: u32 = 10;
        //     let ex_bits: u64 = 9;
        //     let ex_bytes: u64 = 2;

        let mut ser = Biseri::new();

        let v = DynInteger::<u32, 32, 3>::new(val);
        v.bit_serialize(&mut ser);
        let r = ser.finish_add_data().unwrap();
        let (bits, bytes) = (r.total_bits, r.total_bytes);
        println!("bits: {}, bytes: {}", bits, bytes);

        assert_eq!(bits, ex_bits);
        assert_eq!(bytes, ex_bytes);
    }

    #[rstest]
    #[case::val_0(0, 2, 1)]
    #[case::val_1(1, 6, 1)]
    #[case::val_10(10, 10, 2)]
    fn serialize_dyn_int_i32_3(#[case] val: i32, #[case] ex_bits: u64, #[case] ex_bytes: u64) {
        // #[test]
        // fn serialize_dyn_int_i32_3() {
        //     let val: i32 = -1;
        //     let ex_bits: u64 = 6;
        //     let ex_bytes: u64 = 1;

        let mut ser = Biseri::new();

        let v = DynInteger::<i32, 32, 3>::new(val);
        v.bit_serialize(&mut ser);
        let r = ser.finish_add_data().unwrap();
        let (bits, bytes) = (r.total_bits, r.total_bytes);
        println!("bits: {}, bytes: {}", bits, bytes);

        assert_eq!(bits, ex_bits);
        assert_eq!(bytes, ex_bytes);
    }

    #[rstest]
    #[case::val_0(0)]
    #[case::val_1(1)]
    #[case::val_10(10)]
    #[case::val_m1(-1)]
    #[case::val_m1111(-1111)]
    fn ser_and_deserialize_dyn_int_i32_3(#[case] val: i32) {
        // #[test]
        // fn ser_and_deserialize_dyn_int_i32_3() {
        //     let val: i32 = -111;

        let mut ser = Biseri::new();

        let v = DynInteger::<i32, 32, 3>::new(val);
        v.bit_serialize(&mut ser);
        let r = ser.finish_add_data().unwrap();
        let (bits, bytes) = (r.total_bits, r.total_bytes);
        println!("bits: {}, bytes: {}", bits, bytes);

        let mut der = Bides::from_biseri(&ser);

        let dv = DynInteger::<i32, 32, 3>::bit_deserialize(1, &mut der);
        assert!(dv.is_some());

        let dv = dv.unwrap();
        assert_eq!(val, dv.0.val);
    }

    #[rstest]
    fn ser_and_deserialize_fixed_int() {
        let mut ser = Biseri::new();

        let v = IntWithGivenBitSize::<u32, 20>::new(1111);
        v.bit_serialize(&mut ser);

        let r = ser.finish_add_data().unwrap();
        let (bits, bytes) = (r.total_bits, r.total_bytes);
        println!("bits: {}, bytes: {}", bits, bytes);

        let mut der = Bides::from_biseri(&ser);
        let vv = IntWithGivenBitSize::<u32, 20>::bit_deserialize(1, &mut der);

        println!("v: {:?}, vv: {:?}", v, vv);
        assert!(vv.is_some());
        let vv = vv.unwrap().0;

        assert_eq!(v.val, vv.val);
    }

    #[rstest]
    fn ser_and_deserialize_fixed_int_not_enough_data() {
        let mut ser = Biseri::new();

        let v = IntWithGivenBitSize::<u32, 20>::new(1111);
        v.bit_serialize(&mut ser);

        let r = ser.finish_add_data().unwrap();
        let (bits, bytes) = (r.total_bits, r.total_bytes);
        println!("bits: {}, bytes: {}", bits, bytes);

        let mut der = Bides::from_biseri(&ser);
        der.data_cache.truncate(1);
        let vv = IntWithGivenBitSize::<u32, 20>::bit_deserialize(1, &mut der);

        println!("v: {:?}, vv: {:?}", v, vv);
        assert!(vv.is_none());
    }

    #[rstest]
    fn ser_and_deserialize_fixed_precision_1() {
        let mut ser = Biseri::new();

        let v = FixPrecisionMinMax::<20, -50, 50>::new(12.3456);
        v.bit_serialize(&mut ser);

        let r = ser.finish_add_data().unwrap();
        let (bits, bytes) = (r.total_bits, r.total_bytes);
        println!("bits: {}, bytes: {}", bits, bytes);

        let mut der = Bides::from_biseri(&ser);
        let vv = FixPrecisionMinMax::<20, -50, 50>::bit_deserialize(1, &mut der);

        println!("v: {:?}, vv: {:?}", v, vv);
        assert!(vv.is_some());
        let vv = vv.unwrap().0;

        let eps = 1e-1;
        match v.val {
            FixPrecisionVal::Value(fpv) => {
                match vv.val {
                    FixPrecisionVal::Value(fpvv) => assert!((fpv - fpvv).abs() < eps),
                    _ => assert!(false)
                }
            },
            _ => assert!(false)
        }
    }
    #[rstest]
    fn ser_and_deserialize_fixed_precision_2() {
        let mut ser = Biseri::new();

        let v = FixPrecisionMinMax::<20, -50, 50>::new(-12.3456);
        v.bit_serialize(&mut ser);

        let r = ser.finish_add_data().unwrap();
        let (bits, bytes) = (r.total_bits, r.total_bytes);
        println!("bits: {}, bytes: {}", bits, bytes);

        let mut der = Bides::from_biseri(&ser);
        let vv = FixPrecisionMinMax::<20, -50, 50>::bit_deserialize(1, &mut der);

        println!("v: {:?}, vv: {:?}", v, vv);
        assert!(vv.is_some());
        let vv = vv.unwrap().0;

        let eps = 1e-1;
        match v.val {
            FixPrecisionVal::Value(fpv) => {
                match vv.val {
                    FixPrecisionVal::Value(fpvv) => assert!((fpv - fpvv).abs() < eps),
                    _ => assert!(false)
                }
            },
            _ => assert!(false)
        }
    }
    #[rstest]
    fn ser_and_deserialize_fixed_precision_under() {
        let mut ser = Biseri::new();

        let v = FixPrecisionMinMax::<10, -50, 50>::new(-60.0);
        v.bit_serialize(&mut ser);

        let r = ser.finish_add_data().unwrap();
        let (bits, bytes) = (r.total_bits, r.total_bytes);
        println!("bits: {}, bytes: {}", bits, bytes);

        let mut der = Bides::from_biseri(&ser);
        let vv = FixPrecisionMinMax::<10, -50, 50>::bit_deserialize(1, &mut der);

        println!("v: {:?}, vv: {:?}", v, vv);
        assert!(vv.is_some());
        let vv = vv.unwrap().0;

        match vv.val {
            FixPrecisionVal::Underflow => assert!(true),
            _ => assert!(false),
        }
    }
    #[rstest]
    fn ser_and_deserialize_fixed_precision_over() {
        let mut ser = Biseri::new();

        let v = FixPrecisionMinMax::<10, -50, 50>::new(60.0);
        v.bit_serialize(&mut ser);

        let r = ser.finish_add_data().unwrap();
        let (bits, bytes) = (r.total_bits, r.total_bytes);
        println!("bits: {}, bytes: {}", bits, bytes);

        let mut der = Bides::from_biseri(&ser);
        let vv = FixPrecisionMinMax::<10, -50, 50>::bit_deserialize(1, &mut der);

        println!("v: {:?}, vv: {:?}", v, vv);
        assert!(vv.is_some());
        let vv = vv.unwrap().0;

        match vv.val {
            FixPrecisionVal::Overflow => assert!(true),
            _ => assert!(false)
        }
    }

    #[rstest]
    fn ser_and_deserialize_fixed_precision_vals() {
        type ValT = FixPrecisionMinMax<3, 1, 2>;

        {
            let v = ValT::new(1.);
            let vv = deserialize::<ValT>(&serialize(&v).0);
            println!("v: {:?}, vv: {:?}", v, vv);
            assert!(vv.is_some());
            let vv = vv.unwrap().0;
            assert!(v==vv);
        }
        {
            let v = ValT::new(1.2);
            let vv = deserialize::<ValT>(&serialize(&v).0);
            println!("v: {:?}, vv: {:?}", v, vv);
            assert!(vv.is_some());
            let vv = vv.unwrap().0;
            assert!(v==vv);
        }
        {
            let v = ValT::new(1.21);
            let vv = deserialize::<ValT>(&serialize(&v).0);
            println!("v: {:?}, vv: {:?}", v, vv);
            assert!(vv.is_some());
            let vv = vv.unwrap().0;
            assert!(v==vv);
        }
    }
}

