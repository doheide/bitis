//! Negative tests: every syntax / validation rule and every former hang of the compiler.
//! Each case runs in its own thread with a timeout guard.

use std::sync::mpsc;
use std::time::Duration;

use bitis_lib::*;

/// Compiles the sources (named t.bitis, u.bitis, ...) and returns the rendered errors (empty if ok).
fn compile_with_timeout(sources: &[&str]) -> Vec<String> {
    let files: Vec<SourceFile> = sources.iter().enumerate()
        .map(|(i, s)| SourceFile { name: format!("{}.bitis", ["t", "u", "v"][i]), content: s.to_string() })
        .collect();
    let (tx, rx) = mpsc::channel();
    let f2 = files.clone();
    std::thread::spawn(move || {
        let r = match compile_sources(&f2) {
            Ok(_) => vec![],
            Err(errors) => errors.iter().map(|e| e.render(&f2)).collect(),
        };
        let _ = tx.send(r);
    });
    match rx.recv_timeout(Duration::from_secs(10)) {
        Ok(r) => r,
        Err(_) => panic!("compiler did not terminate (hang) for {sources:?}"),
    }
}

/// Asserts that compiling `src` fails with an error that contains `expected`
/// (usually `t.bitis:line:col: error: ...`).
#[track_caller]
fn assert_error(src: &str, expected: &str) {
    let errors = compile_with_timeout(&[src]);
    assert!(errors.iter().any(|e| e.contains(expected)),
            "expected error containing {expected:?} for source {src:?}, got {errors:#?}");
}

#[track_caller]
fn assert_ok(src: &str) {
    let errors = compile_with_timeout(&[src]);
    assert!(errors.is_empty(), "expected no errors for {src:?}, got {errors:#?}");
}

// ********************************************************************
// former hangs / unterminated constructs

#[test]
fn eof_inside_oneof() {
    assert_error("msg Mm { oneof oo(2) { *bool aa;", "t.bitis:1:33: error: unexpected end of file, expected oneof element or '}'");
    assert_error("msg Mm { oneof oo(2) {", "t.bitis:1:23: error: unexpected end of file");
    assert_error("msg Mm { oneof oo(2) { *", "t.bitis:1:25: error: unexpected end of file, expected oneof element after '*'");
    assert_error("msg Mm { oneof oo(2)", "t.bitis:1:21: error: unexpected end of file, expected '{' after oneof 'oo'");
    assert_error("msg Mm { oneof oo", "error: unexpected end of file, expected dyn bit size for oneof 'oo'");
    assert_error("msg Mm { oneof", "error: unexpected end of file, expected oneof name");
}

#[test]
fn eof_in_other_constructs() {
    assert_error("msg Mm { bool aa;", "t.bitis:1:18: error: unexpected end of file, expected attribute or '}'");
    assert_error("msg Mm { bool aa", "t.bitis:1:17: error: unexpected end of file, expected ';' after attribute 'aa'");
    assert_error("msg Mm { bool", "error: unexpected end of file, expected attribute name");
    assert_error("msg Mm { optional", "error: unexpected end of file, expected attribute type");
    assert_error("msg Mm", "t.bitis:1:7: error: unexpected end of file, expected '{' or ':'");
    assert_error("msg Mm : ", "error: unexpected end of file, expected parent message name");
    assert_error("msg", "error: unexpected end of file, expected message name");
    assert_error("enum Ee(2) { *Aa,", "error: unexpected end of file, expected enum value or '}'");
    assert_error("enum Ee(2) { *Aa", "error: unexpected end of file, expected ',' or '}'");
    assert_error("enum Ee(2) { *", "error: unexpected end of file, expected enum value after '*'");
    assert_error("enum Ee(2)", "error: unexpected end of file, expected '{' after enum 'Ee'");
    assert_error("enum Ee", "error: unexpected end of file, expected dyn bit size for enum 'Ee'");
    assert_error("//| doc only\nmsg Mm {\n //| doc\n", "t.bitis:4:1: error: unexpected end of file");
}

