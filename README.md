# bitis
This library/compiler is currently in an early **alpha version**! 

<u>Bit</u>w<u>is</u>e data serialization is a Rust library with Python interface and a C++ implementation for bit-level serialization and deserialization, designed to be lightweight and efficient. It provides support for messages, enums, and various numeric types, including integers, floating points, dynamic-sized integers, and fixed-point numbers.
Key Features

* Bitwise Serialization & Deserialization: Encodes data structures into a compact bitstream representation.
* Space efficient encoding:
  * Optional values not set require 1 bit
  * Dynamic arrays with no elements require 1 bit
  * Dynamic integers with a value of zero requires 1 bit
* Supports Messages & Enums: Allows encoding structured messages and discriminated unions (enums).
  * Variety of Numeric Types:
    * Any size of fixed-size integers, e.g. 3 bit wide, based on standard integers (u8, u16, u32, u64, i8, i16, etc.).
    * Floating-point numbers (f32, f64).
    * Variable-length integers: Similar to varint encoding for reducing storage of small values.
    * Fixed-precision numbers: Efficient representation of decimals using integer-based encoding.
  * Bit-Packed Representation: Minimizes overhead by eliminating byte boundaries even with nested messages.
  * All values can be optional
  * Flexible and fixed arrays
  * Compiler to generate code to serialize and deserialize messages for
    * c++
    * rust
    * python
  * Python code uses Maturin and PyO3 to interface and compile python to rust code for given messages.
  * C++ code uses a header only library.

## Example
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

## Messages

Messages are a set of attributes that are serialized together. The format is
```
msg MSG_NAME {
  {modifier} {type} ATTRIBUTE_1_NAME;
  ...
  {modifier} {type} ATTRIBUTE_N_NAME;
}
```
with MSG_NAME the name of the message, {modifier} and {type} as described in the next sections. The {type} can also be another message.

An example can be found at the end of this section. 

## Base Types

| Title                 | Bitis Type | Example | Description                                |
|-----------------------|------------|---|--------------------------------------------|
| Boolean               |  bool      |            | A value that has the values true or false. |
| Fixed Integer         | int_X<br>uint_X | uint_3 | An sigened (int_X) or unsigned (uint_X) integer value with a fixed number of bits given by a number after the underscore. Signed values need an additional bit that denotes whether the value is negative. <br>Example details: uint_3 allows unsigned integer values with three bits, e.g. values from 0 to including 7. |
| Dynamic Integer       | int_XdY<br>uint_XdY | uint_8d3 | An sigened (int_XdY) or unsigned (uint_Xd>) integer value with a dynamic number of bits depending on the value. A value of zero requires one bit. Depending on the number of bits used in the value, packets of Y bits are serialized. For each packet an additional bit is required to mark wheter further packets are encoded.<br>Example details: a value of 10 for an uint_3d is encoded as follows:  bit0=1 (value bigger than zero), bit1-4=2 (lower three bits of the value), bit5=1 (futher bits are nonzero), bit6-9=1 (bit 3-6 of the value), bit10=0 (higher bits are zero) |
| Floating point number | double<br>float | float   | A floating point variable with 64 bit (double) or 32 bit (float) with the number of bits for exponent and mantisse according to IEC-754 .|
| Fixed point number    | fp_X[Y,Z]  | fp_10[-2,3] | A number with X bits equally spaced on the closed interval [Y, Z]. There is an indication for over- or under-flow condition<br>The example denotes the values between -2 and 3 with an 10 bit number. | 
| Binary                | binary     |  |  | 

## Enumerated type 
Enumerated types are defined on the top level of a bitis definition file. The format is 
```
enum ENUM_NAME(Y) {
  E1, E2, ...
};
```
where ENUM_NAME denotes the name of the enum type and E1, E2, ... the set of named elements. The value is encoded as dynamic integer with u32 packed into chuncks of Y bits.
Emums can be used in messages like any other type.

Example:
```
enum SENSOR_SOURCE(3) {
  TEMPERATUR_SENSOR, MOVEMENT_SENSOR, PRESSURE_SENSOR
};
```

## Container Modifier

To denote arrays and optional values, modifiers can be used.

| Title           | Bitis Modifier | Example | Description |
|-----------------|----------------|---|---|
| Optional field | optional | optional uint_3 val; | If the keyword optional is added to an message, the value is optional. |
| Fixed array | repeated_fixed_X | repeated_fixed_10 uint_3 val; | Fixed array with X elements. |
| Dynamic array | repeated_dyn_X |  repeated_dyn_3 uint_3 val; | Variable sized array. It uses dynamic integer (see above) with an 32 bit that stores the size in chuncks of X bits. |

## OneOfs
Messages can contain fields that can hold one of a list of different types:
```
oneof ONEOF_NAME(X) {
  {type_1} {oneof_1};
  ...
  {type_N} {oneof_N};
}
```
with ONEOF_NAME, the name of the field amd a list of alternatives with types {type_1} ... {type_N} and the respective oneof names {oneof_1}.

An example message could be
```
msg Command_Shutdown {
  uint_7 wait_time;
}
enum SENSOR_SOURCE(3) {
  TEMPERATUR_SENSOR, MOVEMENT_SENSOR, PRESSURE_SENSOR
};
msg Command_StartMeasurement {
  SENSOR_SOURCE source;
}
msg MainMessage {
  uint_10 device_id;

  oneof command(4) {
    Command_Shutdown shutdown;
    Command_StartMeasurement start_measurement;
    uint_3 some_number;
  }
}
```

## Comments
Comments are denoted by //
```
// This is a comment
```

## Example messages

Nested messages:
```
msg Nested1 {
  uint_3 value;
}
msg Nested2 {
  uint_3 value;
}
msg MainMessage {
  Nested1 first_nested_msg;
  optional Nested2 other_nested_msg;
}
```
...

# Bitis compiler

The bitis compiler compiles bitis messages to 

* rust
* python or
* c++
 
code. 

```text
Bitwise serialization of messages defined in a data description language with interfaces for rust, python and c++.

Usage: bitis [OPTIONS] --input-file <FILE> <COMMAND>

Commands:
test     Test bitis data objects file
compile  Compile bitis data objects file
compare  Compare bitis data objects file
setup    Setup directory and file structures
help     Print this message or the help of the given subcommand(s)

Options:
-i, --input-file <FILE>  Sets a custom config file
-d, --debug...           Turn debugging information on
-h, --help               Print help
-V, --version            Print version
```


