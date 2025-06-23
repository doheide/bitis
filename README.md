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

<!-- TOC -->
* [bitis](#bitis)
  * [Example](#example)
* [Description Language](#description-language)
  * [Messages](#messages)
  * [Base Types](#base-types)
  * [Enumerated type](#enumerated-type-)
  * [Container Modifier](#container-modifier)
  * [OneOfs](#oneofs)
  * [Comments](#comments)
  * [Example messages](#example-messages)
    * [Nested Messages](#nested-messages)
    * [Key Value Map](#key-value-map)
* [Bitis compiler](#bitis-compiler)
<!-- TOC -->

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
let binary_data = serialize(msg);
println!("data: {:?}", binary_data);

// deserialize
let deserilized_data: ExampleMessage = deserialize(binary_data);
println!("deserilized_data: {:?}", deserilized_data);
```
The serialization contains 20 bits or 3 bytes and the output is: 
```text
data: [150, 136, 6]
deserilized_data: Some((ExampleMessage { is_active: false, 
    value_one: VarWithGivenBitSize { val: 3 }, signed_value: Some(VarWithGivenBitSize { val: 2 }), 
    array: [VarWithGivenBitSize { val: 1 }, VarWithGivenBitSize { val: 2 }, 
    VarWithGivenBitSize { val: 3 }] }, 20))
```

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

| Title                 | Bitis Type | Example | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
|-----------------------|------------|---|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Boolean               |  bool      |            | A value that has the values true or false.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| Fixed Integer         | int_X<br>uint_X | uint_3 | An sigened (int_X) or unsigned (uint_X) integer value with a fixed number of bits given by a number after the underscore. Signed values need an additional bit that denotes whether the value is negative. <br>Example details: uint_3 allows unsigned integer values with three bits, e.g. values from 0 to including 7.                                                                                                                                                                                                                                                              |
| Dynamic Integer       | int_XdY<br>uint_XdY | uint_8d3 | An sigened (int_XdY) or unsigned (uint_Xd>) integer value with a dynamic number of bits depending on the value. A value of zero requires one bit. Depending on the number of bits used in the value, packets of Y bits are serialized. For each packet an additional bit is required to mark wheter further packets are encoded.<br>Example details: a value of 10 for an uint_3d is encoded as follows:  bit0=1 (value bigger than zero), bit1-4=2 (lower three bits of the value), bit5=1 (futher bits are nonzero), bit6-9=1 (bit 3-6 of the value), bit10=0 (higher bits are zero) |
| Floating point number | double<br>float | float   | A floating point variable with 64 bit (double) or 32 bit (float) with the number of bits for exponent and mantisse according to IEC-754 .                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| Fixed point number    | fp_X[Y,Z]  | fp_10[-2,3] | A number with X bits equally spaced on the closed interval [Y, Z]. There is an indication for over- or under-flow condition<br>The example denotes the values between -2 and 3 with an 10 bit number.                                                                                                                                                                                                                                                                                                                                                                                  | 
| Binary                | binary     |  |                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| String  | astr_dX | astr_d4 | Ascii string with a dynamic integer as string length with chunks of X bits.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
 

## Enumerated type 
Enumerated types are defined on the top level of a bitis definition file. The format is 
```
enum ENUM_NAME(Y) {
  E1, *E2, ...
};
```
where ENUM_NAME denotes the name of the enum type and E1, E2, ... the set of named elements. 
The value is encoded as dynamic integer with u32 packed into chuncks of Y bits.
The default element of an Enum must be marked with a star in front of the value.
Enums can be used in messages like any other type.

Example:
```
enum SENSOR_SOURCE(3) {
  TEMPERATUR_SENSOR, MOVEMENT_SENSOR, *PRESSURE_SENSOR
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
  *{type_N} {oneof_N};
}
```
with ONEOF_NAME, the name of the field amd a list of alternatives with types {type_1} ... {type_N} and the respective oneof names {oneof_1}.
The default oneof is marked by a leading star ('*').

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
    *uint_3 some_number;
  }
}
```

## Comments
Comments are denoted by //
```
// This is a comment
```

## Example messages

### Nested Messages
Bitis example definition for nested messages:
```
enum SensorSource(3) {
  *TEMPERATUR_SENSOR, MOVEMENT_SENSOR
}

enum ExampleEnum(2) {
  E1, E2, *E3, E4, E5, E6, E7, E8, E9
}

msg MsgEnumOpt {
  uint_3 val;
  SensorSource param_1;
  optional ExampleEnum param_2;
}

msg MsgWithInner {
  uint_3 val;
  MsgEnumOpt imsg;
}

```
**Rust** code to set the values when initializing the message:
```
// initialize message
let msg = MsgWithInner{
    val: 2.into(),
    imsg: MsgEnumOpt{
        val: 1.into(),
        param_1: SensorSource::TemperaturSensor.into(),
        param_2: Some(ExampleEnum::E3).into(),
    },
};
```
To serialize the data, use the following code can be used:
```
let binary_data = serialize(msg);
```
Binary data can be deserialized with  
```
let deserialzed_message = deserialize::<MsgWithInner>(binary_data);
```
For **C++**, the code initialize, serialize and deserialize is
```
// Initialize message
auto inner_msg = MsgEnumOpt{
    .val = MsgEnumOpt::Val_T(1),
    .param_1 = MsgEnumOpt::Param1_T::create_enum<SensorSourceEnum::TemperaturSensor>(),
    .param_2 = MsgEnumOpt::Param2_T::create_val(
        MsgEnumOpt::Param2_T::ValT::create_enum<ExampleEnumEnum::E3>())
};
auto msg = MsgWithInner{
    .val = MsgWithInner::Val_T(2),
    .imsg = inner_msg,
};

// seralize 
auto bin_data = serialize(msg);

// deserialize
auto deserialized_msg = deserialize<MsgWithInner>(bin_data);
```

### Key Value Map
There is no data type for key value pairs. Key value pair structures can be easily defined, even
with different types for values:
```
msg MsgKVOO {
  astr_d4 key;
  oneof value(2) {
    astr_d4 str_val;
    double num_val;
    *bool bool_val;
    int_32d7 int_val;
  }
}
msg MsgKVMapOO {
  repeated_dyn_2 MsgKVOO entries;
}
```
To set and serialize key value pairs in **rust**, the following code can be used:
```
let mut msg = MsgKVMapOO::default();
msg.entries.val.push(MsgKVOO{
  key: AsciiString::from_ascii("lala").unwrap().into(),
  value: OO_MsgKvoo_Value::IntVal(312.into()),
});
msg.entries.val.push(MsgKVOO{
  key: AsciiString::from_ascii("lili").unwrap().into(),
  value: OO_MsgKvoo_Value::NumVal(0.56789.into()),
});
msg.entries.val.push(MsgKVOO{
  key: AsciiString::from_ascii("lolo").unwrap().into(),
  value: OO_MsgKvoo_Value::StrVal(AsciiString::from_ascii("val1").unwrap().into()),
});
```
To deserialize and convert the message to a hash map, us ethe following code:
```
auto deserialized_msg = deserialize<MsgWithInner>(bin_data);

let kv_map: HashMap<String, OO_MsgKvoo_Value> = msg.entries.val.iter().map(|v| {
    (v.key.get_string(), v.value.clone())
}).collect();

println!("Hash map: {}", kv_map);
```

# Usage

## Rust

Add "bitis" to cargo toml. Assuming the bitis messages is compiled into the file 'messages.rs' in the same directory as "main.rs", 
import the module and "use" the messages individually or with an asterix:
```
mod messages;
use messages::*;
```

## C++

When compiling bitis messages to C++ code, by default the single header library file "bitis_lib.h" is generated besides 
the c++ code for the messages. Writing / updating "bitis_lib.h" can be disabled with the compiler option 
"--prevent-write-bitis-header-lib". If a custom header file is to be included, the header file can be specified using
the option "--bitis-header-lib-file-name".


## Python

The python implementation is based on maturin and pyo3. The bitis messages are compiled to a python library. 
Setting up the library can be automatically done by the bitis compiler. For that a python venv is required with
the python library "Maturin" installed. How to setup and activate a venv / virtual environment can be found in python 
documentation. Assuming the venv is activated, maturin can be installed with
```bash
pip install maturin
```
If the python library containing the code for messages and it's functionality, is to be called "py_msg", the library 
can be setup with  
```bash
bitis setup -s maturin py_msg/
```


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
  compile  Test bitis data objects file Compile bitis data objects file
  setup    Compare bitis data objects file Setup directory and file structures
  help     Print this message or the help of the given subcommand(s)

Options:
  -i, --input-file <FILE>  Sets a custom config file
  -d, --debug...           Turn debugging information on
  -h, --help               Print help
  -V, --version            Print version
```

The options for the compile command are:
```text
Test bitis data objects file Compile bitis data objects file

Usage: bitis --input-file <FILE> compile [OPTIONS] --lang <LANG>

Options:
  -l, --lang <LANG>
          compile language

          Possible values:
          - rust:   use rust code
          - python
          - cpp

  -o, --output-file-or-path <OUTPUT_FILE_OR_PATH>
          output file

      --bitis-header-lib-file-name <BITIS_HEADER_LIB_FILE_NAME>
          

      --prevent-write-bitis-header-lib
          

  -h, --help
          Print help (see a summary with '-h')
```