#[test]
fn recursive_messages() {
    assert_error("msg Mm { optional Mm next; bool bb; }", "t.bitis:1:19: error: recursive message definition: Mm -> Mm");
    assert_error("msg Mm { repeated_dyn_2 Mm kids; bool bb; }", "t.bitis:1:25: error: recursive message definition: Mm -> Mm");
    assert_error("msg Aa { Bb bb; }\nmsg Bb { optional Aa aa; }", "t.bitis:2:19: error: recursive message definition: Aa -> Bb -> Aa");
    assert_error("msg Aa { Bb bb; }\nmsg Bb { Cc c; }\nmsg Cc { oneof o(1) { *bool x; Aa a; } }",
                 "t.bitis:3:32: error: recursive message definition: Aa -> Bb -> Cc -> Aa");
}

// ********************************************************************
// duplicates and name collisions

#[test]
fn duplicate_names() {
    assert_error("msg Mm { uint_3 aa; bool aa; }", "t.bitis:1:26: error: duplicate attribute 'aa' in message 'Mm' (first defined at t.bitis:1:17)");
    assert_error("enum Ee(2) { *Aa, Aa }", "t.bitis:1:19: error: duplicate enum value 'Aa' in enum 'Ee'");
    assert_error("msg Mm { bool a; }\nmsg Mm { bool b; }", "t.bitis:2:5: error: duplicate definition of type 'Mm' (first defined at t.bitis:1:5)");
    assert_error("enum Ee(2) { *Aa }\nmsg Ee { bool bb; }", "t.bitis:2:5: error: duplicate definition of type 'Ee'");
    assert_error("enum Ee(2) { *Aa }\nenum Ee(2) { *Bb }", "t.bitis:2:6: error: duplicate definition of type 'Ee'");
    assert_error("msg Mm { oneof oo(2) { *bool aa; uint_3 aa; } }", "t.bitis:1:41: error: duplicate oneof element 'aa' in oneof 'oo'");
    assert_error("msg Mm { bool oo; oneof oo(2) { *bool aa; uint_3 bb; } }", "t.bitis:1:25: error: duplicate attribute 'oo' in message 'Mm'");
}

#[test]
fn case_conversion_collisions() {
    assert_error("enum Ee(2) { *FOO_BAR, FooBar }", "t.bitis:1:24: error: enum value 'FooBar' and 'FOO_BAR' (at t.bitis:1:15) in enum 'Ee' both map to 'FooBar'");
    assert_error("msg Mm { uint_3 a_b; bool aB; }", "t.bitis:1:27: error: attribute 'aB' and 'a_b' (at t.bitis:1:17) in message 'Mm' both map to 'AB'");
    assert_error("msg Mm { oneof oo(2) { *bool a_b; uint_3 aB; } }", "t.bitis:1:42: error: oneof element 'aB' and 'a_b'");
}

#[test]
fn generated_type_name_collisions() {
    assert_error("msg DTS_Msg { oneof value(2){ *bool aa; bool bb; } }\nmsg DtsMsg { oneof value(2){ *uint_3 cc; bool dd; } }",
                 "t.bitis:2:20: error: oneof 'value' of message 'DtsMsg' clashes with oneof 'value' of message 'DTS_Msg' (at t.bitis:1:21): both generate the type name 'OO_DtsMsg_Value'");
    assert_error("msg OO_Mm_Val { bool x; }\nmsg Mm { oneof val(1) { *bool a; } }", "t.bitis:2:16: error: oneof 'val' of message 'Mm' clashes with type 'OO_Mm_Val'");
    assert_error("enum Color(2) { *Red }\nmsg ColorEnum { bool x; }", "t.bitis:2:5: error: type 'ColorEnum' clashes with C++ enum namespace of enum 'Color'");
    assert_error("msg Binary { bool x; }", "t.bitis:1:5: error: type 'Binary' uses the name 'Binary' which is reserved");
    assert_error("msg BITIS_VERSION { bool x; }", "t.bitis:1:5: error: type 'BITIS_VERSION' uses the name 'BITIS_VERSION' which is reserved");
}

