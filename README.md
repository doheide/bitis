# bitis
This library/compiler is currently a **beta version**! 

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
    * dart (pure Dart, also for Flutter and the web)
  * Python code uses Maturin and PyO3 to interface and compile python to rust code for given messages.
  * C++ code uses a header only library.
  * Dart code uses the pure-Dart runtime package in `dart/bitis`.

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
  * [Names and validation](#names-and-validation)
  * [Example messages](#example-messages)
    * [Nested Messages](#nested-messages)
    * [Key Value Map](#key-value-map)
* [Usage](#usage)
  * [Rust](#rust)
  * [C++](#c)
  * [Python](#python)
  * [Dart](#dart)
* [Tutorial](#tutorial)
  * [Install](#install-)
    * [Rust](#rust-1)
    * [C++](#c-1)
    * [Python](#python-1)
  * [Using generated code](#using-generated-code)
    * [Rust](#rust-2)
    * [C++](#c-2)
    * [Python](#python-2)
      * [Important Notes](#important-notes)
* [Bitis compiler](#bitis-compiler)
  * [Schema stamp](#schema-stamp)
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
  repeated_fixed_3 uint_3 array;
}
```
This is compiled to the following rust code:
```
#[derive(BiserdiMsg, Debug, Clone, PartialEq, Default)]
#[allow(nonstandard_style)]
pub struct ExampleMessage {
  pub is_active: bool,
  pub value_one: IntWithGivenBitSize<u16, 3>,
  pub signed_value: BitisOption<IntWithGivenBitSize<i16, 5>>,
  pub array: FixedArray<IntWithGivenBitSize<u16, 3>,3>,
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
with MSG_NAME the name of the message, {modifier} and {type} as described in the next sections. The {type} can also be another message
or an enum (defined anywhere in the input files). Messages must not contain themselves, directly or indirectly (no recursion).
The attributes are serialized in the given order without any padding.

A message can name a parent (`msg CHILD : PARENT { ... }`). The parent must be a defined message; it is currently
informational only, attributes are not inherited.

An example can be found at the end of this section. 

## Base Types

| Title                 | Bitis Type | Example | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
|-----------------------|------------|---|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Boolean               |  bool      |            | A value that has the values true or false.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| Fixed Integer         | int_X<br>uint_X | uint_3 | An signed (int_X) or unsigned (uint_X) integer value with a fixed number of bits X (1 ≤ X ≤ 64) given by a number after the underscore. Signed values need an additional bit that denotes whether the value is negative. <br>Example details: uint_3 allows unsigned integer values with three bits, e.g. values from 0 to including 7. |
| Dynamic Integer       | int_XdY<br>uint_XdY | uint_8d3 | An signed (int_XdY) or unsigned (uint_XdY) integer value of at most X bits (1 ≤ Y ≤ X ≤ 64) with a dynamic number of bits depending on the value. A value of zero requires one bit (two bits for int_XdY). Depending on the number of bits used in the value, packets of Y bits are serialized. For each packet an additional bit is required to mark whether further packets are encoded.<br>Example details: a value of 10 for an uint_32d3 is encoded as follows: bit0=1 (value bigger than zero), bit1-3=2 (lower three bits of the value), bit4=1 (further bits are nonzero), bit5-7=1 (bits 3-5 of the value), bit8=0 (higher bits are zero) |
| Floating point number | double<br>float | float   | A floating point variable with 64 bit (double) or 32 bit (float) with the number of bits for exponent and mantisse according to IEC-754 .                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| Fixed point number    | fp_X[Y,Z]  | fp_10[-2,3] | A number with X bits (2 ≤ X ≤ 53) equally spaced on the closed interval [Y, Z] with integers Y < Z. There is an indication for over- or under-flow condition<br>The example denotes the values between -2 and 3 with an 10 bit number. | 
| Binary                | binary_dY<br>binary_dY[M]  | binary_d4<br>binary_d4[64] | Binary data (bytes) with a dynamic integer as length with chunks of Y bits (1 ≤ Y ≤ 32). Optional maximum length M in bytes (see below). |
| String  | astr_dY<br>astr_dY[M] | astr_d4<br>astr_d4[32] | Ascii string with a dynamic integer as string length with chunks of Y bits (1 ≤ Y ≤ 32). Optional maximum length M in characters (see below). |

### Maximum lengths `[M]`

`astr_dY[M]`, `binary_dY[M]` and `repeated_dyn_Y[M] T` (see the modifiers below) limit the length to at most M
bytes, characters or elements (1 ≤ M ≤ 65535, no space between the type and `[M]` is required). The encoding is
the same as without the bound, but decoders reject a longer length (`InvalidValue`) and the checked encoders
(`try_serialize`, Dart `toBytes`) reject longer values (`ValueOutOfRange`); the legacy `serialize` writes only the
first M elements. Bounded fields have a known maximum size (`max_bits`), and the C++ runtime stores them without
heap allocation (`BoundedAString<Y, M>`, `BoundedBinary<Y, M>`, `BoundedDynArray<T, Y, M>`: `std::array` plus a
length). Rust uses `BoundedAString<Y, M>`, `BoundedBinary<Y, M>` and `BoundedDynArray<T, Y, M>` (`try_new` /
`TryFrom` check the bound), Python raises `ValueError` when a too long value is assigned, and Dart keeps
`String` / `Uint8List` / `List`. The bound is part of the schema hash. `[M]` is not allowed anywhere else.
Example: `repeated_dyn_2[8] astr_d4[16] tags;` (at most 8 tags with at most 16 characters each).
 

## Enumerated type 
Enumerated types are defined on the top level of a bitis definition file. The format is 
```
enum ENUM_NAME(Y) {
  E1, *E2, ...
}
```
where ENUM_NAME denotes the name of the enum type and E1, E2, ... the set of named elements separated by commas
(a trailing comma is allowed).
The value is the zero based index of the element, encoded as dynamic integer with u32 packed into chunks of Y bits (1 ≤ Y ≤ 32).
The default element of an Enum must be marked with a star in front of the value.
Enums can be used in messages like any other type. The generated code uses the element names in PascalCase
(e.g. `PRESSURE_SENSOR` becomes `PressureSensor`), so element names must differ after this conversion.

Example:
```
enum SENSOR_SOURCE(3) {
  TEMPERATUR_SENSOR, MOVEMENT_SENSOR, *PRESSURE_SENSOR
}
```

## Container Modifier

To denote arrays and optional values, modifiers can be used.

| Title           | Bitis Modifier | Example | Description |
|-----------------|----------------|---|---|
| Optional field | optional | optional uint_3 val; | If the keyword optional is added to an message, the value is optional. |
| Fixed array | repeated_fixed_X | repeated_fixed_10 uint_3 val; | Fixed array with X elements (1 ≤ X ≤ 255). |
| Dynamic array | repeated_dyn_X<br>repeated_dyn_X[M] |  repeated_dyn_3 uint_3 val;<br>repeated_dyn_3[10] uint_3 val; | Variable sized array. It uses a dynamic integer (see above) with 32 bit that stores the size in chunks of X bits (1 ≤ X ≤ 32). Optional maximum number of elements M (see "Maximum lengths" above). The element type must not be a message without any bits (e.g. an empty message). |

At most one modifier can be used per attribute. Modifiers can not be used for oneofs and inside oneofs.

## OneOfs
Messages can contain fields that can hold one of a list of different types:
```
oneof ONEOF_NAME(X) {
  {type_1} {oneof_1};
  ...
  *{type_N} {oneof_N};
}
```
with ONEOF_NAME, the name of the field and a list of alternatives with types {type_1} ... {type_N} and the respective oneof names {oneof_1}.
The default oneof is marked by a leading star ('*'). The index of the selected alternative is encoded as dynamic integer with
chunks of X bits (1 ≤ X ≤ 32), followed by the value of the alternative.
The generated type of the oneof is named `OO_{MessageName}_{OneofName}` (both parts in PascalCase); these names must be
unique, e.g. a oneof `value` in the messages `DTS_Msg` and `DtsMsg` is an error. In Dart the oneof is a sealed class
with this name and one subclass `OO_{MessageName}_{OneofName}_{ElementName}` (element name in PascalCase) per element.

An example message could be
```
msg Command_Shutdown {
  uint_7 wait_time;
}
enum SENSOR_SOURCE(3) {
  *TEMPERATUR_SENSOR, MOVEMENT_SENSOR, PRESSURE_SENSOR
}
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
Documentation comments are denoted by //| and are copied into the generated code (as `///` comments in Rust,
C++ and Dart, as doc string in Python). They can be placed before messages, enums, attributes, oneofs, oneof elements and enum
values; several consecutive //| lines form one multi-line comment.
```
//| A sensor reading.
//| The value is given in milli Kelvin.
msg Reading {
  //| Sensor value
  uint_20 value;
}
```

## Names and validation
Names of messages, enums, attributes, oneofs and enum elements start with a letter followed by letters, digits and
underscores (`[A-Za-z][A-Za-z0-9_]*`). The compiler rejects:
* duplicate names: message and enum names (shared namespace), attribute names within a message, oneof elements,
  enum elements, also if they only differ in the case conversion used by the generated code (e.g. `a_b` and `aB`),
* names that are keywords in Rust, C++, Python or Dart (e.g. `type`, `class`, `from`, `new`, `int`), also after case
  conversion, and attribute names that clash with generated members (e.g. `serialize`, `print`, `msg_attr`,
  `max_bits`, and in Dart `to_json`, `copy_with`, `hash_code`, or `x_fp_type` next to a fixed point attribute `x`),
* enum values whose Dart name (lowerCamelCase) is a member of the generated Dart enum (e.g. `values`, `name`, `index`,
  `default_value`, `bitis_name`), attribute names whose Dart name equals a message or enum name,
* type names reserved by the generated code / runtime (e.g. `Binary`, `String`, `BITIS_VERSION`, and for Dart
  `List`, `Map`, `Object`, `Int64`, `BigInt`, `FixedPoint`, `BitisMessage`, `bitisVersion`, built-in identifiers
  like `dynamic` or `get`),
* unknown types and parents, recursive messages, bit sizes outside the ranges given above.

Errors are reported as `file:line:col: error: message`.

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

The header is C++17 and works without exceptions and RTTI. Besides the legacy `serialize()` / `deserialize<T>()`
there is a fallible API returning a `BitisError` (see the header comment of `bitis_lib.h`):
```cpp
std::vector<uint8_t> bytes;
BitisError e = try_serialize(msg, bytes);                  // ValueOutOfRange if a value does not fit
e = try_deserialize(bytes, msg_out);                       // or try_deserialize_exact (TrailingData)

uint8_t buf[bitis_max_bytes<MsgWithInner>()];             // compile time bound, no heap for the output
BitisSize size;
e = try_serialize(msg, buf, sizeof(buf), &size);           // BufferTooSmall if buf is too small
```
`bitis_min_bits<T>()` / `bitis_max_bits<T>()` / `bitis_max_bytes<T>()` (and `T::min_bits()` / `T::max_bits()`) are
`constexpr`; types containing unbounded strings, binaries or `repeated_dyn` lists give `BITIS_UNBOUNDED`.

For small targets: define `BITIS_NO_PRINT` before including the generated header (or with `-DBITIS_NO_PRINT`) to
turn every `print()` into an empty function; no names end up in the binary and `printf` is not referenced. Use the
same setting in all translation units of a program. Fixed point values store their raw wire value (the smallest
unsigned type holding N bits, default: overflow); use `value()` / `get_value()` (0 for under- and overflow),
`state()` / `get_state()`, `has_value()`, `is_underflow()`, `is_overflow()`, `set(v)`, `set_underflow()`,
`set_overflow()` and `raw()` / `set_raw(r)`; equality compares the raw values.


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

## Dart

`bitis compile -l dart -o lib/messages.dart -i a.bitis -i b.bitis` writes ONE Dart library for all input files. It
only imports the pure-Dart runtime package `bitis` (`dart/bitis/` in this repository, see
[dart/README.md](dart/README.md) for how to depend on it) and `dart:typed_data`; it runs on the Dart VM, Flutter and
the web (dart2js and dart2wasm). Generating code does not need a Dart toolchain.

* Message: `final class Name implements BitisMessage` with `final` fields (lowerCamelCase of the attribute names),
  a constructor with named, optional parameters (omitted attributes get the default value, the constructor is
  `const` if all defaults are constants), `bitisWrite` / `static bitisRead`, `toBytes()`, `static fromBytes(data)`
  (trailing bytes are ignored), `static fromBytesExact(data)`, `toJson()` / `factory fromJson(json)`, `copyWith`
  (pass `null` to clear an optional attribute), `==` / `hashCode` (element-wise for lists and `Uint8List`, fixed
  point values compared on their wire value), `toString`, `static const bitisMinBits`, `static const bitisMaxBits`
  (spec section 4; type `int?` and `null` if the message is unbounded) and one
  `static const FixedPointType <attribute>FpType` per fixed point attribute.
* Enum: Dart `enum` with the values in lowerCamelCase in declaration order (wire index = `index`),
  `bitisName` (the name as written in the schema, used in JSON), `static defaultValue`.
* Oneof: `sealed class OO_{Msg}_{Oneof}` with one `final class OO_{Msg}_{Oneof}_{Element}` per element holding the
  `value`; switch over the subclasses is exhaustive. `static defaultValue` is the `*` element with its default value.
* Types: `bool`; `int` for integers up to 32 bits; `Int64` (package `fixnum`, exported by `bitis`) for signed
  integers with more than 32 bits and unsigned integers with 33..63 bits; `BigInt` for `uint_64` / `uint_64dY`;
  `double` for `float`/`double`; `FixedPoint` for `fp_N` (value, underflow or overflow); `String` (ASCII) for
  `astr`; `Uint8List` for `binary`; `T?` for `optional`; `List<T>` for `repeated_fixed`/`repeated_dyn`.
  Value ranges, string encodings and fixed array lengths are checked when encoding (`BitisEncodeException`), decoding
  throws `BitisDecodeException`.
* Default values are those of the Rust `Default`: 0, `false`, empty string / list / binary, `null` for optional,
  the `*` enum value / oneof element, and **overflow** for fixed point attributes.
* JSON: keys are the attribute names as written in the schema; optional attributes are omitted when absent (missing
  or `null` is absent when parsing); enums are the value name as written; oneofs `{"element": value}`; fixed point
  values are numbers in physical units or `{"state": "underflow"|"overflow"}`; integers beyond ±(2^53−1) are decimal
  strings; binary is base64; float NaN/±infinity are `"NaN"`, `"Infinity"`, `"-Infinity"`.
* Class names are the schema names as written (e.g. `DTS_Msg`); the file disables the corresponding naming lints.
  The code is laid out like `dart format` (Dart >= 3.7, page width 80) and contains `// dart format off`, so it is
  not reformatted by projects with other formatter settings.

```dart
import 'dart:convert';

import 'package:my_app/messages.dart';

final msg = DTS_Msg(
  msgs: [
    DTS_MsgWrapper(value: OO_DtsMsgWrapper_Value_Acknowledge(DTS_Msg_Acknowledge(params: true))),
  ],
  nonce: 3,
);
final bytes = msg.toBytes();
final back = DTS_Msg.fromBytesExact(bytes);
final text = jsonEncode(back.toJson());
final description = switch (back.msgs.first.value) {
  OO_DtsMsgWrapper_Value_Acknowledge(:final value) => 'ack ${value.params}',
  _ => 'other',
};
```

## CRC-32 helper

Every runtime contains the same CRC-32 helper, e.g. to protect a bitis payload in a transport framing (the framing
itself is not part of bitis, and the CRC is not part of the wire format). The CRC is **CRC-32/ISO-HDLC**: reflected
polynomial `0xEDB88320`, init `0xFFFFFFFF`, xorout `0xFFFFFFFF`, reflected input and output -- the CRC of zlib /
Python `zlib.crc32`, Ethernet, PNG and gzip. Check value `crc32("123456789") = 0xCBF43926`, `crc32("") = 0`. The
input may be split arbitrarily between `update` calls; `finish()` / `value` does not change the state.

| Runtime | One-shot | Incremental | Table |
|---|---|---|---|
| Rust | `bitis_lib::crc32(&[u8]) -> u32` | `bitis_lib::Crc32`: `new()`, `update(&[u8])`, `finish() -> u32`, `reset()` | 256 entries (1 KiB), `const fn` |
| C++ | `bitis_crc32(const uint8_t*/const void*, size_t) -> uint32_t` (constexpr for `uint8_t*`) | `BitisCrc32`: `update(ptr, len)`, `finish()`, `reset()` | 16 entries (64 bytes); 256 with `BITIS_CRC32_TABLE256` |
| Dart | `crc32(List<int>) -> int` | `Crc32`: `update(List<int>)`, `value` / `finish()`, `reset()` | 256 entries (`Uint32List`) |
| Python | `crc32(bytes) -> int` | `Crc32`: `update(bytes)`, `value` / `finish()`, `reset()` | Rust implementation |

- **C++**: header only `cpp_lib/bitis_crc32.h`, independent of `bitis_lib.h` and not written by the compiler (copy it
  or add `cpp_lib/` to the include path). C++17, no heap, exceptions or RTTI. The default 16-entry nibble table costs
  64 bytes of flash and two table lookups per byte; `#define BITIS_CRC32_TABLE256` before the include selects the
  256-entry table (1 KiB, about twice as fast). The two variants are in different inline namespaces, so translation
  units may choose differently. The names carry the `bitis_`/`Bitis` prefix of the C++ runtime (and do not collide
  with zlib's global `crc32()`).
- **Dart**: in `package:bitis`; results are unsigned (`0 .. 0xFFFFFFFF`) on the VM, dart2js and dart2wasm. Only the
  low 8 bits of each element are used.
- **Python**: `crc32` and `Crc32` are part of every module generated with `bitis compile -l python` (implemented by
  the Rust runtime); they accept `bytes`, `bytearray`, `memoryview` or a sequence of ints. Python's own
  `zlib.crc32(data)` (incrementally `zlib.crc32(data, previous)`) gives the same values. A message named `Crc32` in
  the schema would shadow the class.
- **ESP32 firmware**: the ROM function `esp_rom_crc32_le` (`esp_rom_crc.h`, also `esp_crc32_le` from `esp_crc.h`)
  computes the same CRC when called with start value **0**; it inverts the register on entry and exit itself, so the
  previous result is passed in to continue:

  ```c
  #include "esp_rom_crc.h"
  uint32_t crc = esp_rom_crc32_le(0, buf, len);       // == crc32(buf) of all bitis runtimes
  uint32_t c = 0;                                     // incremental, any split:
  c = esp_rom_crc32_le(c, part1, len1);
  c = esp_rom_crc32_le(c, part2, len2);               // c == crc32(part1 + part2)
  ```

  Do not pass `~0` / `0xFFFFFFFF` as start value and do not invert the result: `esp_rom_crc32_le(0xFFFFFFFF, ...)`
  and `~esp_rom_crc32_le(~0, ...)` give different values (for `"123456789"`: `0xD202D277` and `0x2DFD2D88` instead of
  `0xCBF43926`). On the device the ROM function is the natural choice; `bitis_crc32.h` is meant for host tools,
  tests and as a portable fallback.
- Shared test vectors: `golden/vectors/crc32.txt` (see [golden/README.md](golden/README.md)).

# Tutorial
## Install 
...
### Rust
...
### C++
...
### Python
...

## Using generated code
...
### Rust
...
### C++
...
### Python
...
#### Important Notes
...

# Bitis compiler

The bitis compiler compiles bitis messages to 

* rust
* python
* c++ or
* dart
 
code. 

```text
Bitwise serialization of messages defined in a data description language with interfaces for rust, python and c++.

Usage: bitis [OPTIONS] <COMMAND>

Commands:
  compile  Compile bitis definition files to code
  setup    Setup directory and file structures
  help     Print this message or the help of the given subcommand(s)

Options:
  -d, --debug...  Turn debugging information on (can be given multiple times)
  -h, --help      Print help
  -V, --version   Print version
```

The options for the compile command are:
```text
Compile bitis definition files to code

Usage: bitis compile [OPTIONS] --input-files <FILE>... --lang <LANG>

Options:
  -i, --input-files <FILE>...
          Input file(s) with extension .bitis; give the option once per file or list several files

  -p, --input-base-path <INPUT_BASE_PATH>
          Directory for the output file if no output file or path is given (rust, cpp and dart)

  -l, --lang <LANG>
          Language to generate code for

          Possible values:
          - rust:   generate rust code
          - python: generate python library (based on generated rust code and maturin)
          - cpp:    generate c++ code
          - dart:   generate one dart library (uses the dart package `bitis`)

  -o, --output-file-or-path <OUTPUT_FILE_OR_PATH>
          Output file or directory. rust/cpp/dart: file, or directory to write <input name>.rs/.h/.dart (messages.rs/.h/.dart for multiple input files) into; python: directory of the maturin project

      --bitis-header-lib-file-name <BITIS_HEADER_LIB_FILE_NAME>
          Name (path relative to the generated header) of the bitis header lib that is included by and written next to the generated header (cpp only, default: bitis_lib.h)

      --prevent-write-bitis-header-lib
          Do not write the bitis header lib (cpp only)

      --prevent-update-bitis-lib-in-crate
          Do not set the bitis dependency in Cargo.toml of the maturin project (python only)

  -h, --help
          Print help (see a summary with '-h')
```

Examples:
```bash
# one rust file for several input files (written to rust/src/messages.rs)
bitis compile -l rust -o rust/src -i messages_a.bitis -i messages_b.bitis
# c++ header messages.h plus bitis_lib.h next to it
bitis compile -l cpp -o messages.h -i messages.bitis
# one dart library for several input files
bitis compile -l dart -o lib/messages.dart -i messages_a.bitis -i messages_b.bitis
```
Errors are printed to stderr as `file:line:col: error: message` and the compiler exits with code 1.
Additional output (e.g. the precision of fixed point attributes and the schema hash) is printed with `-d`.

## Schema stamp
The generated code contains the version of the compiler and a 64 bit hash of the schema (see "Schema hash" in
docs/WIRE_FORMAT.md). The hash only changes if types, names, attributes or their order change, not for comments,
formatting or a different split into input files.

* Rust: `pub const BITIS_VERSION: &str` and `pub const BITIS_SCHEMA_HASH: u64` in the generated module.
* C++: `constexpr const char* BITIS_VERSION` and `constexpr uint64_t BITIS_SCHEMA_HASH` in the namespace
  `bitis_schema_<header file name>` (e.g. `bitis_schema_messages`). The same names are also defined at global
  scope by the first generated header included in a translation unit (guarded by `BITIS_SCHEMA_STAMP_DEFINED`), so
  use the namespace if several generated headers are included.
* Python: module attributes `BITIS_VERSION` (str) and `BITIS_SCHEMA_HASH` (int).
* Dart: top level `const String bitisVersion`, `const String bitisSchemaHashHex` (16 hex digits) and
  `final Int64 bitisSchemaHash` (the 64 bit pattern; import the libraries with a prefix if several generated
  libraries are used in one file).
