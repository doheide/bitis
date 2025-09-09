use std::collections::HashMap;
use std::process::{abort};
use askama::Template;
use logos::{Lexer, Logos, Span};
use regex::Regex;
use stringcase::Caser;


// ************************************************************************
// ************************************************************************
fn integer_bit_size(bit_size: &u8) -> u8 {
    match bit_size {
        0..=16 => 16,
        // 0..=8 => 8,
        // 9..=16 => 16,
        17..=32 => 32,
        _ => 64
    }
}
// ************************************************************************
#[derive(Debug, Clone )]
#[allow(unused)]
pub struct AttributeEx {
    base: Attribute,
    rust_type_str: String,
    // rust_pyo3_str: String,
    base_type_str: String,
    is_py_wrapped: bool, is_msg: bool, is_enum: bool, is_oo: bool, add_val: bool,
}
#[derive(Debug, Clone )]
pub struct MessageR {
    pub name: String,
    // pub version_info: VersionInfo,
    pub comment: Option<String>,
    pub parent: Option<String>,
    pub attributes: Vec<AttributeEx>,
}
#[derive(Debug, Clone )]
#[allow(dead_code)]
pub struct OneOfInfoR {
    msg_name: String,
    name: String,
    dyn_bits: u8,
    attributes: Vec<AttributeEx>,
    default_attrib_name: String,
}
fn to_rust_attribute(attribute: &Attribute, msg_names: &Vec<String>) -> AttributeEx {
    let (rtype, base_type, is_py_wrapped, is_msg, is_enum, is_oo, add_val) = {
        let mut is_py_wrapped = false;
        let mut is_enum = false;
        let mut is_msg = false;
        let mut is_oo = false;
        let mut add_val = false;

        let (rtype, base_type) = match &attribute.specific_details {
            AttributeDetails::AttributeSimple(a) => {
                match a {
                    SimpleType::NoTypeSetYet => {
                        println!("Unexpected unspecified attribute type");
                        abort()
                    },
                    SimpleType::Bool => { ("bool".to_string(), "bool".to_string()) },
                    SimpleType::UIntFixed(b) => {
                        add_val = true;
                        let base = format!("u{}", integer_bit_size(&b));
                        (format!("IntWithGivenBitSize<{}, {}>", base.clone(), b), base) },
                    SimpleType::IntFixed(b) => {
                        add_val = true;
                        let base = format!("i{}", integer_bit_size(&b));
                        (format!("IntWithGivenBitSize<{}, {}>", base.clone(), b), base) },
                    SimpleType::UIntDyn(b) => {
                        add_val = true;
                        let base = format!("u{}", integer_bit_size(&b.0));
                        (format!("DynInteger<{}, {}, {}>", base.clone(), b.0, b.1), base) },
                    SimpleType::IntDyn(b) => {
                        add_val = true;
                        let base = format!("i{}", integer_bit_size(&b.0));
                        (format!("DynInteger<{}, {}, {}>", base.clone(), b.0, b.1), base) },
                    SimpleType::Float => {
                        add_val = true;
                        let base = "f32".to_string();
                        (base.clone(), base)
                    },
                    SimpleType::Double => {
                        let base = "f64".to_string();
                        (base.clone(), base) },
                    SimpleType::FixedPrecision(fpp) => {
                        add_val = true;
                        (format!("FixPrecisionMinMax<{}, {}, {}>", fpp.bits, fpp.min_val, fpp.max_val), "f64".to_string())
                    },
                    SimpleType::Binary(b) => {
                        add_val = true;
                        (format!("Binary<{}>", b), "Vec<u8>".to_string()) },
                    SimpleType::AString(b) => {
                        add_val = true;
                        (format!("BitisAString<{}>", b), "String".to_string()) },
                }
            }
            AttributeDetails::AttributeEnumOrMsg(em) => {
                is_py_wrapped = true;
                is_msg = msg_names.contains(&em);
                is_enum = !is_msg.clone();
                (em.clone(), em.clone()) }
            AttributeDetails::AttributeOneOf(ooi) => {
                is_py_wrapped=true; is_oo = true;
                (ooi.name.clone(), ooi.name.clone()) }
        };
        (rtype, base_type, is_py_wrapped, is_msg, is_enum, is_oo,  add_val)
    };
    AttributeEx{base: attribute.clone(), rust_type_str: rtype, base_type_str: base_type,
        is_py_wrapped, is_msg, is_enum, is_oo, add_val }
}

pub fn to_rust_messages(msgs: &Vec<Message>) -> Vec<MessageR> {
    let msgs_names: Vec<_> = msgs.iter().map(|m| {m.name.clone()}).collect();

    msgs.iter().map(|msg| {
        let attrs_rust: Vec<_> = msg.attributes.iter().map(|attribute| {
            to_rust_attribute(attribute, &msgs_names) }).collect();
        MessageR{name: msg.name.clone(), comment: msg.comment.clone(), parent: msg.parent.clone(),
            attributes: attrs_rust}
    }).collect()
}
pub fn to_rust_oneofs(oos: &Vec<(String, OneOfInfo)>, msgs: &Vec<Message>) -> HashMap<String, OneOfInfoR> {
    let msgs_names: Vec<_> = msgs.iter().map(|m| {m.name.clone()}).collect();

    oos.iter().map(|(msg_name, oo)| {
        let attrs_rust: Vec<_> = oo.attributes.iter().map(|attribute| {
            to_rust_attribute(attribute, &msgs_names) }).collect();
        (oo.name.clone(), OneOfInfoR{msg_name: msg_name.clone(), name: oo.name.clone(), dyn_bits: oo.dyn_bits,
            attributes: attrs_rust, default_attrib_name: oo.default_attrib_name.clone()})
    }).collect()
}

fn to_cpp_attribute(attribute: &Attribute, msg_names: &Vec<String>) -> AttributeEx {
    let (rtype, base_type, is_py_wrapped, is_msg, is_enum, is_oo, add_val) = {
        let mut is_py_wrapped = false;
        let mut is_enum = false;
        let mut is_msg = false;
        let mut is_oo = false;
        let mut add_val = false;

        let (rtype, base_type) = match &attribute.specific_details {
            AttributeDetails::AttributeSimple(a) => {
                match a {
                    SimpleType::NoTypeSetYet => {
                        println!("Unexpected unspecified attribute type");
                        abort()
                    },
                    SimpleType::Bool => { ("BitisBool".to_string(), "bool".to_string()) }
                    SimpleType::UIntFixed(b) => {
                        add_val = true;
                        let base = format!("uint{}_t", integer_bit_size(&b));
                        (format!("IntgralWithGivenBitSize<{}, {}>", base.clone(), b), base) }
                    SimpleType::IntFixed(b) => {
                        add_val = true;
                        let base = format!("int{}_t", integer_bit_size(&b));
                        (format!("IntgralWithGivenBitSize<{}, {}>", base.clone(), b), base) }
                    SimpleType::UIntDyn(b) => {
                        add_val = true;
                        let base = format!("uint{}_t", integer_bit_size(&b.0));
                        (format!("DynInteger<{}, {}, {}>", base.clone(), b.0, b.1), base) }
                    SimpleType::IntDyn(b) => {
                        add_val = true;
                        let base = format!("int{}_t", integer_bit_size(&b.0));
                        (format!("DynInteger<{}, {}, {}>", base.clone(), b.0, b.1), base) }
                    SimpleType::Float => {
                        add_val = true;
                        let base = "float".to_string();
                        (format!("BitisFloatingPoint<{}>", base.clone()), base)
                    }
                    SimpleType::Double => {
                        let base = "double".to_string();
                        (format!("BitisFloatingPoint<{}>", base.clone()), base) },
                    SimpleType::FixedPrecision(fpp) => {
                        add_val = true;
                        (format!("FixPrecisionMinMax<{}, {}, {}>", fpp.bits, fpp.min_val, fpp.max_val), "double".to_string())
                    },
                    SimpleType::Binary(b) => {
                        add_val = true;
                        (format!("Binary<{}>", b), "Vec<u8>".to_string()) },
                    SimpleType::AString(b) => {
                        add_val = true;
                        (format!("BitisAString<{}>", b), "char *".to_string()) },
                }
            }
            AttributeDetails::AttributeEnumOrMsg(em) => {
                is_py_wrapped = true;
                is_msg = msg_names.contains(&em);
                is_enum = !is_msg.clone();
                (em.clone(), em.clone()) }
            AttributeDetails::AttributeOneOf(ooi) => {
                is_py_wrapped=true; is_oo = true;
                (ooi.name.clone(), ooi.name.clone()) }
        };
        (rtype, base_type, is_py_wrapped, is_msg, is_enum, is_oo,  add_val)
    };
    AttributeEx{base: attribute.clone(), rust_type_str: rtype, base_type_str: base_type,
        is_py_wrapped, is_msg, is_enum, is_oo, add_val }
}
pub fn to_cpp_messages(msgs: &Vec<Message>) -> Vec<MessageR> {
    let msgs_names: Vec<_> = msgs.iter().map(|m| {m.name.clone()}).collect();

    msgs.iter().map(|msg| {
        let attrs_rust: Vec<_> = msg.attributes.iter().map(|attribute| {
            to_cpp_attribute(attribute, &msgs_names) }).collect();
        MessageR{name: msg.name.clone(), comment: msg.comment.clone(), parent: msg.parent.clone(),
            attributes: attrs_rust}
    }).collect()
}
pub fn to_cpp_oneofs(oos: &Vec<(String, OneOfInfo)>, msgs: &Vec<Message>) -> HashMap<String, OneOfInfoR> {
    let msgs_names: Vec<_> = msgs.iter().map(|m| {m.name.clone()}).collect();

    oos.iter().map(|(msg_name, oo)| {
        let attrs_cpp: Vec<_> = oo.attributes.iter().map(|attribute| {
            to_cpp_attribute(attribute, &msgs_names) }).collect();
        (oo.name.clone(), OneOfInfoR{msg_name: msg_name.clone(), name: oo.name.clone(), dyn_bits: oo.dyn_bits.clone(),
            attributes: attrs_cpp, default_attrib_name: oo.default_attrib_name.clone() })
    }).collect()
}