#[test]
fn generated_member_collisions() {
    assert_error("msg Mm { uint_3 print; }", "t.bitis:1:17: error: attribute name 'print' clashes with a generated member of the C++ message struct");
    assert_error("msg Mm { bool msg_attr; }", "t.bitis:1:15: error: attribute name 'msg_attr' clashes");
    assert_error("msg Mm { bool serialize; }", "t.bitis:1:15: error: attribute name 'serialize' clashes");
    assert_error("msg Mm { bool to_rust; }", "t.bitis:1:15: error: attribute name 'to_rust' clashes with a generated method of the Python message class");
    assert_error("msg Mm { bool max_bits; }", "t.bitis:1:15: error: attribute name 'max_bits' clashes with a generated member of the C++ message struct");
    assert_error("msg Mm { bool min_bits; }", "t.bitis:1:15: error: attribute name 'min_bits' clashes");
    // the C++ name members str_<attr> do not exist any more (static name table since 0.11)
    assert_ok("msg Mm { bool x; bool str_x; }");
    assert_error("msg Mm { bool val_oo; oneof val(1) { *bool a; } }", "error: attribute 'val' generates the member 'val_oo' in the Python message class");
}

// ********************************************************************
// unknown types

#[test]
fn unknown_types() {
    assert_error("msg Mm { Unknown uu; }", "t.bitis:1:10: error: unknown type 'Unknown' of attribute 'uu' in message 'Mm'");
    assert_error("msg Mm { oneof oo(2) { *Unknown uu; bool bb; } }", "t.bitis:1:25: error: unknown type 'Unknown' of attribute 'uu' in oneof 'oo'");
    assert_error("msg Mm { oneof oo(2) { *bool b; oneof inner(1) { *Nope n; } } }", "t.bitis:1:51: error: unknown type 'Nope'");
    assert_error("msg Mm { binary data; }", "t.bitis:1:10: error: unknown type 'binary'");
    assert_error("msg Cc : Nope { bool bb; }", "t.bitis:1:10: error: unknown parent message 'Nope' of message 'Cc'");
    assert_error("enum Ee(1) { *A }\nmsg Cc : Ee { bool bb; }", "t.bitis:2:10: error: parent 'Ee' of message 'Cc' is an enum");
}

// ********************************************************************
// bit sizes

#[test]
fn integer_bit_sizes() {
    assert_error("msg Mm { uint_0 v; }", "t.bitis:1:10: error: bit size of uint_X must be in 1..=64, got 0");
    assert_error("msg Mm { uint_65 v; }", "error: bit size of uint_X must be in 1..=64, got 65");
    assert_error("msg Mm { uint_300 v; }", "error: bit size of uint_X must be in 1..=64, got 300");
    assert_error("msg Mm { uint_99999999999999 v; }", "error: bit size of uint_X must be in 1..=64, got a too large value");
    assert_error("msg Mm { int_0 v; }", "error: bit size of int_X must be in 1..=64, got 0");
    assert_error("msg Mm { int_65 v; }", "error: bit size of int_X must be in 1..=64, got 65");
    assert_error("msg Mm { uint_8d0 v; }", "error: chunk bit size of uint_XdY (Y) must be in 1..=8, got 0");
    assert_error("msg Mm { uint_8d9 v; }", "error: chunk bit size of uint_XdY (Y) must be in 1..=8, got 9");
    assert_error("msg Mm { int_4d8 v; }", "error: chunk bit size of int_XdY (Y) must be in 1..=4, got 8");
    assert_error("msg Mm { uint_0d0 v; }", "error: bit size of uint_XdY (X) must be in 1..=64, got 0");
    assert_error("msg Mm { int_65d3 v; }", "error: bit size of int_XdY (X) must be in 1..=64, got 65");
    assert_ok("msg Mm { uint_1 a; uint_64 b; int_1 c; int_64 d; uint_64d64 e; int_1d1 f; uint_8d1 g; }");
}

