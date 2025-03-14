# bitis
<u>Bit</u>w<u>is</u>e data serialization is a Rust library with Python interface and a C++ implementation for bit-level serialization and deserialization, designed to be lightweight and efficient. It provides support for messages, enums, and various numeric types, including integers, floating points, dynamic-sized integers, and fixed-point numbers.
Key Features

* Bitwise Serialization & Deserialization: Encodes data structures into a compact bitstream representation.
* Supports Messages & Enums: Allows encoding structured messages and discriminated unions (enums).
  * Variety of Numeric Types:
    * Any size of fixed-size integers, e.g. 3 bit wide, based on standard integers (u8, u16, u32, u64, i8, i16, etc.).
    * Floating-point numbers (f32, f64).
    * Variable-length integers: Similar to varint encoding for reducing storage of small values.
    * Fixed-precision numbers: Efficient representation of decimals using integer-based encoding.
    * Bit-Packed Representation: Minimizes overhead by eliminating byte boundaries even with nested messages.
    * All values can be optional
    * Flexible and fixed arrays

The following messages consists of 
* A one bit wide boolean.
* A three bit wide unsigned integer.
* An optional signed five bit wide integer (plus on bit for the sign). The size is either one bit if not set or 5+1+1 bits when a value is set. 
* An array of fixed three elements of three bit wide unsigned integers.

```
msg ExampleMessage {
  bool is_active;
  uint_3 value_one;
  optional int_5 signed_value;
  repeated_fixed_3 uint_3; 
}
```
This is compiled to the following rust code:
```
#[derive(BiserdiMsg, Debug, Clone, PartialEq)]
#[allow(nonstandard_style)]
pub struct ExampleMessage {
pub is_active: bool,
pub value_one: VarWithGivenBitSize<u8, 3>,
pub signed_value: Option<VarWithGivenBitSize<i8, 5>>,
pub array: [VarWithGivenBitSize<u8, 3>;3],
}
```

A short example code how serialize and deserialize the ExampleMessage:
```
let msg = ExampleMessage{
    is_active: false,
    value_one: 3.into(),
    signed_value: Some(2.into()),
    array: [1.into(), 2.into(), 3.into()],
};

// serialize
let mut ser = Biseri::new();
msg.bit_serialize(&mut ser);
let r = ser.finish_add_data().unwrap();
println!("bits: {}, bytes: {}", r.0, r.1);
println!("data: {:?}", ser.get_data());

// deserialize
let mut der = Bides::from_biseri(&ser.clone());
let data = ExampleMessage::bit_deserialize(0, &mut der);
println!("data: {:?}", data);
```
That produces the following output: 
```text
bits: 20, bytes: 3
data: [150, 136, 6]
data: Some((ExampleMessage { is_active: false, value_one: VarWithGivenBitSize { val: 3 }, signed_value: Some(VarWithGivenBitSize { val: 2 }), array: [VarWithGivenBitSize { val: 1 }, VarWithGivenBitSize { val: 2 }, VarWithGivenBitSize { val: 3 }] }, 20))
```
As can be seen, the message consists of 20 bits in total.

# Description Language

...

## Types

...

## OneOfs


## Comments