#[derive(Clone, Debug)]
pub struct JinjaData {
    pub enums: Vec<Enum>,
    pub msgs: Vec<MessageR>,
    pub oos: HashMap<String, OneOfInfoR>,
}

#[derive(Template, Clone, Debug)]
#[template(path = "data_objects.rs.jinja")]
pub struct RustDataObjects {
    pub d: JinjaData
}
#[derive(Template, Clone, Debug)]
#[template(path = "pyclasses.py.rs.jinja")]
pub struct RustPyDataObjects {
    pub d: JinjaData
}
#[derive(Template, Clone, Debug)]
#[template(path = "pylib.py.rs.jinja")]
pub struct RustPyLib {
    pub d: JinjaData,
    pub lib_name: String
}
#[derive(Template, Clone, Debug)]
#[template(path = "py_type_hints.pyi.jinja")]
pub struct PyTypeHints {
    pub d: JinjaData
}
#[derive(Template, Clone, Debug)]
#[template(path = "data_objects.cpp.jinja")]
pub struct CppDataObjects {
    pub d: JinjaData,
    pub object_order: Vec<String>,
    pub bitis_header_lib_file_name: String,
    pub bitis_version: String,
}


mod filters {
    #[allow(dead_code)]
    pub fn snake_case<T: std::fmt::Display>(s: T, _: &dyn askama::Values) -> ::askama::Result<String> {
        Ok(stringcase::snake_case(s.to_string().as_str()))
    }
    #[allow(dead_code)]
    pub fn camel_case<T: std::fmt::Display>(s: T, _: &dyn askama::Values,) -> ::askama::Result<String> {
        Ok(stringcase::camel_case(s.to_string().as_str()))
    }
    #[allow(dead_code)]
    pub fn pascal_case<T: std::fmt::Display>(s: T, _: &dyn askama::Values,) -> ::askama::Result<String> {
        Ok(stringcase::pascal_case(s.to_string().as_str()))
    }
    #[allow(dead_code)]
    pub fn to_py_type<T: std::fmt::Display>(s: T, _: &dyn askama::Values,) -> ::askama::Result<String> {
        if ["u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64"].contains(&s.to_string().as_str()) {
            Ok("int".to_string()) }
        else if ["f32", "f64"].contains(&s.to_string().as_str()) {
            Ok("float".to_string())
        }
        else { Ok(s.to_string()) }
    }
}

// ************************************************************************
type Error = (String, Span);

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum DynOrFixedType {
    Dyn(u8),
    Fixed(u8)
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct FixedPrecisionProperties {
    bits: u8, min_val: i64, max_val: i64
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SimpleType {
    NoTypeSetYet,
    Bool,
    UIntFixed(u8), IntFixed(u8),
    UIntDyn((u8,u8)), IntDyn((u8,u8)),
    Float, Double,
    FixedPrecision(FixedPrecisionProperties),
    Binary(u8),
    AString(u8)
}
/*impl SimpleType {
    fn int_size(min_bits: u8) -> std::result::Result<u8, String> {
        match min_bits {
            1..=8 => Ok(8),
            9..=16 => Ok(16),
            17..=32 => Ok(32),
            33..=64 => Ok(64),
            34..=128 => Ok(128),
            0 => Err("Bitsize of zero for integer is not allowed".into()),
            _ => Err("Bitsize larger than 128  for integer are not allowed".into())
        }
    }
    fn get_int_bits(self) -> std::result::Result<u8, String> {
        match self {
            SimpleType::UIntFixed(s) => Ok(SimpleType::int_size(s)?),
            SimpleType::IntFixed(s) => Ok(SimpleType::int_size(s)?),
            SimpleType::UIntDyn((s,_)) => Ok(SimpleType::int_size(s)?),
            SimpleType::IntDyn((s,_)) => Ok(SimpleType::int_size(s)?),
            SimpleType::FixedPrecision(fps) => Ok(SimpleType::int_size(fps.bits)?),
            SimpleType::UFixedPrecision(fps) => Ok(SimpleType::int_size(fps.bits)?),
            _ => Err("get_int_bits(): Only integers types allowed.".into())
        }
    }
    fn get_int_bits_no_error(self) -> u8 {
        match self.get_int_bits() {
            Ok(bits) => bits,
            Err(e) => { println!("Error: {}", e); abort(); }
        }
    }
}*/

#[derive(Debug, Clone )]
pub struct OneOfInfo {
    name: String,
    dyn_bits: u8,
    attributes: Vec<Attribute>,
    default_attrib_name: String,
}
#[derive(Debug, Clone )]
pub enum AttributeDetails {
    AttributeSimple(SimpleType),
    AttributeEnumOrMsg(String),
    AttributeOneOf(OneOfInfo),
}
#[derive(Debug, Clone )]
pub struct Attribute {
    name: String,
    comment: Option<String>,
    is_repeated_and_size: Option<DynOrFixedType>,
    is_optional: bool,
    specific_details: AttributeDetails
}
// #[derive(Debug, Clone)]
// pub enum VersionInfo {
//     Version(u16),
//     BaseWithAllowedVersion(u16),
// }
#[derive(Debug, Clone)]
pub struct Message {
    pub name: String,
    // pub version_info: VersionInfo,
    pub comment: Option<String>,
    pub parent: Option<String>,
    pub attributes: Vec<Attribute>,
}

/// Enum information for bitis. The ids are always DynInteger with a given bit size.
#[derive(Debug, Clone)]
pub struct Enum {
    pub name: String,
    // pub version_info: VersionInfo,
    pub comment: Option<String>,
    pub bit_size: u8,
    pub values: Vec<String>,
    pub default: String
}

pub fn get_suffix_number(lex: &mut Lexer<Token>) -> Option<u8> {
    let slice = lex.slice();
    let re = Regex::new(r".*_d?([0-9]+)$").unwrap();
    let num_str = re.captures(slice)?.get(1)?;
    num_str.as_str().parse().ok()
}
pub fn get_d_suffix_numbers(lex: &mut Lexer<Token>) -> Option<(u8,u8)> {
    let slice = lex.slice();
    let re = Regex::new(r".*_([0-9]+)d([0-9]+)$").unwrap();
    let first_num_str = re.captures(slice)?.get(1)?.as_str().parse().ok()?;
    let second_num_str = re.captures(slice)?.get(2)?.as_str().parse().ok()?;
    Some((first_num_str, second_num_str))
}
pub fn get_fp_properties_number(lex: &mut Lexer<Token>) -> Option<FixedPrecisionProperties> {
    let slice = lex.slice();
    let re = Regex::new(r"fp_([0-9]+)\[ *(-?[0-9]+) *, *(-?[0-9]+) *]").unwrap();
    let bits = re.captures(slice)?.get(1)?.as_str().parse::<u8>().ok()?;
    let min_val = re.captures(slice)?.get(2)?.as_str().parse::<i64>().ok()?;
    let max_val = re.captures(slice)?.get(3)?.as_str().parse::<i64>().ok()?;
    Some(FixedPrecisionProperties {bits, min_val, max_val})
}
/*pub fn get_dyn_or_fixed_from_args(lex: &mut Lexer<Token>) -> Option<DynOrFixedType> {
    let slice = lex.slice();
    let re = Regex::new(r" *(dyn|fixed) *, *([0-9]+)").unwrap();
    let type_str = re.captures(slice)?.get(1)?.as_str();
    let bits = re.captures(slice)?.get(2)?.as_str().parse::<u8>().ok()?;
    if type_str == "dyn" {
        Some(DynOrFixedType::Dyn(bits))
    }
    else {
        Some(DynOrFixedType::Fixed(bits))
    }
}*/
pub fn get_enum_bit_size(lex: &mut Lexer<Token>) -> Option<u8> {
    let slice = lex.slice();
    let re = Regex::new(r"\( *([0-9]+) *\)").unwrap();
    let bits = re.captures(slice)?.get(1)?.as_str().parse::<u8>().ok()?;
    Some(bits)
}
pub fn get_version(lex: &mut Lexer<Token>) -> Option<u16> {
    let slice = lex.slice();
    let re = Regex::new(r"\[.* +(v[0-9]+) *]").unwrap();
    let ver_str = re.captures(slice)?.get(1)?.as_str();
    Some(ver_str.parse::<u16>().ok()?)
}

#[derive(Debug, Logos)]
#[logos(skip r"[ \t\r\n\f]+")]
#[logos(extras = u16)]
#[allow(dead_code)]
pub enum Token{
    #[regex(r"//[^\n]*\n?", priority=40)] Comment,
    #[regex(r"//\|[^\n]*\n?", |lex| lex.slice()[3..].to_owned(), priority=41)] SpecificComment(String),
    #[token("msg", priority=20)] Msg,
    #[token("enum", priority=20)] Enum,
    #[token("oneof", priority=20)] OneOf,
    #[token("{")] CBraceOpen,
    #[token("}")] CBraceClose,
    #[token("(")] BraceOpen,
    #[token(")")] BraceClose,
    #[token(":")] Colon,
    #[token(";")] SemiColon,
    #[token(",")] Comma,
    #[token("*")] Star,
    // #[token("fixed", priority=20)] FixedFlag,
    // #[token("dyn", priority=20)] DynFlag,
    #[regex(r"\[ *base +use +starting +with +v[0-9]+ *\]", get_version, priority=35)] MsgBaseInfoToken(u16),
    #[regex(r"\[ *version +v[0-9]+ *\]", get_version, priority=35)] MsgVersionToken(u16),
    // #[regex(r"\[ *base +use +starting +with +v[0-9]+ *\]", get_version, priority=35)] MsgVersionToken((MsgVersion, u16)),
    #[regex("[0-9]+", |lex| lex.slice().parse::<isize>().unwrap(), priority=1)] IntegerVal(isize),
    #[regex(r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?",
        |lex| lex.slice().parse::<f64>().unwrap(), priority=2)] Number(f64),
    #[token("bool", priority=30)] Bool,
    #[token("msg_size_type", priority=30)] MsgSizeType,
    #[regex(r"uint_[0-9]+", get_suffix_number, priority=30)] UIntFixed(u8),
    #[regex(r"int_[0-9]+", get_suffix_number, priority=30)] IntFixed(u8),
    #[regex(r"uint_[0-9]+d[0-9]+", get_d_suffix_numbers, priority=31)] UIntDyn((u8,u8)),
    #[regex(r"int_[0-9]+d[0-9]+", get_d_suffix_numbers, priority=31)] IntDyn((u8,u8)),
    #[token("float", priority=30)] Float,
    #[token("double", priority=30)] Double,
    #[regex(r"astr_d[0-9]+", get_suffix_number, priority=31)] AStr(u8),
    #[regex(r"fp_[0-9]+\[ *-?[0-9]+ *, *-?[0-9]+ *]", get_fp_properties_number, priority=30)] FixedPoint(FixedPrecisionProperties),
    // #[regex(r"ufp_[0-9]+\[ *-?[0-9]+ *, *-?[0-9]+ *]", get_fp_properties_number, priority=30)] UFixedPoint(FixedPrecisionProperties),
    #[token("binary_d[0-9]+", get_suffix_number, priority=30)] Binary(u8),
    #[regex(r"repeated_dyn_[0-9]+", get_suffix_number, priority=30)] RepeatedDyn(u8),
    #[regex(r"repeated_fixed_[0-9]+", get_suffix_number, priority=30)] RepeatedFixed(u8),
    #[token("optional", priority=30)] Optional,
    #[regex(r"[A-Za-z][A-Za-z0-9_-]+", |lex| lex.slice().to_owned(), priority=11)] StringVal(String),
    #[token("false", |_| false, priority=20)]
    #[token("true", |_| true, priority=20)] BoolVal(bool),
    #[regex(r"\( *([0-9]+) *\)", get_enum_bit_size, priority=40)] EnumDynSize(u8),
}