#[test]
fn dyn_bit_sizes() {
    assert_error("enum Ee(0) { *Aa, Bb }", "t.bitis:1:8: error: dyn bit size of enum 'Ee' must be in 1..=32, got 0");
    assert_error("enum Ee(33) { *Aa, Bb }", "error: dyn bit size of enum 'Ee' must be in 1..=32, got 33");
    assert_error("msg Mm { oneof oo(0) { *bool aa; } }", "t.bitis:1:18: error: dyn bit size of oneof 'oo' must be in 1..=32, got 0");
    assert_error("msg Mm { oneof oo(33) { *bool aa; } }", "error: dyn bit size of oneof 'oo' must be in 1..=32, got 33");
    assert_error("msg Mm { repeated_dyn_0 bool b; }", "t.bitis:1:10: error: dyn bit size of repeated_dyn must be in 1..=32, got 0");
    assert_error("msg Mm { repeated_dyn_33 bool b; }", "error: dyn bit size of repeated_dyn must be in 1..=32, got 33");
    assert_error("msg Mm { repeated_fixed_0 bool b; }", "error: number of elements of repeated_fixed must be in 1..=255, got 0");
    assert_error("msg Mm { repeated_fixed_256 bool b; }", "error: number of elements of repeated_fixed must be in 1..=255, got 256");
    assert_error("msg Mm { astr_d0 s; }", "error: dyn bit size of astr_dY must be in 1..=32, got 0");
    assert_error("msg Mm { astr_d33 s; }", "error: dyn bit size of astr_dY must be in 1..=32, got 33");
    assert_error("msg Mm { binary_d0 s; }", "error: dyn bit size of binary_dY must be in 1..=32, got 0");
    assert_error("msg Mm { binary_d33 s; }", "error: dyn bit size of binary_dY must be in 1..=32, got 33");
    assert_ok("enum Ee(32) { *A }\nmsg Mm { Ee e; repeated_dyn_32 bool a; repeated_fixed_255 bool b; astr_d32 c; binary_d1 d; oneof o(32) { *bool x; } }");
}

#[test]
fn fixed_point_parameters() {
    assert_error("msg Mm { fp_1[0,1] v; }", "t.bitis:1:10: error: bit size of fp_N must be in 2..=53, got 1");
    assert_error("msg Mm { fp_0[0,1] v; }", "error: bit size of fp_N must be in 2..=53, got 0");
    assert_error("msg Mm { fp_54[0,1] v; }", "error: bit size of fp_N must be in 2..=53, got 54");
    assert_error("msg Mm { fp_64[0,1] v; }", "error: bit size of fp_N must be in 2..=53, got 64");
    assert_error("msg Mm { fp_8[5,5] v; }", "error: fp_N[min,max] requires min < max, got [5,5]");
    assert_error("msg Mm { fp_8[10,1] v; }", "error: fp_N[min,max] requires min < max, got [10,1]");
    assert_error("msg Mm { fp_8[0,99999999999999999999] v; }", "error: bounds of fp_N[min,max] must fit into a 64 bit signed integer");
    assert_error("msg Mm { fp_8[-9223372036854775808,9223372036854775807] v; }", "error: range max - min of fp_N[min,max] must fit");
    assert_ok("msg Mm { fp_2[-1,1] a; fp_53[-180, 180] b; fp_10[ -2 , 3 ] c; }");
}

#[test]
fn repeated_dyn_of_zero_bit_messages() {
    assert_error("msg Empty { }\nmsg Mm { repeated_dyn_2 Empty e; }", "t.bitis:2:25: error: repeated_dyn of message 'Empty' is not allowed: 'Empty' encodes to 0 bits");
    assert_error("msg Empty { }\nmsg Wrap { Empty e; repeated_fixed_3 Empty f; }\nmsg Mm { repeated_dyn_2 Wrap e; }",
                 "t.bitis:3:25: error: repeated_dyn of message 'Wrap' is not allowed");
    assert_error("msg Empty { }\nmsg Mm { oneof o(1) { *bool b; Wrap w; } }\nmsg Wrap { repeated_dyn_1 Empty e; }",
                 "t.bitis:3:27: error: repeated_dyn of message 'Empty' is not allowed");
    // 0 bit messages are fine everywhere else, and messages with optional / dyn content have at least 1 bit
    assert_ok("msg Empty { }\nmsg Opt { optional Empty e; }\nmsg Dyn { repeated_dyn_1 bool b; }\n\
               msg Mm { Empty e; optional Empty o; repeated_fixed_2 Empty f; repeated_dyn_2 Opt a; repeated_dyn_2 Dyn d; oneof x(1) { *Empty y; } }");
}

// ********************************************************************
// identifiers and reserved words

