use std::process::abort;
use askama::Template;
use logos::{Lexer, Logos, Span};
use regex::Regex;
use stringcase::Caser;

// ************************************************************************
#[derive(Template, Clone, Debug)]
#[template(path = "data_objects.rs.jinja")]
pub struct RustDataObjects {
    pub enums: Vec<Enum>,
    pub msgs: Vec<Message>,
    pub oos: Vec<OneOfInfo>,
}

mod filters {
    #[allow(dead_code)]
    pub fn camel_case<T: std::fmt::Display>(s: T) -> ::askama::Result<String> {
        Ok(stringcase::camel_case(s.to_string().as_str()))
    }
    #[allow(dead_code)]
    pub fn pascal_case<T: std::fmt::Display>(s: T) -> ::askama::Result<String> {
        Ok(stringcase::pascal_case(s.to_string().as_str()))
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
pub struct FixedPointProperties {
    bits: u8, min_val: i64, max_val: i64
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SimpleType {
    NoTypeSetYet,
    Bool,
    UIntFixed(u8), IntFixed(u8),
    UIntDyn((u8,u8)), IntDyn((u8,u8)),
    Float, Double,
    UFixedPoint(FixedPointProperties),
    FixedPoint(FixedPointProperties),
    Binary
}
impl SimpleType {
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
            SimpleType::FixedPoint(fps) => Ok(SimpleType::int_size(fps.bits)?),
            SimpleType::UFixedPoint(fps) => Ok(SimpleType::int_size(fps.bits)?),
            _ => Err("get_int_bits(): Only integers types allowed.".into())
        }
    }
    fn get_int_bits_no_error(self) -> u8 {
        match self.get_int_bits() {
            Ok(bits) => bits,
            Err(e) => { println!("Error: {}", e); abort(); }
        }
    }
}

#[derive(Debug, Clone )]
pub struct OneOfInfo {
    name: String,
    dyn_bits: u8,
    attrs: Vec<Attribute>,
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
pub fn get_fp_properties_number(lex: &mut Lexer<Token>) -> Option<FixedPointProperties> {
    let slice = lex.slice();
    let re = Regex::new(r".?fp_([0-9]+)\[ *(-?[0-9]+) *, *(-?[0-9]+) *]").unwrap();
    let bits = re.captures(slice)?.get(1)?.as_str().parse::<u8>().ok()?;
    let min_val = re.captures(slice)?.get(2)?.as_str().parse::<i64>().ok()?;
    let max_val = re.captures(slice)?.get(3)?.as_str().parse::<i64>().ok()?;
    Some(FixedPointProperties{bits, min_val, max_val})
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
    // #[token("fixed", priority=20)] FixedFlag,
    // #[token("dyn", priority=20)] DynFlag,
    #[regex(r"\[ *base +use +starting +with +v[0-9]+ *\]", get_version, priority=35)] MsgBaseInfoToken(u16),
    #[regex(r"\[ *version +v[0-9]+ *\]", get_version, priority=35)] MsgVersionToken(u16),
    // #[regex(r"\[ *base +use +starting +with +v[0-9]+ *\]", get_version, priority=35)] MsgVersionToken((MsgVersion, u16)),
    #[regex("[0-9]+", |lex| lex.slice().parse::<isize>().unwrap(), priority=1)] IntegerVal(isize),
    #[regex(r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?",
        |lex| lex.slice().parse::<f64>().unwrap(), priority=2)] Number(f64),
    #[token("bool", priority=30)] Bool,
    #[regex(r"uint_[0-9]+", get_suffix_number, priority=30)] UIntFixed(u8),
    #[regex(r"int_[0-9]+", get_suffix_number, priority=30)] IntFixed(u8),
    #[regex(r"uint_[0-9]+d[0-9]+", get_d_suffix_numbers, priority=31)] UIntDyn((u8,u8)),
    #[regex(r"int_[0-9]+d[0-9]+", get_d_suffix_numbers, priority=31)] IntDyn((u8,u8)),
    #[token("float", priority=30)] Float,
    #[token("double", priority=30)] Double,
    #[regex(r"fp_[0-9]+\[ *-?[0-9]+ *, *-?[0-9]+ *]", get_fp_properties_number, priority=30)] FixedPoint(FixedPointProperties),
    #[regex(r"ufp_[0-9]+\[ *-?[0-9]+ *, *-?[0-9]+ *]", get_fp_properties_number, priority=30)] UFixedPoint(FixedPointProperties),
    //#[token("str", priority=30)] String,
    #[token("binary", priority=30)] Binary,
    #[regex(r"repeated_dyn_[0-9]+", get_suffix_number, priority=30)] RepeatedDyn(u8),
    #[regex(r"repeated_fixed_[0-9]+", get_suffix_number, priority=30)] RepeatedFixed(u8),
    #[token("optional", priority=30)] Optional,
    #[regex(r"[A-Za-z][A-Za-z0-9_-]+", |lex| lex.slice().to_owned(), priority=11)] StringVal(String),
    #[token("false", |_| false, priority=20)]
    #[token("true", |_| true, priority=20)]
    BoolVal(bool),
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
                    specific_comment = Some(s); None },
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
            Token::UIntFixed(s) => { attr_type = SimpleType::UIntFixed(s); break; },
            Token::UIntDyn((m,s)) => { attr_type = SimpleType::UIntDyn((m, s)); break; },
            Token::IntFixed(s) => { attr_type = SimpleType::IntFixed(s); break; },
            Token::IntDyn((m,s)) => { attr_type = SimpleType::IntDyn((m,s)); break; },
            //Token::String => { attr_type = SimpleType::String; break; },
            Token::Float => { attr_type = SimpleType::Float; break; },
            Token::Double => { attr_type = SimpleType::Double; break; },
            Token::FixedPoint(s) => { attr_type = SimpleType::FixedPoint(s); break; },
            Token::UFixedPoint(s) => { attr_type = SimpleType::UFixedPoint(s); break; },
            Token::Binary => { attr_type = SimpleType::Binary; break; },
            Token::StringVal(s) => { enum_or_msg_str = Some(s); break; }
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
        Ok(Attribute{name, comment: specific_comment, is_repeated_and_size, is_optional,
            specific_details: AttributeDetails::AttributeEnumOrMsg(t)})
    }
    else {
        Ok(Attribute{name, comment: specific_comment, is_repeated_and_size, is_optional,
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
    loop {
        if let Some(token) = lexer.next() {
            match token {
                Ok(Token::CBraceClose) => break,
                Ok(last_token) => {
                    match parse_attribute(last_token, lexer, oo_name.clone(), true) {
                        Ok(o) => oo_attribs.push(o),
                        Err(s) => return Err(s),
                    }
                }
                Err(_) => { return Err((format!("Error: Unexpected text when decoding oneof ({token:?})").to_owned(), lexer.span())); },
            }
        }
    }
    Ok(Attribute{name: oo_name.clone(), comment, is_repeated_and_size, is_optional,
        specific_details: AttributeDetails::AttributeOneOf(OneOfInfo{
            name: format!("OO_{}_{}", parent_name.to_pascal_case(), oo_name.to_pascal_case()),
            dyn_bits: bit_size, attrs: oo_attribs})})
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
    loop {
        if let Some(token) = lexer.next() {
            match token {
                Ok(Token::CBraceClose) => break,
                Ok(Token::StringVal(s)) => values.push(s),
                Ok(Token::Comma) | Ok(Token::Comment)=> (),
                _ => { return Err((format!("Error: Unexpected text found for enum '{name}'.").into(), lexer.span())) },
            }
        } else { return Err(("Unexpected end of file".into(), lexer.span())); }
    }

    Ok(Value::Enum(Enum{name, /*version_info,*/ comment, bit_size, values}))
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
#[derive(Debug)]
pub struct BitisProcessed {
    pub max_version_number: u16,
    pub msgs: Vec<Message>,
    pub enums: Vec<Enum>,
    pub oo_enums: Vec<OneOfInfo>,
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

    fn get_oneofs(attrs: &Vec<Attribute>) -> Option<Vec<OneOfInfo>> {
        let direct_oos = attrs.iter().filter_map(|attr| {
            match &attr.specific_details {
                AttributeDetails::AttributeOneOf(oo) => Some(vec![oo.clone()]),
                _ => None
            }
        }).collect::<Vec<Vec<OneOfInfo>>>().concat();

        let inner_oos = direct_oos.iter().filter_map(|doo| {
            get_oneofs(&doo.attrs)
        }).collect::<Vec<Vec<_>>>().concat();

        let all_oos = vec![direct_oos, inner_oos].concat();
        if all_oos.len() == 0 { None }
        else { Some(all_oos) }
    }
    let oo_enums: Vec<_> = msgs.iter().filter_map(|msg| {
        get_oneofs(&msg.attributes)
    }).collect::<Vec<_>>().concat();

    println!("\noo_enums:\n{:?}\n", oo_enums);

    { // Test msgs and enum
        let msg_names = msgs.iter().map(|msg| &msg.name).collect::<Vec<_>>();
        msg_names.iter().for_each(|name| {
            if msg_names.iter().filter(|cname| **cname == *name).count() > 1 {
                println!("Error: Multiple instances of msg '{}' found.", name); abort()
            }
        });
        let enum_names = enums.iter().map(|enm| &enm.name).collect::<Vec<_>>();
        enum_names.iter().for_each(|name| {
            if enum_names.iter().filter(|cname| **cname == *name).count() > 1 {
                println!("Error: Multiple instances of enum '{}' found.", name); abort()
            }
        });

        // check that all attributes are defined
        //...
    }

    BitisProcessed { max_version_number: 0, msgs, enums, oo_enums}
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
        let test_empty_msg = "enum Lala(4) { one, two }";

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
    const ENUMS_HEADER: &str = "// Enums\n\n";
    const OO_HEADER: &str = "// Enums for oneof\n\n";
    const MSG_HEADER: &str = "// Messages\n\n";
    const PER_ENUM_HEADER: &str = "#[derive(BiserdiEnum, Debug, Clone, PartialEq)]\n#[biserdi_enum_id_dynbits(3)]\n#[allow(nonstandard_style)]\n";
    const PER_OO_HEADER: &str = "#[derive(BiserdiOneOf, Debug, Clone, PartialEq)]\n#[biserdi_enum_id_dynbits(3)]\n#[allow(nonstandard_style)]\n";
    const PER_MSG_HEADER: &str = "#[derive(BiserdiMsg, Debug, Clone, PartialEq)]\n#[allow(nonstandard_style)]\n";

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

        let processed_bitis = process_and_validate_bitis(&parsed_bitis);
        let rdo = RustDataObjects{ enums: processed_bitis.enums, msgs: processed_bitis.msgs,
            oos: processed_bitis.oo_enums };

        let rendered = rdo.render().unwrap();
        let lala_empty = "pub struct Lala {\n}\n";
        assert_eq!(rendered, (HEADER.to_owned() + ENUMS_HEADER + "\n\n" + OO_HEADER + "\n\n"  +
            MSG_HEADER + PER_MSG_HEADER +lala_empty).to_string());
    }

    #[rstest]
    fn msg_simple_msg() {
        let test_empty_msg = "//| comment for Lala\nmsg Lala { uint_5 a1; repeated_fixed_4 bool bool_array; }";
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
        let rdo = RustDataObjects{ enums: processed_bitis.enums, msgs: processed_bitis.msgs,
            oos: processed_bitis.oo_enums };

        let rendered = rdo.render().unwrap();
        let lala_commment = "/// comment for Lala\n";
        let lala_empty = "pub struct Lala {\n  pub a1: VarWithGivenBitSize<u8, 5>,\n  pub bool_array: [bool;4],\n}\n";
        println!("rendered:\n{}",rendered);
        assert_eq!(rendered, (HEADER.to_owned() + ENUMS_HEADER + "\n\n" + OO_HEADER + "\n\n" +
            MSG_HEADER + lala_commment + PER_MSG_HEADER +lala_empty).to_string());
    }

    #[rstest]
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
        let rdo = RustDataObjects{ enums: processed_bitis.enums, msgs: processed_bitis.msgs,
            oos: processed_bitis.oo_enums };

        let rendered = rdo.render().unwrap();
        let lala_commment = "/// comment for Numbers\n";
        let lala_enum = "pub enum Numbers {\n  One,\n  Two,\n  Three,\n}\n\n";
        println!("*rendered:\n{}",rendered);
        assert_eq!(rendered, (HEADER.to_owned() + ENUMS_HEADER + lala_commment + PER_ENUM_HEADER + lala_enum + OO_HEADER +
            "\n\n" + MSG_HEADER ).to_string());
    }

    #[rstest]
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
        let rdo = RustDataObjects{ enums: processed_bitis.enums, msgs: processed_bitis.msgs,
            oos: processed_bitis.oo_enums };

        let rendered = rdo.render().unwrap();
        let testoo_commment = "/// comment for Oneof\n";
        let testoo_enum = "pub enum OO_TestOo_OoLi {\n  test1(VarWithGivenBitSize<u8, 3>),\n  test2(f32),\n}\n\n";
        let testoo_msg = "pub struct TestOO {\n  pub oo_li: OO_TestOo_OoLi,\n  pub b1: bool,\n}\n";
        println!("*rendered:\n{}",rendered);
        assert_eq!(rendered, (HEADER.to_owned() + ENUMS_HEADER + "\n\n" + OO_HEADER + PER_OO_HEADER
            + testoo_enum + MSG_HEADER + testoo_commment + PER_MSG_HEADER + testoo_msg).to_string());
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