#[derive(Debug, Clone)]
pub enum Value {
    /// null.
    Message(Message),
    /// Enum.
    Enum(Enum)
}

macro_rules! parse_one_token {
    ($token_enum: path, $lexer: expr, $error_msg_or_empty: expr) => {
        loop {
            let rv = $lexer.next();
            if let Some(token) = rv {
                match token {
                    Ok($token_enum) => {
                        break Ok(Ok(()));
                    },
                    Ok(Token::Comment) => (),
                    _ => {
                        if let Some(err_str) = $error_msg_or_empty {
                            break Err((format!("{err_str}\nToken: {token:?}").to_owned(), $lexer.span()));
                        }
                        else {
                            break Ok(Err($lexer.span()));
                        }
                    }
                }
            }
            else {
                break Err((format!("Unexpected end or text {rv:?}").to_owned(), $lexer.span()));
            }
        }
    }
}
macro_rules! parse_one_token_with_arg {
    ($token_enum: path, $lexer: expr, $error_msg_or_empty: expr) => {
        loop {
            let rv = $lexer.next();
            if let Some(token) = rv {
                match token {
                    Ok($token_enum(s)) => {
                        break Ok(Ok(s));
                    },
                    Ok(Token::Comment) => (),
                    _ => {
                        if let Some(err_str) = $error_msg_or_empty {
                            break Err((format!("{}\nFound token: {:?}.",
                                err_str, token).to_owned(), $lexer.span()));
                        }
                        else {
                            break Ok(Err($lexer.span()));
                        }
                    }
                }
            }
            else {
                break Err((format!("Unexpected end or text {rv:?}").to_owned(), $lexer.span()));
            }
        }
    }
}

pub fn parse_root(lexer: &mut Lexer<'_, Token>) -> Result<Vec<Value>> {
    let mut list: Vec<Value> = Vec::new();
    let mut specific_comment: Option<String> = None;
    loop {
        if let Some(token) = lexer.next() {
            let rv = match token {
                Ok(Token::Msg) => Some(parse_msg(lexer, specific_comment.clone())),
                Ok(Token::Enum) => Some(parse_enum(lexer, specific_comment.clone())),
                Ok(Token::Comment) => None,
                Ok(Token::SpecificComment(s)) => {
                    specific_comment = Some(s.trim().to_string()); None },
                _ => Some(Err((format!("Unexpected token {:?}", token).to_owned(), lexer.span()))),
            };
            match rv {
                None => (),
                Some(Ok(value)) => { list.push(value); specific_comment = None; },
                Some(Err(err)) => return Err(err)
            }
        }
        else { break; }
    }
    Ok(list)
}

pub fn parse_msg(lexer: &mut Lexer<'_, Token>, comment_for_msg: Option<String>) -> Result<Value> {
    let mut attributes = Vec::new();

    let name = match parse_one_token_with_arg!(Token::StringVal, lexer, Some("Expected msg name but received:"))? {
        Ok(s) => s,
        Err(s) => { return Err(("Code should not be reached".into(), s)); }
    };

    // let version_info = if(lexer.extras == 0) {
    //     if let Some(token) = lexer.next() {
    //         match token {
    //             Ok(Token::MsgVersionToken(v)) => VersionInfo::Version(v),
    //             Ok(Token::MsgBaseInfoToken(v)) => VersionInfo::BaseWithAllowedVersion(v),
    //             Ok(_) => { return Err((format!("Unexpected token {:?} for message '{}' when expecting version info", token, name)
    //                                    .to_owned(), lexer.span())); }
    //             Err(_) => { return Err((format!("Error: Syntax error for message '{}'", name).to_owned(), lexer.span())); }
    //         }
    //     } else { return Err(("Unexpectedly did not find version information".to_owned(), lexer.span())); }
    // }
    // else { VersionInfo::Version(lexer.extras) };

    let parent = {
        let has_parent; let p;
        if let Some(token) = lexer.next() {
            match token {
                Ok(Token::Colon) => has_parent = true,
                Ok(Token::CBraceOpen) => has_parent = false,
                _ => { return Err((format!("Unexpected text for msg '{name}'.").into(), lexer.span())) },
            }
            if has_parent {
                match parse_one_token_with_arg!(Token::StringVal, lexer, Some("Expected msg name."))? {
                    Ok(s) => p = Some(s),
                    Err(s) => { return Err((format!("For msg '{name} colon found but no parent name").into(), s)); }
                };
                parse_one_token!(Token::CBraceOpen, lexer, Some(format!("Expected curly bracket open for msg '{name}'")))?.unwrap();
            }
            else {
                p = None
            }
        }
        else { return Err(("Unexpected end of file".into(), lexer.span())); }
        p
    };

    loop {
        if let Some(token) = lexer.next() {
            match token {
                Ok(Token::CBraceClose) => break,
                Ok(Token::Comment) => (),
                Ok(ctoken) => match parse_attribute(ctoken, lexer, name.clone(), false) {
                    Ok(a) => { attributes.push(a); },
                    Err(e) => { return Err(e); }
                },
                _ => { return Err((format!("Error: Unexpected text found for msg '{name}'.").into(), lexer.span())) },
            };
        }
        else { return Err(("Unexpected end of file".into(), lexer.span())); }
    }

    Ok(Value::Message(Message{name, /*version_info,*/ comment: comment_for_msg, parent, attributes}))
}