#[test]
fn reserved_words() {
    assert_error("msg Mm { uint_3 type; }", "t.bitis:1:17: error: attribute name 'type' is a reserved word in Rust, C++, Python or Dart");
    assert_error("msg Mm { bool class; }", "error: attribute name 'class' is a reserved word");
    assert_error("msg Mm { bool from; }", "error: attribute name 'from' is a reserved word");
    assert_error("msg Mm { bool var; }", "error: attribute name 'var' is a reserved word");
    assert_error("msg Mm { bool Var; }", "error: attribute name 'Var' (as 'var') is a reserved word");
    assert_error("msg Mm { oneof Type(1) { *bool b; } }", "error: attribute name 'Type' (as 'type') is a reserved word");
    assert_ok("msg Dyn { bool b; }\nenum Type(1) { *A }");
    assert_error("enum Ee(2) { *Aa, self }", "t.bitis:1:19: error: enum value 'self' is a reserved word");
    assert_error("enum Ee(2) { *Aa, none }", "t.bitis:1:19: error: enum value 'none' (as 'None') is a reserved word");
    assert_error("msg struct { bool b; }", "t.bitis:1:5: error: message name 'struct' is a reserved word");
    assert_error("enum new(1) { *A }", "t.bitis:1:6: error: enum name 'new' is a reserved word");
    assert_error("msg Mm { oneof switch(1) { *bool b; } }", "t.bitis:1:16: error: attribute name 'switch' is a reserved word");
    assert_error("msg Mm { oneof o(1) { *bool int; } }", "t.bitis:1:29: error: oneof element name 'int' is a reserved word");
    assert_error("enum Ee(2) { *float, Bb }", "t.bitis:1:15: error: expected enum value of enum 'Ee', found type 'float'");
}

#[test]
fn identifiers() {
    assert_error("msg Mm { uint_3 my-val; }", "t.bitis:1:19: error: unexpected text '-'");
    assert_error("msg Mm { uint_3 x$; }", "t.bitis:1:18: error: unexpected text '$'");
    assert_error("msg 1Mm { }", "t.bitis:1:5: error: unexpected text '1'");
    assert_ok("msg A { uint_3 x; }\nenum E(2) { *B, C }\nmsg D { E e; A a; oneof o(1) { *bool y; } }");
}

// ********************************************************************
// syntax

#[test]
fn syntax_errors() {
    assert_error("msg Cc {\n  bool dd;\n  uint_3 ee\n  bool ff;\n}", "t.bitis:4:3: error: expected ';' after attribute 'ee', found type 'bool'");
    assert_error("enum Ee(3) { *Aa Bb }", "t.bitis:1:18: error: expected ',' or '}' after enum value in enum 'Ee', found identifier 'Bb'");
    assert_error("enum Ee(3) { *Aa, Bb };", "t.bitis:1:23: error: expected 'msg' or 'enum', found ';'");
    assert_error("enum Ee(3) { Aa, Bb }", "t.bitis:1:6: error: no default value in enum 'Ee'");
    assert_error("enum Ee(3) { *Aa, *Bb }", "t.bitis:1:19: error: multiple values of enum 'Ee' are marked as default");
    assert_error("enum Ee(3) { *Aa, , Bb }", "t.bitis:1:19: error: expected enum value of enum 'Ee', found ','");
    assert_error("msg Mm { oneof oo(2) { bool aa; } }", "t.bitis:1:16: error: no default element in oneof 'oo'");
    assert_error("msg Mm { oneof oo(2) { *bool aa; *bool bb; } }", "t.bitis:1:34: error: multiple elements of oneof 'oo' are marked as default");
    assert_error("msg Mm { oneof oo(2) { bool aa; * } }", "t.bitis:1:35: error: expected oneof element after '*'");
    assert_error("msg Mm { optional optional bool aa; }", "t.bitis:1:19: error: only one modifier");
    assert_error("msg Mm { repeated_dyn_2 repeated_fixed_3 bool aa; }", "t.bitis:1:25: error: only one modifier");
    assert_error("msg Ii { bool x; }\nmsg Mm { repeated_dyn_2 optional Ii aa; }", "t.bitis:2:25: error: only one modifier");
    assert_error("msg Mm { oneof oo(2) { *optional bool aa; } }", "t.bitis:1:25: error: modifiers (optional, repeated) are not allowed inside a oneof");
    assert_error("msg Mm { optional oneof oo(2) { *bool aa; } }", "t.bitis:1:19: error: a oneof can not be used with modifiers");
    assert_error("msg Mm [version v1] { bool bb; }", "t.bitis:1:8: error: unexpected text '['");
    assert_error("bool x;", "t.bitis:1:1: error: expected 'msg' or 'enum', found type 'bool'");
    assert_error("msg Mm { bool; }", "t.bitis:1:14: error: expected attribute name in message 'Mm', found ';'");
}