pub fn parse_attribute(last_token: Token, lexer: &mut Lexer<'_, Token>,
                       parent_name: String, attributes_for_oneof: bool) -> Result<Attribute> {
    let mut is_optional = false;
    let mut is_repeated_and_size: Option<DynOrFixedType> = None;
    let mut attr_type = SimpleType::NoTypeSetYet;
    let mut ctoken = last_token;
    let mut enum_or_msg_str = None;
    let mut oneof_infos = None;
    let lexer_span_start = lexer.span();
    let mut specific_comment: Option<String> = None;

    loop {
        match ctoken {
            Token::SpecificComment(s) => {
                specific_comment = Some(s); () },
            Token::Optional if is_repeated_and_size.is_some() =>
                return Err(("Error: Optional and repeated not allowed together".to_owned(), lexer.span())),
            Token::RepeatedFixed(_) | Token::RepeatedDyn(_) if is_optional =>
                return Err(("Error: Optional and repeated are not allowed together".to_owned(), lexer.span())),

            Token::Optional | Token::RepeatedDyn(_) | Token::RepeatedFixed(_) if attributes_for_oneof =>
                return Err(("Error: Optional and repeated are not allowed in oneof".to_owned(), lexer.span())),

            Token::Optional => is_optional = true,
            Token::RepeatedDyn(b) => is_repeated_and_size = Some(DynOrFixedType::Dyn(b)),
            Token::RepeatedFixed(b) => is_repeated_and_size = Some(DynOrFixedType::Fixed(b)),
            Token::Bool => { attr_type = SimpleType::Bool; break; },
            Token::AStr(s) => {
                attr_type = SimpleType::AString(s); break; },
            Token::UIntFixed(s) => { attr_type = SimpleType::UIntFixed(s); break; },
            Token::UIntDyn((m,s)) if m < s =>
                return Err(("Error: Unsigned dyn integer bit size of integer type must be bigger than the bit size of the package".to_owned(), lexer.span())),
            // The next condition is allowed.
            // Token::UIntDyn((m,_)) if (m & 3) != 0 =>
            //     return Err(("Error: Unsigned dyn integer bit size of integer type must be a multiple of 8".to_owned(), lexer.span())),
            Token::UIntDyn((m,s)) => { attr_type = SimpleType::UIntDyn((m, s)); break; },
            Token::IntFixed(s) => { attr_type = SimpleType::IntFixed(s); break; },
            Token::IntDyn((m,s)) if m < s =>
                return Err(("Error: Unsigned dyn integer bit size of integer type must be bigger than the bit size of the package".to_owned(), lexer.span())),
            // The next condition is allowed.
            // Token::IntDyn((m,_)) if (m & 3) != 0 =>
            //     return Err(("Error: Unsigned dyn integer bit size of integer type must be a multiple of 8".to_owned(), lexer.span())),
            Token::IntDyn((m,s)) => {
                attr_type = SimpleType::IntDyn((m,s)); break;
            },
            //Token::String => { attr_type = SimpleType::String; break; },
            Token::Float => { attr_type = SimpleType::Float; break; },
            Token::Double => { attr_type = SimpleType::Double; break; },
            Token::FixedPoint(s) => { attr_type = SimpleType::FixedPrecision(s); break; },
            Token::Binary(b) => { attr_type = SimpleType::Binary(b); break; },
            Token::StringVal(s) => { enum_or_msg_str = Some(s); break; }
            Token::OneOf if is_optional || is_repeated_and_size.is_some() =>
                return Err(("Error: Oneof is not allowed to be used with modifiers".to_owned(), lexer.span())),
            Token::OneOf => {
                oneof_infos = match parse_oneof(lexer, parent_name.clone(), specific_comment.clone(),
                                                is_repeated_and_size.clone(), is_optional.clone()) {
                    Ok(oo) => Some(oo),
                    Err(s) => { return Err(s); }
                };
                break;
            }
            _ => { return Err((format!("Error: Expected attribute type or modifier (got {ctoken:?}) when parsing msg or oneof '{parent_name}'")
                                   .to_owned(), lexer.span())); }
        }
        if let Some(token) = lexer.next() {
            match token {
                Ok(t) => ctoken = t,
                Err(_) => { return Err((format!("Error: Unexpected text found for msg '{parent_name}'.").to_owned(), lexer.span())); }
            }
        } else {
            return Err(("Unexpected end of file".to_string(), lexer.span()));
        }
    }

    let mut name= "".to_owned();
    if oneof_infos.is_none() {
        name = parse_one_token_with_arg!(
            Token::StringVal, lexer, Some(format!("Error: Expected attribute name for msg '{parent_name}' (type: {attr_type:?}/{enum_or_msg_str:?})")))?.unwrap();

        parse_one_token!(Token::SemiColon, lexer, Some(format!(
            "Error: Expected semicolon to end line of attribute '{name}' of msg or oneof '{parent_name}'")))?.unwrap();
    }
    let num_of_set_types_or_opts = vec![(attr_type != SimpleType::NoTypeSetYet), enum_or_msg_str.is_some(), oneof_infos.is_some()]
        .iter().map(|&x| if x { 1_u8 } else { 0_u8 }).sum::<u8>();
    if num_of_set_types_or_opts > 1 {
        let mut span = lexer_span_start.clone();
        span.end = lexer.span().end;
        return Err(("Error: Attribute contains inconsistent optional, simple types and messages or Enums".to_string(), span));
    }

    if let Some(oo) = oneof_infos {
        Ok(oo)
    }
    else if let Some(t) = enum_or_msg_str {
        Ok(Attribute{name, comment: specific_comment,
            is_repeated_and_size: is_repeated_and_size, is_optional,
            specific_details: AttributeDetails::AttributeEnumOrMsg(t)})
    }
    else {
        Ok(Attribute{name, comment: specific_comment,
            is_repeated_and_size: is_repeated_and_size, is_optional,
            specific_details: AttributeDetails::AttributeSimple(attr_type)})
    }
}

pub fn parse_oneof(lexer: &mut Lexer<'_, Token>, parent_name: String, comment: Option<String>,
                   is_repeated_and_size: Option<DynOrFixedType>, is_optional: bool) -> Result<Attribute> {
    let oo_name = parse_one_token_with_arg!(
            Token::StringVal, lexer, Some(format!("Error: Expected name for oneof in parent '{parent_name}'")))?.unwrap();

    let bit_size = match parse_one_token_with_arg!(Token::EnumDynSize, lexer, Some("Expected oneof properties for dyn size, e.g. (4)."))? {
        Ok(s) => s, Err(s) => { return Err(("Code should not be reached".into(), s)); }
    };

    parse_one_token!(Token::CBraceOpen, lexer, Some("Error: Expected open curly bracket to enclose oneof elements"))?.unwrap();

    let mut oo_attribs = Vec::new();
    let mut is_default = false; let mut default_name = None;
    loop {
        if let Some(token) = lexer.next() {
            match token {
                Ok(Token::CBraceClose) => break,
                Ok(Token::Star) => { is_default = true; },
                Ok(last_token) => {
                    let oo_attr = match parse_attribute(last_token, lexer, oo_name.clone(), true) {
                        Ok(o) => o,
                        Err(s) => return Err(s),
                    };
                    oo_attribs.push(oo_attr.clone());

                    if is_default {
                        if default_name.is_some() {
                            return Err((format!("Error: Multiple attributes of one-of '{}' (in '{}') are marked as default.",
                                                oo_name, parent_name), lexer.span())); }
                        default_name = Some(oo_attr.name); is_default = false;
                    }
                }
                Err(_) => { return Err((format!("Error: Unexpected text when decoding oneof ({token:?})").to_owned(), lexer.span())); },
            }
        }
    }

    if default_name.is_none() {
        return Err((format!("Error: No default name in oneof elements for '{}' in '{}'",
                            oo_name, parent_name), lexer.span()));
    }

    Ok(Attribute{name: oo_name.clone(), comment,
        is_repeated_and_size: is_repeated_and_size, is_optional,
        specific_details: AttributeDetails::AttributeOneOf(OneOfInfo{
            name: format!("OO_{}_{}", parent_name.to_pascal_case(), oo_name.to_pascal_case()),
            dyn_bits: bit_size, attributes: oo_attribs,
            default_attrib_name: default_name.unwrap(),
        })})
}

pub fn parse_enum(lexer: &mut Lexer<'_, Token>, comment: Option<String>) -> Result<Value> {
    let name = match parse_one_token_with_arg!(Token::StringVal, lexer, Some("Expected msg name but received."))? {
        Ok(s) => s, Err(s) => { return Err(("Code should not be reached".into(), s)); }
    };

    let bit_size = match parse_one_token_with_arg!(Token::EnumDynSize, lexer, Some("Expected enum properties for dyn size, e.g. (4)."))? {
        Ok(s) => s, Err(s) => { return Err(("Code should not be reached".into(), s)); }
    };

    // let version_info = if(lexer.extras == 0) {
    //     if let Some(token) = lexer.next() {
    //         match token {
    //             Ok(Token::MsgVersionToken(v)) => VersionInfo::Version(v),
    //             Ok(Token::MsgBaseInfoToken(v)) => VersionInfo::BaseWithAllowedVersion(v),
    //             Ok(_) => { return Err((format!("Unexpected token {:?} for enum '{}' when expecting version info", token, name)
    //                                        .to_owned(), lexer.span())); }
    //             Err(_) => { return Err((format!("Error: Syntax error for enum '{}'", name).to_owned(), lexer.span())); }
    //         }
    //     } else { return Err(("Unexpectedly did not find version information".to_owned(), lexer.span())); }
    // }
    // else { VersionInfo::Version(lexer.extras) };

    parse_one_token!(Token::CBraceOpen, lexer, Some(format!("Expected open curly bracket for enum '{name}'")))?.unwrap();

    let mut values = Vec::new();
    let (mut value_found, mut is_default) = (false, false);
    let mut default_val = None;
    loop {
        if let Some(token) = lexer.next() {
            match token {
                Ok(Token::CBraceClose) => break,
                Ok(Token::StringVal(s)) => {
                    values.push(s.clone()); value_found=true;
                    if is_default { default_val = Some(s); }
                    is_default = false;
                },
                Ok(Token::Comma) => {
                    if !value_found { return Err(("Error: found comma but no enum value.".to_string(), lexer.span())) }
                    value_found=false },
                Ok(Token::Comment) => (),
                Ok(Token::Star) => {
                    if default_val.is_some() || value_found {
                        return Err(("Error: found default value marker (*) but default already set or marker not preceding value.".to_string(), lexer.span()))}
                    is_default = true;
                }
                _ => { return Err((format!("Error: Unexpected text found for enum '{name}'."), lexer.span())) },
            }
        } else { return Err(("Unexpected end of file".into(), lexer.span())); }
    }
    if default_val.is_none() { return Err(("Error: missing default value marker.".to_string(), lexer.span())); }

    Ok(Value::Enum(Enum{name, comment, bit_size, values, default: default_val.unwrap() }))
}


/*pub fn validate_bitis(parsed_bitis: &Vec<Value>) -> Option<String> {
    let enum_types = parsed_bitis.iter().filter_map(|x|
        match x { Value::Enum(ev) => Some(ev.name.clone()), _ => None }).collect::<Vec<String>>();
    let msg_types = parsed_bitis.iter().filter_map(|x|
        match x { Value::Message(msg) => Some(msg.name.clone()), _ => None }).collect::<Vec<String>>();

    // ***
    if let Some (err_str) = parsed_bitis.iter().find_map(|s| match s {
        Value::Message(msg) => {
            msg.attributes.iter().find_map(|a| match a.specific_details.clone() {
                AttributeDetails::AttributeEnumOrMsg(eon) => {
                    if !enum_types.contains(&eon) && !msg_types.contains(&eon) {
                        Some(format!("Type or enum '{eon}' unknown"))
                    }
                    else { None }
                },
                _ => None
            })
        },
        _ => None,
    }) { return Some(err_str); };

    // *** check msg versions
    let msgs_with_version: Vec<_> = parsed_bitis.iter().filter_map(|s| match s {
        Value::Message(msg) => {
            Some(match msg.version {
                MsgVersion::Fixed => format!("{}_fixed", msg.name),
                // MsgVersion::VersionedMsg => format!("{}_versioned", msg.name),
                // MsgVersion::Base => format!("{}_base", msg.name),
                MsgVersion::Versioned(v) => format!("{}_v{}", msg.name, v)
            })
        },
        _ => None,
    }).collect();
    match (1..msgs_with_version.len()).into_iter().find_map(|i| {
        let s = &msgs_with_version[i - 1];
        if msgs_with_version[i..].contains(s) { Some(s.clone()) } else { None }
    })
    {
        Some(msg) => return Some(format!("Conflicting versions of {}.", msg)),
        None => ()
    };

    let fixed_msgs: Vec<_> = parsed_bitis.iter().filter_map(|s| match s {
        Value::Message(msg) => {
            match msg.version.clone() {
                MsgVersion::Fixed => Some(msg.name.clone()), _ => None }
        }, _ => None,
    }).collect();
    if let Some (err_str) = parsed_bitis.iter().find_map(|s| match s {
        Value::Message(msg) => {
            match msg.version.clone() {
                MsgVersion::Fixed => None,
                _ => {
                    if fixed_msgs.contains(&msg.name) {
                        Some(format!("Multiple conflicting versions of {} (fixed and version).", msg.name))
                    }
                    else { None }
                }
            }
        }, _ => None,
    }) { return Some(err_str); };

    // ***
    // Check that only attributes were added
    // parsed_bitis.iter().for_each(|s| match s {
    //     Value::Message(msg) => {
    //
    //     },
    //     Value::Enum(eon) => {
    //
    //     }
    // })

    None
}
*/
// Struct that collects all bitis information
#[derive(Debug, Clone)]
pub struct Dependencies {
    pub in_deps: Vec<String>,
    pub out_deps: Vec<String>,
}
#[derive(Debug, Clone)]
pub struct BitisProcessed {
    pub max_version_number: u16,
    pub msgs: Vec<Message>,
    pub enums: Vec<Enum>,
    pub oo_enums: Vec<(String, OneOfInfo)>,
}