#[test]
fn accepted_syntax() {
    // trailing comma, comments everywhere, doc comments before '}' and inside enums, single letters
    assert_ok("enum E(4) {\n //| doc\n *A, // c\n B, //| dangling\n}\nmsg M {\n //| doc\n // plain\n E e;\n //| dangling\n}");
    assert_ok("msg Mm { oneof oo(2) { * // c\n bool aa; uint_3 bb; } }");
    assert_ok("msg Mm { }");
    assert_ok("");
    assert_ok("// only a comment");
    assert_ok("msg P { bool a; }\nmsg C : P { bool b; }");
}

// ********************************************************************
// error positions with several input files

#[test]
fn multiple_files_keep_file_identity() {
    let errors = compile_with_timeout(&["msg Aa { bool x; }\n", "\n\nmsg Bb { Unknown u; }\n"]);
    assert_eq!(errors, vec!["u.bitis:3:10: error: unknown type 'Unknown' of attribute 'u' in message 'Bb'".to_string()]);
    let errors = compile_with_timeout(&["msg Aa { bool x; }\n", "msg Aa { bool y; }\n"]);
    assert_eq!(errors, vec!["u.bitis:1:5: error: duplicate definition of type 'Aa' (first defined at t.bitis:1:5)".to_string()]);
    let errors = compile_with_timeout(&["msg Aa { bool x; }\n", "msg Bb {\n  bool y\n"]);
    assert_eq!(errors, vec!["u.bitis:3:1: error: unexpected end of file, expected ';' after attribute 'y'".to_string()]);
    // types defined in another file can be used
    assert!(compile_with_timeout(&["msg Aa { Bb b; }\n", "msg Bb { bool y; }\n"]).is_empty());
}

#[test]
fn several_errors_are_reported_sorted() {
    let errors = compile_with_timeout(&["msg Mm { Unknown a; uint_3 a; bool class; }"]);
    assert_eq!(errors.len(), 3, "{errors:#?}");
    assert!(errors[0].starts_with("t.bitis:1:10:"), "{errors:#?}");
    assert!(errors[1].starts_with("t.bitis:1:28:"), "{errors:#?}");
    assert!(errors[2].starts_with("t.bitis:1:36:"), "{errors:#?}");
}

#[test]
fn dart_name_collisions() {
    assert_error("msg Mm { uint_3 to_json; }",
                 "t.bitis:1:17: error: attribute name 'to_json' (Dart name 'toJson') clashes with a member of the generated Dart message class");
    assert_error("msg Mm { bool hash_code; }", "t.bitis:1:15: error: attribute name 'hash_code' (Dart name 'hashCode')");
    assert_error("enum Ee(1) { *Values }",
                 "t.bitis:1:15: error: enum value 'Values' (Dart name 'values') clashes with a member of the generated Dart enum 'Ee'");
    assert_error("enum Ee(1) { *A, default_value }", "t.bitis:1:18: error: enum value 'default_value' (Dart name 'defaultValue')");
    assert_error("msg Mm { fp_8[0, 1] a; uint_3 a_fp_type; }",
                 "t.bitis:1:31: error: attribute 'a_fp_type' generates the member 'aFpType' in the Dart message class of message 'Mm', which is already generated for attribute 'a'");
    assert_error("msg inner { bool x; }\nmsg Mm { inner inner; }",
                 "t.bitis:2:16: error: attribute name 'inner' (Dart name 'inner') clashes with the type name 'inner' in the generated Dart code");
    assert_error("msg Mm { oneof oo(1) { *bool a; } }\nmsg OO_Mm_Oo_A { bool x; }",
                 "t.bitis:2:5: error: type 'OO_Mm_Oo_A' clashes with Dart class of element 'a' of oneof 'oo' of message 'Mm'");
    for name in ["List", "Int64", "FixedPoint", "BitisMessage", "bitisVersion", "dynamic"] {
        assert_error(&format!("msg {name} {{ bool x; }}"), &format!("uses the name '{name}' which is reserved"));
    }
}