/// This function prepares message and enums for rendering
pub fn process_and_validate_bitis(parsed_bitis: &Vec<Value>) -> BitisProcessed {
    /*let (processed_msgs, max_msg_version_number) = {
        let msgs: Vec<_> = parsed_bitis.iter().filter_map(|v| {
            match v { Value::Message(msg) => Some(msg.clone()), _ => None }
        }).collect();

        let max_version_number: u16 = msgs.iter().fold(0_u16, |cur_max, v| {
            std::cmp::max(cur_max, match v.version_info.clone()
            { VersionInfo::BaseWithAllowedVersion(_) => 0, VersionInfo::Version(v) => v })
        });
        println!("Max version number for msgs found: {}", max_version_number);

        // ***
        // sort msgs per versions
        let msgs_per_version: HashMap<u16, HashMap<String, Message>> = (0..=max_version_number).map(|cver| {
            let msgs: HashMap<String, Message> = msgs.iter().filter_map(|cmsg| {
                match &cmsg.version_info {
                    VersionInfo::BaseWithAllowedVersion(_) if cver==0  => Some(cmsg.clone()),
                    VersionInfo::Version(msg_ver) => {
                        if *msg_ver == 0 {
                            println!("Error: Message '{}' has version zero which is not allowed", cmsg.name);
                            abort();
                        } else if *msg_ver == cver { Some(cmsg.clone()) }
                        else { None }
                    }, _ => None
                } }
            ).map(|msg| { (msg.name.clone(), msg) }).collect();
            (cver, msgs)
        }).collect();

        // todo check that messages as attributes are used with the correct version

        // todo check that messages have the same allowed_to_be_used_starting_with_version for all versions

        // todo check that attributes for different versions are unique

        // ***
        let msg_names_and_ver_type: HashMap<_, _> = msgs.iter().map(|v| {
            (v.name.clone(), v.version_info.clone()) }).collect();

        let msg_version_to_use_per_version: HashMap<String, HashMap<u16, u16>> = {
            let mut temp_msg_last_version: HashMap<String, u16> =
                msgs.iter().enumerate().map(|(x1, x2)| { (x2.name.clone(), x1.clone() as u16) }).collect();

            msgs.iter().map(|v| {
                (1..=max_version_number).map(|cver| {

                }
            }
        }


        // let msg_base_with_version: Vec<_> = msgs.iter().filter_map(|msg| {
        // });

        let msgs_processed: Vec<Vec<Message>> = msgs_per_version.iter()
            .map(|(&cver, cver_msgs) | {
                let mut msgs_for_version: HashMap<String, Message> = HashMap::new();

                println!("Processing V{} msgs: {:?}", cver, cver_msgs);
                if cver == 0 {
                    msg_names_and_ver_type.iter().for_each(|(mi_name, _)| {
                        if !cver_msgs.contains_key(mi_name) {
                            println!("Error: Message '{}' not found in base version. All messages must be declared in base.", mi_name);
                            abort();
                        }
                    });
                }
                else {
                    // add missing msg definitions for each version
                    let new_msgs: HashMap<_, _> = msg_names_and_ver_type.iter().filter_map(|(mi_name, ver_type)| {
                        // do it only for versioned msgs
                        if let VersionInfo::BaseWithAllowedVersion(_) = ver_type { None } else {
                            // check if
                            if !cver_msgs.contains_key(mi_name) {
                                println!("Generating empty version msg '{mi_name}'");

                                let base_ver_msg = msgs_per_version.get(&0).unwrap().get(mi_name).unwrap();

                                let name = format!("{}_DataV{}", mi_name, cver);
                                Some((name.clone(), Message { name, attributes: vec![], comment: Some("Automatically generated empty msg".to_string()),
                                    ..base_ver_msg.clone() }))
                            } else { None }
                        }
                    }).collect();
                    msgs_for_version.extend(new_msgs);
                }

                cver_msgs.iter().for_each(|(_, cmsg)| {
                    if let VersionInfo::Version(msg_ver) = &cmsg.version_info {
                        match cver {
                            cver_iter if cver_iter >= *msg_ver => {
                                let cname = format!("{}_DataV{}", cmsg.name, cver_iter);
                                msgs_for_version.insert(cname.clone(), Message { name: cname, ..cmsg.clone() });
                            },
                            _ => ()
                        };
                    }
                    else {
                        let processed_attributes: Vec<_> = cmsg.attributes.iter().map(|attr| {
                            match &attr.specific_details {
                                AttributeDetails::AttributeSimple(_) => attr.clone(),
                                AttributeDetails::AttributeEnumOrMsg(at) => {
                                    Attribute{specific_details: AttributeEnumOrMsg(format!("{}_V{}", at, cver)), ..attr.clone()}
                                }
                                AttributeDetails::AttributeOneOf(_) => {
                                    Attribute{name: format!("{}_TODO", attr.name), ..attr.clone()}
                                }
                            }
                        }).collect();
                        let cname = format!("{}_BaseV{}", cmsg.name, cver);
                        msgs_for_version.insert(cname.clone(), Message { name: cname, attributes: processed_attributes, ..cmsg.clone() });
                    }
                });
                msgs_for_version.values().cloned().collect::<Vec<Message>>()
            }).collect();

        let mut msg_processed_concat: Vec<_> = msgs_processed.concat();
        msg_processed_concat.sort_by_key(|msg| { msg.name.to_lowercase() });


        (msg_processed_concat, max_version_number)
    };*/

    //
    let msgs: Vec<_> = parsed_bitis.iter().filter_map(|v| {
        match v { Value::Message(msg) => Some(msg.clone()), _ => None }
    }).collect();
    let enums: Vec<_> = parsed_bitis.iter().filter_map(|v| {
        match v { Value::Enum(enm) => Some(enm.clone()), _ => None }
    }).collect();

    fn get_oneofs(msg_name: String, attrs: &Vec<Attribute>) -> Option<Vec<(String, OneOfInfo)>> {
        let direct_oos = attrs.iter().filter_map(|attr| {
            match &attr.specific_details {
                AttributeDetails::AttributeOneOf(oo) => Some(vec![(msg_name.clone(), oo.clone())]),
                _ => None
            }
        }).collect::<Vec<Vec<(String, OneOfInfo)>>>().concat();

        let inner_oos = direct_oos.iter().filter_map(|(_, doo)| {
            get_oneofs(msg_name.clone(), &doo.attributes)
        }).collect::<Vec<Vec<_>>>().concat();

        let all_oos = vec![direct_oos, inner_oos].concat();
        if all_oos.len() == 0 { None }
        else { Some(all_oos) }
    }
    let oo_enums: Vec<_> = msgs.iter().filter_map(|msg| {
        get_oneofs(msg.name.clone(), &msg.attributes)
    }).collect::<Vec<_>>().concat();

    // println!("\noo_enums:\n{:?}\n", oo_enums);

    { // Test msgs and enum
        let msg_names = msgs.iter().map(|msg| &msg.name).collect::<Vec<_>>();
        msg_names.iter().for_each(|name| {
            // println!("name: {}", name);
            if msg_names.iter().filter(|cname| **cname == *name).count() > 1 {
                println!("Error: Multiple instances of msg '{}' found.", name);
                abort()
            }
        });
        let enum_names = enums.iter().map(|enm| &enm.name).collect::<Vec<_>>();
        enum_names.iter().for_each(|name| {
            if enum_names.iter().filter(|cname| **cname == *name).count() > 1 {
                println!("Error: Multiple instances of enum '{}' found.", name); abort()
            }
        });

        let enums_and_msg_names = [&msg_names[..], &enum_names[..]].concat();

        // check that all attributes are defined
        msgs.iter().for_each(|msg| {
            for attr in msg.attributes.clone() {
                match attr.specific_details {
                    AttributeDetails::AttributeEnumOrMsg(enum_or_msg) => {
                        let em_found = enums_and_msg_names.contains(&&enum_or_msg);
                        if !em_found {
                            println!("!!! Error: Attribute '{}' type '{}' in message '{}' not defined.", attr.name, enum_or_msg, msg.name);
                            panic!("->Exiting");
                        }
                    },
                    _ => {}
                }
            }
        });
    }

    {
        println!("FixedPoint summary:");
        msgs.iter().for_each(|msg| {
            for attr in msg.attributes.clone() {
                match attr.specific_details {
                    AttributeDetails::AttributeSimple(asi) => match asi {
                        SimpleType::FixedPrecision(fpp) => {
                            let values_number = 1_u64 << fpp.bits;
                            let prec = (fpp.max_val - fpp.min_val) as f64 / values_number as f64;
                            println!("{}::{} => [{} : {}] precision: {}", msg.name, attr.name,
                                fpp.min_val, fpp.max_val, prec);
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        });
    }

    BitisProcessed { max_version_number: 0, msgs, enums, oo_enums}
}

pub fn dependencies_process(jd: JinjaData) -> Vec<String>{
    let mut dependencies = HashMap::new();

    for msgs in jd.msgs.clone() {
        dependencies.insert(
            msgs.name.clone(), Dependencies{in_deps: vec![], out_deps: vec![]});
    }
    for enm in jd.enums {
        dependencies.insert(
            enm.name.clone(), Dependencies{in_deps: vec![], out_deps: vec![]});
    }
    for (_, oos) in jd.oos.clone() {
        dependencies.insert(
            oos.name.clone(), Dependencies{in_deps: vec![], out_deps: vec![]});
    }

    println!("{:?}", dependencies);

    if dependencies.len() == 0 {
        println!("No dependencies found, skipping message and type analysis!");
        return Vec::new();
    }

    // msgs
    for msgs in jd.msgs {
        for attr in msgs.attributes {
            if attr.is_enum || attr.is_msg || attr.is_oo {
                println!("attr '{}' -> attr.rust_type_str: {}", attr.base.name, attr.rust_type_str);
                println!("{:?}", attr);
                dependencies.get_mut(&attr.rust_type_str).unwrap().out_deps.push(msgs.name.clone());
                dependencies.get_mut(&msgs.name).unwrap().in_deps.push(attr.rust_type_str.clone());
            }
        }
    }
    for (_, msgs) in jd.oos {
        for attr in msgs.attributes {
            if attr.is_enum || attr.is_msg || attr.is_oo {
                dependencies.get_mut(&attr.rust_type_str).unwrap().out_deps.push(msgs.name.clone());
                dependencies.get_mut(&msgs.name).unwrap().in_deps.push(attr.rust_type_str.clone());
            }
        }
    }
    println!("{:#?}", dependencies.clone());

    let mut object_order = Vec::new();
    while dependencies.len() > 0 {
        let mut cobjs: Vec<_> = dependencies.clone().iter().filter_map(|(obj_name, deps)| {
            if deps.in_deps.len() == 0 { Some(obj_name.clone()) }
            else { None }
        }).collect();

        for co in cobjs.clone() {
            dependencies.remove(&co);
        }
        for co in cobjs.clone() {
            for (_, deps) in &mut dependencies {
                deps.in_deps.retain(|x| *x != co);
            }
        }
        object_order.append(&mut cobjs);
    }
    println!("{:?}", object_order);

    object_order
}
// ***************************************************

#[cfg(test)]
mod bitis_semantic {
    use rstest::rstest;
    use super::*;

    #[rstest]
    fn msg_empty_msg() {
        let test_empty_msg = "msg Lala { }";

        let mut lexer = Token::lexer(test_empty_msg);
        lexer.extras = 0;

        let parsed_bitis = parse_root(&mut lexer);
        if let Err(s) = parsed_bitis.clone() {
            panic!("Error: {} ('{}' ,span: {:?})", s.0, &test_empty_msg[s.1.clone()], s.1);
        }
        assert_eq!(parsed_bitis.is_ok(), true);

        let parsed_bitis = parsed_bitis.unwrap();
        assert_eq!(parsed_bitis.len(), 1);

        assert!(if let Value::Message(_) = parsed_bitis[0].clone() { true } else { false });

        if let Value::Message(msg) = parsed_bitis[0].clone() {
            assert_eq!(msg.name, "Lala".to_string());
        }

        // let validate_result = validate_bitis(&parsed_bitis);
        // println!("validate_result: {:?}", validate_result);

        let process_result = process_and_validate_bitis(&parsed_bitis);
        println!("process_result {:?}", process_result);

        assert_eq!(process_result.msgs.len(), 1);
        assert_eq!(process_result.enums.len(), 0);
    }

    #[rstest]
    fn msg_simple_msg() {
        let test_empty_msg = "msg Lala { uint_7 a1; }";

        let mut lexer = Token::lexer(test_empty_msg);
        lexer.extras = 0;

        let parsed_bitis = parse_root(&mut lexer);
        if let Err(s) = parsed_bitis.clone() {
            panic!("Error: {} ('{}' ,span: {:?})", s.0, &test_empty_msg[s.1.clone()], s.1);
        }
        assert_eq!(parsed_bitis.is_ok(), true);

        let parsed_bitis = parsed_bitis.unwrap();
        assert_eq!(parsed_bitis.len(), 1);

        if let Value::Message(msg) = parsed_bitis[0].clone() {
            assert_eq!(msg.attributes.len(), 1);
            assert_eq!(msg.attributes[0].name, "a1".to_string());
            if let AttributeDetails::AttributeSimple(s) = msg.attributes[0].specific_details.clone() {
                assert_eq!(s, SimpleType::UIntFixed(7));
            }
            else { assert!(false, "Attribute type must be AttributeSimple."); }
        }
        else { assert!(false, "Value must be a message."); }
    }

    #[rstest]
    fn msg_simple_enum() {
        let test_empty_msg = "enum Lala(4) { *one, two }";

        let mut lexer = Token::lexer(test_empty_msg);
        lexer.extras = 0;

        let parsed_bitis = parse_root(&mut lexer);
        if let Err(s) = parsed_bitis.clone() {
            panic!("Error: {} ('{}' ,span: {:?})", s.0, &test_empty_msg[s.1.clone()], s.1);
        }
        assert_eq!(parsed_bitis.is_ok(), true);

        let parsed_bitis = parsed_bitis.unwrap();
        assert_eq!(parsed_bitis.len(), 1);

        if let Value::Enum(enm) = parsed_bitis[0].clone() {
            assert_eq!(enm.values.len(), 2);
            assert_eq!(enm.values[0], "one".to_string());
            assert_eq!(enm.values[1], "two".to_string());
        }
        else { assert!(false, "Value must be a message."); }
    }


    /*#[rstest]
        #[case::float("float", SimpleType::Float)]
        #[case::uint_12("uint_12", SimpleType::UIntFixed(12))]
        #[case::uint_32d4("uint_32d4", SimpleType::UIntDyn((32,4)))]
        #[case::bool("bool", SimpleType::Bool)]
        fn msg_various_attribute_types(#[case] attr_type_str: String, #[case] attr_type: SimpleType) {
            let test_msg = format!("msg Lala [fixed] {{ {attr_type_str} attr; }}");

            let mut lexer = Token::lexer(test_msg.as_str());
            lexer.extras = 0;

            let parsed_bitis = parse_root(&mut lexer);
            assert_eq!(parsed_bitis.is_ok(), true);

            let parsed_bitis = parsed_bitis.unwrap();
            assert_eq!(parsed_bitis.len(), 1);

            assert!(if let Value::Message(_) = parsed_bitis[0].clone() { true } else { false });

            if let Value::Message(msg) = parsed_bitis[0].clone() {
                assert_eq!(msg.name, "Lala".to_string());

                assert_eq!(msg.attributes.len(), 1);
                assert_eq!(msg.attributes[0].name, "attr".to_string());

                assert!(if let AttributeDetails::AttributeSimple(_) = msg.attributes[0].specific_details.clone() { true } else { false });

                if let AttributeDetails::AttributeSimple(at) = msg.attributes[0].specific_details.clone() {
                    assert_eq!(at, attr_type);
                }
            }

            let validate_result = validate_bitis(&parsed_bitis);
            println!("validate_result: {:?}", validate_result);
            assert!(validate_result.is_none());
        }*/
}

#[cfg(test)]
mod bitis_generate_rust {
    use rstest::rstest;
    use super::*;

    const HEADER: &str = "use bitis_lib::*;\n\n";
    const ENUMS_HEADER: &str = "// Enums\n";
    const OO_HEADER: &str = "// Enums for oneof\n";
    const MSG_HEADER: &str = "// Messages\n";
    const PER_ENUM_HEADER: &str = "#[derive(BiserdiEnum, Debug, Clone, PartialEq)]\n#[biserdi_enum_id_dynbits(3)]\n#[allow(nonstandard_style)]\n";
    const PER_OO_HEADER: &str = "#[derive(BiserdiOneOf, Debug, Clone, PartialEq)]\n#[biserdi_enum_id_dynbits(3)]\n#[allow(nonstandard_style)]\n";
    const PER_MSG_HEADER: &str = "#[derive(BiserdiMsg, Debug, Clone, PartialEq)]\n#[allow(nonstandard_style)]\n";

    #[rstest]
    #[ignore]
    fn msg_empty_msg() {
        let test_empty_msg = "msg Lala { }";

        let mut lexer = Token::lexer(test_empty_msg);
        lexer.extras = 0;

        let parsed_bitis = parse_root(&mut lexer);
        if let Err(s) = parsed_bitis.clone() {
            panic!("Error: {} ('{}' ,span: {:?})", s.0, &test_empty_msg[s.1.clone()], s.1);
        }
        assert_eq!(parsed_bitis.is_ok(), true);

        let parsed_bitis = parsed_bitis.unwrap();
        assert_eq!(parsed_bitis.len(), 1);

        let processed_bitis = process_and_validate_bitis(&parsed_bitis);
        let rdo = RustDataObjects{ d: JinjaData{
            enums: processed_bitis.enums,
            msgs: to_rust_messages(&processed_bitis.msgs),
            oos: to_rust_oneofs(&processed_bitis.oo_enums, &processed_bitis.msgs) } };

        let rendered = rdo.render().unwrap();
        let lala_empty = "pub struct Lala {\n}\n";
        assert_eq!(rendered, (HEADER.to_owned() + ENUMS_HEADER + "\n\n" + OO_HEADER + "\n\n"  +
            MSG_HEADER + PER_MSG_HEADER +lala_empty).to_string());
    }

    #[rstest]
    #[ignore]
    fn msg_simple_msg() {
        let test_empty_msg = "//| comment for Lala\nmsg Lala { int_5 a1; repeated_fixed_4 bool bool_array; }";
        println!("Input code:\n{}", test_empty_msg);

        let mut lexer = Token::lexer(test_empty_msg);
        lexer.extras = 0;

        let parsed_bitis = parse_root(&mut lexer);
        if let Err(s) = parsed_bitis.clone() {
            panic!("Error: {} ('{}' ,span: {:?})", s.0, &test_empty_msg[s.1.clone()], s.1);
        }
        assert_eq!(parsed_bitis.is_ok(), true);

        let parsed_bitis = parsed_bitis.unwrap();
        assert_eq!(parsed_bitis.len(), 1);

        let processed_bitis = process_and_validate_bitis(&parsed_bitis);
        let rdo = RustDataObjects{ d: JinjaData{
            enums: processed_bitis.enums, msgs: to_rust_messages(&processed_bitis.msgs),
            oos: to_rust_oneofs(&processed_bitis.oo_enums, &processed_bitis.msgs) } };

        let rendered = rdo.render().unwrap();
        let lala_commment = "/// comment for Lala\n";
        let lala_empty = "pub struct Lala {\n  pub a1: IntWithGivenBitSize<i8, 5>,\n  pub bool_array: FixedArray<bool,4>,\n}\n";
        println!("rendered:\n{}",rendered);
        assert_eq!(rendered, (HEADER.to_owned() + ENUMS_HEADER + "\n\n" + OO_HEADER + "\n\n" +
            MSG_HEADER + lala_commment + PER_MSG_HEADER +lala_empty).to_string());
    }

    #[rstest]
    #[ignore]
    fn msg_simple_enum() {
        let test_enum_msg = "//| comment for Numbers\nenum Numbers(3) {\n  // Comment for One\n  One,\n  Two,\n  Three\n}";
        println!("Input code:\n{}", test_enum_msg);

        let mut lexer = Token::lexer(test_enum_msg);
        lexer.extras = 0;

        let parsed_bitis = parse_root(&mut lexer);
        if let Err(s) = parsed_bitis.clone() {
            panic!("Error: {} ('{}' ,span: {:?})", s.0, &test_enum_msg[s.1.clone()], s.1);
        }
        assert_eq!(parsed_bitis.is_ok(), true);

        let parsed_bitis = parsed_bitis.unwrap();
        assert_eq!(parsed_bitis.len(), 1);

        let processed_bitis = process_and_validate_bitis(&parsed_bitis);
        let rdo = RustDataObjects{ d: JinjaData{ enums: processed_bitis.enums,
            msgs: to_rust_messages(&processed_bitis.msgs),
            oos: to_rust_oneofs(&processed_bitis.oo_enums, &processed_bitis.msgs) } };

        let rendered = rdo.render().unwrap();
        let lala_commment = "/// comment for Numbers\n";
        let lala_enum = "pub enum Numbers {\n  One,\n  Two,\n  Three,\n}\n\n";
        println!("*rendered:\n{}",rendered);
        assert_eq!(rendered, (HEADER.to_owned() + ENUMS_HEADER + lala_commment + PER_ENUM_HEADER + lala_enum + OO_HEADER +
            "\n\n" + MSG_HEADER ).to_string());
    }

    #[rstest]
    #[ignore]
    fn msg_simple_oneof() {
        let test_enum_msg = "//| comment for Oneof\nmsg TestOO {\n  oneof oo_li(3) { uint_3 test1; float test2; }\n  bool b1;\n}";
        println!("Input code:\n{}", test_enum_msg);

        let mut lexer = Token::lexer(test_enum_msg);
        lexer.extras = 0;

        let parsed_bitis = parse_root(&mut lexer);
        if let Err(s) = parsed_bitis.clone() {
            panic!("Error: {} ('{}' ,span: {:?})", s.0, &test_enum_msg[s.1.clone()], s.1);
        }
        assert_eq!(parsed_bitis.is_ok(), true);

        let parsed_bitis = parsed_bitis.unwrap();
        assert_eq!(parsed_bitis.len(), 1);

        let processed_bitis = process_and_validate_bitis(&parsed_bitis);
        let rdo = RustDataObjects{ d: JinjaData{ enums: processed_bitis.enums,
            msgs: to_rust_messages(&processed_bitis.msgs),
            oos: to_rust_oneofs(&processed_bitis.oo_enums, &processed_bitis.msgs) } };

        let rendered = rdo.render().unwrap();
        let testoo_commment = "/// comment for Oneof\n";
        let testoo_enum = "pub enum OO_TestOo_OoLi {\n  Test1(IntWithGivenBitSize<u8, 3>),\n  Test2(f32),\n}\n\n";
        let testoo_msg = "pub struct TestOo {\n  pub oo_li: OO_TestOo_OoLi,\n  pub b1: bool,\n}\n";
        println!("*rendered:\n{}",rendered);
        assert_eq!(rendered, (HEADER.to_owned() + ENUMS_HEADER + "\n\n" + OO_HEADER + PER_OO_HEADER
            + testoo_enum + MSG_HEADER + testoo_commment + PER_MSG_HEADER + testoo_msg).to_string());
    }
}

#[cfg(test)]
mod bitis_compile {
    use std::fs;
    use std::path::Path;
    use rstest::rstest;
    use super::*;

    fn compile(content: &str) -> BitisProcessed {
        let mut lexer = Token::lexer(content);
        lexer.extras = 0;
        println!("*** content:\n{}", content);
        let bitis_parsed = match parse_root(&mut lexer) {
            Ok(v) => v,
            Err(e) => {
                let (err_str, err_span) = e.clone();
                let content_err = &content[err_span];
                println!("Error: {}\n  -> Source: '{}'", err_str, content_err);
                abort()
            }
        };
        println!("** content:\n{:?}", bitis_parsed);
        process_and_validate_bitis(&bitis_parsed)
    }
    fn render(d: JinjaData) {
        let rdo = RustDataObjects{ d: d.clone() };
        let rendered_rust = rdo.render().unwrap();
        println!("*** rendered DO:\n{}", rendered_rust);
        fs::write(Path::new("./test_data/test_py/bitis/src/messages_test.rs"), rendered_rust).expect("Unable to write file");

        let rdo = RustPyDataObjects{ d: d.clone() };
        let rendered_rust = rdo.render().unwrap();
        println!("*** rendered PyDO:\n{}", rendered_rust);
        fs::write(Path::new("./test_data/test_py/bitis/src/pyrust_test.rs"), rendered_rust).expect("Unable to write file");

        let rdo = RustPyLib{ d: d.clone(), lib_name: "bitis_msgs".into() };
        let rendered_rust = rdo.render().unwrap();
        println!("*** rendered pyLib:\n{}", rendered_rust);
        fs::write(Path::new("./test_data/test_py/bitis/src/lib_test.rs"), rendered_rust).expect("Unable to write file");

        let rdo = PyTypeHints{ d };
        let rendered_rust = rdo.render().unwrap();
        println!("*** rendered py_type_hints:\n{}", rendered_rust);
        fs::write(Path::new("./test_data/test_py/bitis/bitis_msgs/bitis_msgs.pyi"), rendered_rust).expect("Unable to write file");
    }

    #[rstest]
    #[ignore]
    fn simple_rust_py() {
        let bitis_str = "msg ParamTestSimple { uint_4 param_1; bool param_2; }";

        let bitis_processed_org = compile(bitis_str);

        let bitis_processed = bitis_processed_org.clone();
        let d = JinjaData{
            enums: bitis_processed.enums,
            msgs: to_rust_messages(&bitis_processed.msgs),
            oos: to_rust_oneofs(&bitis_processed.oo_enums, &bitis_processed.msgs)
        };
        render(d);
    }

    #[rstest]
    #[ignore]
    fn nested_rust_py() {
        let bitis_str = "msg Inner { uint_2 val; }\nmsg ParamTestWithInner { uint_4 param_1; bool param_2; Inner inner; }";

        let bitis_processed_org = compile(bitis_str);

        let bitis_processed = bitis_processed_org.clone();

        let d = JinjaData{
            enums: bitis_processed.enums,
            msgs: to_rust_messages(&bitis_processed.msgs),
            oos: to_rust_oneofs(&bitis_processed.oo_enums, &bitis_processed.msgs)
        };
        render(d);
    }
    #[test]
    #[ignore]
    fn nested_and_enum_rust_py() {
        let bitis_str = [
            "enum Numbers(4) { one, two, three, four }\n/// Test comment for Inner\nmsg Inner { uint_3 val; Numbers num; }\n",
            "msg ParamTestWithInner { uint_4 param_1; bool param_2; Inner inner; } }"
        ].join("");

        let bitis_processed_org = compile(bitis_str.as_str());

        let bitis_processed = bitis_processed_org.clone();

        let d = JinjaData{
            enums: bitis_processed.enums,
            msgs: to_rust_messages(&bitis_processed.msgs),
            oos: to_rust_oneofs(&bitis_processed.oo_enums, &bitis_processed.msgs)
        };
        render(d);
    }
    #[rstest]
    #[ignore]
    fn oneof_nested_and_enum_rust_py() {
        let bitis_str = [
            "//| Test comment for Enum\nenum Numbers(4) { one, two, three, four }\n\n//| Test comment for Inner\nmsg Inner { uint_3 val; Numbers num; }\n",
            "msg ParamTestWithInner { uint_4 param_1; bool param_2; oneof action(4) { Inner inner; uint_3 val; } }"
        ].join("");

        let bitis_processed_org = compile(bitis_str.as_str());

        let bitis_processed = bitis_processed_org.clone();

        let d = JinjaData{
            enums: bitis_processed.enums,
            msgs: to_rust_messages(&bitis_processed.msgs),
            oos: to_rust_oneofs(&bitis_processed.oo_enums, &bitis_processed.msgs)
        };
        render(d);
    }
}

#[cfg(test)]
mod bitis_serialization {
    // use std::fs;
    use rstest::rstest;
    use super::*;

    //noinspection DuplicatedCode
    #[rstest]
    fn msg_simple_msg_compile() {
        let test_empty_msg = "msg Lala { repeated_fixed_10 bool data_bool; uint_4 data1_uint; uint_12 data2_uint; }";

        let mut lexer = Token::lexer(test_empty_msg);
        lexer.extras = 0;

        let parsed_bitis = parse_root(&mut lexer);
        assert_eq!(parsed_bitis.is_ok(), true);

        let _parsed_bitis = parsed_bitis.unwrap();

        // let rdo = RustDataObjects {
        //     enums: parsed_bitis.iter().filter_map(|x|
        //         match x {
        //             Value::Enum(ev) => Some((ev.name.clone(), ev.clone())),
        //             _ => None
        //         })
        //         .collect::<HashMap<_, _>>(),
        //     msgs: parsed_bitis.iter().filter_map(|x|
        //         match x {
        //             Value::Message(mv) => Some((mv.name.clone(), mv.clone())),
        //             _ => None
        //         })
        //         .collect::<HashMap<_, _>>(),
        // };
        //
        // let rendered = rdo.render().unwrap();
        //
        // let current_test_simple_code = String::from(std::str::from_utf8(&fs::read("test_data/test_simple_msg.rs")
        //     .expect("Unable to read test_simple_msg.rs file")).unwrap());
        // assert_eq!(current_test_simple_code, rendered);
        //
        // let validate_result = validate_bitis(&parsed_bitis);
        // println!("validate_result: {:?}", validate_result);
        // assert!(validate_result.is_none());
    }
}

#[cfg(test)]
mod bitis_processing {
    use rstest::rstest;
    use crate::AttributeDetails::{AttributeEnumOrMsg, AttributeSimple};
    use super::*;

    #[rstest]
    #[ignore]
    fn msg_base_and_v2() {
        let bitis_values = vec![
            Value::Message(Message{
                name: "TestMsg".to_string(),
                /*version_info: VersionInfo::BaseWithAllowedVersion(0),*/
                comment: Some("This is a test".to_string()),
                parent: None,
                attributes: vec![Attribute{name: "a1".to_string(), comment: None,
                    is_repeated_and_size: None, is_optional: false,
                    specific_details: AttributeSimple(SimpleType::UIntFixed(4)),
                }],
            }),
            Value::Message(Message{
                name: "TestMsg".to_string(),
                /*version_info: VersionInfo::Version(2),*/
                comment: Some("This is a test".to_string()),
                parent: None,
                attributes: vec![Attribute{name: "a2".to_string(), comment: None,
                    is_repeated_and_size: None, is_optional: false,
                    specific_details: AttributeSimple(SimpleType::UIntFixed(4)),
                }],
            })
        ];
        let pb = process_and_validate_bitis(&bitis_values);

        assert_eq!(pb.max_version_number, 2);
        assert_eq!(pb.msgs.len(), 3);

        assert_eq!(pb.msgs[0].name, "TestMsg_Base".to_string());
        assert_eq!(pb.msgs[1].name, "TestMsg_V1".to_string());
        assert_eq!(pb.msgs[2].name, "TestMsg_V2".to_string());

        assert_eq!(pb.msgs[0].attributes.len(), 1);
        assert_eq!(pb.msgs[0].attributes.get(0).unwrap().name, "a1".to_string());
        assert_eq!(pb.msgs[1].attributes.len(), 0);
        assert_eq!(pb.msgs[2].attributes.len(), 1);
        assert_eq!(pb.msgs[2].attributes.get(0).unwrap().name, "a2".to_string());
    }

    #[rstest]
    #[ignore]
    fn msg_base_and_v2_and_add_msg() {
        let bitis_values = vec![
            Value::Message(Message{
                name: "TestMsgInner".to_string(),
                /*version_info: VersionInfo::BaseWithAllowedVersion(0),*/
                comment: Some("This is a test2".to_string()),
                parent: None,
                attributes: vec![Attribute{name: "lala".to_string(), comment: None,
                    is_repeated_and_size: None, is_optional: false,
                    specific_details: AttributeSimple(SimpleType::UIntFixed(4)),
                }],
            }),
            Value::Message(Message{
                name: "TestMsgInner".to_string(),
                /*version_info: VersionInfo::Version(1),*/
                comment: Some("This is a test2".to_string()),
                parent: None,
                attributes: vec![
                    Attribute{name: "lala".to_string(), comment: None, is_repeated_and_size: None, is_optional: false,
                        specific_details: AttributeSimple(SimpleType::UIntFixed(4)),},
                    Attribute{name: "lala2".to_string(), comment: None, is_repeated_and_size: None, is_optional: false,
                        specific_details: AttributeSimple(SimpleType::UIntFixed(3)),},
                ],
            }),
            Value::Message(Message{
                name: "TestMsg".to_string(),
                /*version_info: VersionInfo::BaseWithAllowedVersion(0),*/
                comment: Some("This is a test".to_string()),
                parent: None,
                attributes: vec![
                    Attribute{ name: "a1".to_string(), comment: None, is_repeated_and_size: None, is_optional: false,
                        specific_details: AttributeSimple(SimpleType::UIntFixed(4)) },
                    Attribute{ name: "lala_use".to_string(), comment: None, is_repeated_and_size: None, is_optional: false,
                        specific_details: AttributeEnumOrMsg("TestMsgInner".to_string()) },
                ],
            }),
            Value::Message(Message{
                name: "TestMsg".to_string(),
                /*version_info: VersionInfo::Version(2),*/
                comment: Some("This isa test".to_string()),
                parent: None,
                attributes: vec![Attribute{name: "a2".to_string(), comment: None,
                    is_repeated_and_size: None, is_optional: false,
                    specific_details: AttributeSimple(SimpleType::UIntFixed(4)),
                }],
            }),
        ];
        let pb = process_and_validate_bitis(&bitis_values);

        assert_eq!(pb.max_version_number, 2);
        assert_eq!(pb.msgs.len(), 4);

/*        assert_eq!(pb.msgs[0].name, "TestMsg_Base".to_string());
        assert_eq!(pb.msgs[1].name, "TestMsg_V1".to_string());
        assert_eq!(pb.msgs[2].name, "TestMsg_V2".to_string());
        assert_eq!(pb.msgs[3].name, "TestMsgLala".to_string());

        if let MsgVersion::Versioned(l) = pb.msgs[0].version { assert_eq!(l, 0); }
        assert_eq!(pb.msgs[0].attributes.len(), 1);
        assert_eq!(pb.msgs[0].attributes.get(0).unwrap().name, "a1".to_string());
        assert_eq!(pb.msgs[1].attributes.len(), 0);
        assert_eq!(pb.msgs[2].attributes.len(), 1);
        assert_eq!(pb.msgs[2].attributes.get(0).unwrap().name, "a2".to_string());
        if let MsgVersion::Fixed = pb.msgs[0].version { assert!(false) }
        assert_eq!(pb.msgs[3].attributes.len(), 1);
        assert_eq!(pb.msgs[3].attributes.get(0).unwrap().name, "lala".to_string());*/
    }
}

