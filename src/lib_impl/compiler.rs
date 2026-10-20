//! The bitis compiler: lexer, parser, validation, schema stamp and code generation.
//!
//! Pipeline: [`parse_sources`] (or [`parse_root`] for a single source) produces a list of
//! [`Value`]s, [`process_and_validate_bitis`] checks them and produces a [`BitisProcessed`],
//! which is rendered by the `render_*` functions (or the askama templates directly).
//! All user errors are reported as [`CompileError`]s carrying a source position.

use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::sync::LazyLock;

use askama::Template;
use indexmap::IndexMap;
use logos::{Lexer, Logos};
use regex::Regex;
use sha2::{Digest, Sha256};
use stringcase::Caser;

/// Version of the bitis compiler (stamped into the generated code).
pub const BITIS_COMPILER_VERSION: &str = env!("CARGO_PKG_VERSION");

// ************************************************************************
// Source positions and errors
// ************************************************************************

/// Byte range of a token in one of the input files (`file` indexes the list of [`SourceFile`]s).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SrcSpan {
    pub file: usize,
    pub start: usize,
    pub end: usize,
}

/// One input file of the compiler.
#[derive(Debug, Clone)]
pub struct SourceFile {
    pub name: String,
    pub content: String,
}

/// A user error (syntax or semantic) with the position of the offending token.
#[derive(Debug, Clone, PartialEq)]
pub struct CompileError {
    pub span: Option<SrcSpan>,
    pub message: String,
}
impl CompileError {
    pub fn new(span: SrcSpan, message: impl Into<String>) -> Self {
        CompileError { span: Some(span), message: message.into() }
    }
    pub fn without_span(message: impl Into<String>) -> Self {
        CompileError { span: None, message: message.into() }
    }
    /// Renders the error as `file:line:col: error: message` (line and column are 1-based).
    pub fn render(&self, files: &[SourceFile]) -> String {
        match self.span.and_then(|s| files.get(s.file).map(|f| (s, f))) {
            Some((span, file)) => {
                let (line, col) = line_col(&file.content, span.start);
                format!("{}:{}:{}: error: {}", file.name, line, col, self.message)
            }
            None => format!("error: {}", self.message),
        }
    }
}
impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// 1-based line and column (in characters) of a byte offset.
pub fn line_col(content: &str, offset: usize) -> (usize, usize) {
    let mut offset = offset.min(content.len());
    while !content.is_char_boundary(offset) { offset -= 1; }
    let before = &content[..offset];
    let line = before.matches('\n').count() + 1;
    let line_start = before.rfind('\n').map(|p| p + 1).unwrap_or(0);
    let col = before[line_start..].chars().count() + 1;
    (line, col)
}

/// Position of a definition for "first defined at" hints.
fn pos_hint(span: SrcSpan, files: Option<&[SourceFile]>) -> String {
    match files.and_then(|f| f.get(span.file).map(|f| (f, span))) {
        Some((f, s)) => { let (l, c) = line_col(&f.content, s.start); format!("{}:{}:{}", f.name, l, c) }
        None => format!("offset {}", span.start),
    }
}

// ************************************************************************
// Syntax tree
// ************************************************************************

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DynOrFixedType {
    Dyn(u8),
    Fixed(u8),
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct FixedPrecisionProperties {
    pub bits: u8,
    pub min_val: i64,
    pub max_val: i64,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SimpleType {
    Bool,
    UIntFixed(u8),
    IntFixed(u8),
    UIntDyn((u8, u8)),
    IntDyn((u8, u8)),
    Float,
    Double,
    FixedPrecision(FixedPrecisionProperties),
    /// `binary_dY` / `binary_dY[M]`: dyn bit size Y of the count, optional maximum length M (spec 2.16).
    Binary(u8, Option<u16>),
    /// `astr_dY` / `astr_dY[M]`: dyn bit size Y of the count, optional maximum length M (spec 2.16).
    AString(u8, Option<u16>),
}

#[derive(Debug, Clone)]
pub struct OneOfInfo {
    /// Generated type name `OO_{Parent}_{Name}` (both parts in PascalCase).
    pub name: String,
    pub dyn_bits: u8,
    pub attributes: Vec<Attribute>,
    pub default_attrib_name: String,
}
#[derive(Debug, Clone)]
pub enum AttributeDetails {
    AttributeSimple(SimpleType),
    AttributeEnumOrMsg(String),
    AttributeOneOf(OneOfInfo),
}
#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: String,
    /// Doc comment (`//|` lines joined with '\n').
    pub comment: Option<String>,
    pub is_repeated_and_size: Option<DynOrFixedType>,
    /// Maximum number of elements `M` of a `repeated_dyn_Y[M]` attribute (spec 2.16).
    pub repeated_max_len: Option<u16>,
    pub is_optional: bool,
    pub specific_details: AttributeDetails,
    pub name_span: SrcSpan,
    pub type_span: SrcSpan,
}
#[derive(Debug, Clone)]
pub struct Message {
    pub name: String,
    /// Doc comment (`//|` lines joined with '\n').
    pub comment: Option<String>,
    pub parent: Option<String>,
    pub attributes: Vec<Attribute>,
    pub name_span: SrcSpan,
    pub parent_span: Option<SrcSpan>,
}

/// Enum information for bitis. The ids are always DynInteger with a given bit size.
#[derive(Debug, Clone)]
pub struct Enum {
    pub name: String,
    /// Doc comment (`//|` lines joined with '\n').
    pub comment: Option<String>,
    pub bit_size: u8,
    pub values: Vec<String>,
    pub default: String,
    /// Doc comment per value (same length as `values`).
    pub value_comments: Vec<Option<String>>,
    pub name_span: SrcSpan,
    pub value_spans: Vec<SrcSpan>,
}

impl Enum {
    /// Doc comment of the value with the given index (used by the templates).
    pub fn value_comment(&self, index: impl std::borrow::Borrow<usize>) -> Option<&String> {
        self.value_comments.get(*index.borrow()).and_then(|c| c.as_ref())
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Message(Message),
    Enum(Enum),
}

// ************************************************************************
// Lexer
// ************************************************************************

/// Number after the last `_`, e.g. `uint_12` -> 12, `astr_d4` -> 4. Saturates on overflow.
fn suffix_number(lex: &mut Lexer<Token>) -> u32 {
    let s = lex.slice();
    let digits: &str = s.rsplit(|c: char| !c.is_ascii_digit()).next().unwrap_or("");
    digits.parse().unwrap_or(u32::MAX)
}
/// `uint_32d4` -> (32, 4). Saturates on overflow.
fn d_suffix_numbers(lex: &mut Lexer<Token>) -> (u32, u32) {
    let s = lex.slice();
    let nums = &s[s.find('_').map(|p| p + 1).unwrap_or(0)..];
    let mut it = nums.split('d').map(|n| n.parse().unwrap_or(u32::MAX));
    (it.next().unwrap_or(0), it.next().unwrap_or(0))
}
static FP_REGEX: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r"^fp_([0-9]+)\[[ \t]*(-?[0-9]+)[ \t]*,[ \t]*(-?[0-9]+)[ \t]*\]$").unwrap());
/// `fp_10[-2, 3]` -> (10, -2, 3). Saturates on overflow (the range is validated by the parser).
fn fp_properties(lex: &mut Lexer<Token>) -> Option<(u32, i128, i128)> {
    let c = FP_REGEX.captures(lex.slice())?;
    Some((c[1].parse().unwrap_or(u32::MAX),
          c[2].parse().unwrap_or(i128::MAX),
          c[3].parse().unwrap_or(i128::MAX)))
}
/// `[ 32 ]` -> 32 (maximum length, spec 2.16). Saturates on overflow (validated by the parser).
fn max_len_value(lex: &mut Lexer<Token>) -> i64 {
    let s = lex.slice().trim_matches(|c: char| c == '[' || c == ']' || c.is_whitespace());
    s.parse().unwrap_or(if s.starts_with('-') { i64::MIN } else { i64::MAX })
}
/// `( 4 )` -> 4
fn dyn_size(lex: &mut Lexer<Token>) -> u32 {
    let s = lex.slice().trim_matches(|c: char| c == '(' || c == ')' || c.is_whitespace());
    s.parse().unwrap_or(u32::MAX)
}
/// Text of a `//|` doc comment: one leading space and trailing whitespace removed.
fn doc_comment_text(lex: &mut Lexer<Token>) -> String {
    let s = &lex.slice()[3..];
    s.strip_prefix(' ').unwrap_or(s).trim_end().to_string()
}

#[derive(Debug, Logos, Clone, PartialEq)]
#[logos(skip r"[ \t\r\n\f]+")]
pub enum Token {
    #[regex(r"//[^\n]*", priority = 40)] Comment,
    #[regex(r"//\|[^\n]*", doc_comment_text, priority = 41)] SpecificComment(String),
    #[token("msg", priority = 20)] Msg,
    #[token("enum", priority = 20)] Enum,
    #[token("oneof", priority = 20)] OneOf,
    #[token("{")] CBraceOpen,
    #[token("}")] CBraceClose,
    #[token(":")] Colon,
    #[token(";")] SemiColon,
    #[token(",")] Comma,
    #[token("*")] Star,
    #[token("bool", priority = 30)] Bool,
    #[regex(r"uint_[0-9]+", suffix_number, priority = 30)] UIntFixed(u32),
    #[regex(r"int_[0-9]+", suffix_number, priority = 30)] IntFixed(u32),
    #[regex(r"uint_[0-9]+d[0-9]+", d_suffix_numbers, priority = 31)] UIntDyn((u32, u32)),
    #[regex(r"int_[0-9]+d[0-9]+", d_suffix_numbers, priority = 31)] IntDyn((u32, u32)),
    #[token("float", priority = 30)] Float,
    #[token("double", priority = 30)] Double,
    #[regex(r"astr_d[0-9]+", suffix_number, priority = 31)] AStr(u32),
    #[regex(r"fp_[0-9]+\[[ \t]*-?[0-9]+[ \t]*,[ \t]*-?[0-9]+[ \t]*\]", fp_properties, priority = 30)]
    FixedPoint((u32, i128, i128)),
    #[regex(r"binary_d[0-9]+", suffix_number, priority = 31)] Binary(u32),
    #[regex(r"repeated_dyn_[0-9]+", suffix_number, priority = 30)] RepeatedDyn(u32),
    #[regex(r"repeated_fixed_[0-9]+", suffix_number, priority = 30)] RepeatedFixed(u32),
    #[token("optional", priority = 30)] Optional,
    #[regex(r"[A-Za-z][A-Za-z0-9_]*", |lex| lex.slice().to_owned(), priority = 11)] StringVal(String),
    #[regex(r"\([ \t]*[0-9]+[ \t]*\)", dyn_size, priority = 40)] EnumDynSize(u32),
    /// Maximum length `[M]` after `astr_dY`, `binary_dY` or `repeated_dyn_Y` (spec 2.16).
    #[regex(r"\[[ \t]*-?[0-9]+[ \t]*\]", max_len_value, priority = 40)] MaxLen(i64),
}

fn token_desc(t: &Token) -> String {
    match t {
        Token::Comment | Token::SpecificComment(_) => "comment".into(),
        Token::Msg => "'msg'".into(),
        Token::Enum => "'enum'".into(),
        Token::OneOf => "'oneof'".into(),
        Token::CBraceOpen => "'{'".into(),
        Token::CBraceClose => "'}'".into(),
        Token::Colon => "':'".into(),
        Token::SemiColon => "';'".into(),
        Token::Comma => "','".into(),
        Token::Star => "'*'".into(),
        Token::EnumDynSize(n) => format!("'({n})'"),
        Token::MaxLen(n) => format!("maximum length '[{n}]'"),
        Token::StringVal(s) => format!("identifier '{s}'"),
        Token::Optional => "'optional'".into(),
        Token::RepeatedDyn(n) => format!("'repeated_dyn_{n}'"),
        Token::RepeatedFixed(n) => format!("'repeated_fixed_{n}'"),
        Token::Bool => "type 'bool'".into(),
        Token::Float => "type 'float'".into(),
        Token::Double => "type 'double'".into(),
        Token::UIntFixed(b) => format!("type 'uint_{b}'"),
        Token::IntFixed(b) => format!("type 'int_{b}'"),
        Token::UIntDyn((x, y)) => format!("type 'uint_{x}d{y}'"),
        Token::IntDyn((x, y)) => format!("type 'int_{x}d{y}'"),
        Token::AStr(b) => format!("type 'astr_d{b}'"),
        Token::Binary(b) => format!("type 'binary_d{b}'"),
        Token::FixedPoint((b, min, max)) => format!("type 'fp_{b}[{min},{max}]'"),
    }
}

// ************************************************************************
// Parser
// ************************************************************************

struct Parser<'s> {
    lexer: Lexer<'s, Token>,
    file: usize,
    len: usize,
    docs: Vec<String>,
    peeked: Option<(Token, SrcSpan)>,
}

impl<'s> Parser<'s> {
    fn new(content: &'s str, file: usize) -> Self {
        Parser { lexer: Token::lexer(content), file, len: content.len(), docs: vec![], peeked: None }
    }
    fn span(&self, r: std::ops::Range<usize>) -> SrcSpan { SrcSpan { file: self.file, start: r.start, end: r.end } }
    fn eof_span(&self) -> SrcSpan { SrcSpan { file: self.file, start: self.len, end: self.len } }

    /// Next significant token. Plain comments are skipped, doc comments are collected.
    fn next(&mut self) -> Result<Option<(Token, SrcSpan)>, CompileError> {
        if let Some(p) = self.peeked.take() { return Ok(Some(p)); }
        loop {
            match self.lexer.next() {
                None => return Ok(None),
                Some(Err(())) => {
                    let span = self.span(self.lexer.span());
                    return Err(CompileError::new(span, format!("unexpected text '{}'", self.lexer.slice())));
                }
                Some(Ok(Token::Comment)) => (),
                Some(Ok(Token::SpecificComment(s))) => self.docs.push(s),
                Some(Ok(t)) => return Ok(Some((t, self.span(self.lexer.span())))),
            }
        }
    }
    fn expect_next(&mut self, expected: &str) -> Result<(Token, SrcSpan), CompileError> {
        match self.next()? {
            Some(t) => Ok(t),
            None => Err(CompileError::new(self.eof_span(), format!("unexpected end of file, expected {expected}"))),
        }
    }
    fn expect(&mut self, tok: Token, expected: &str) -> Result<SrcSpan, CompileError> {
        let (t, span) = self.expect_next(expected)?;
        if t == tok { Ok(span) } else { Err(unexpected(&t, span, expected)) }
    }
    fn expect_ident(&mut self, expected: &str) -> Result<(String, SrcSpan), CompileError> {
        match self.expect_next(expected)? {
            (Token::StringVal(s), span) => Ok((s, span)),
            (t, span) => Err(unexpected(&t, span, expected)),
        }
    }
    fn expect_dyn_size(&mut self, what: &str) -> Result<u8, CompileError> {
        match self.expect_next(&format!("dyn bit size for {what}, e.g. '(4)'"))? {
            (Token::EnumDynSize(n), span) => check_range(n, 1, 32, span, &format!("dyn bit size of {what}")),
            (t, span) => Err(unexpected(&t, span, &format!("dyn bit size for {what}, e.g. '(4)'"))),
        }
    }
    /// Optional maximum length `[M]` (spec 2.16) directly after `astr_dY`, `binary_dY` or
    /// `repeated_dyn_Y`; validates `1 <= M <= 65535`.
    fn opt_max_len(&mut self, what: &str) -> Result<Option<u16>, CompileError> {
        match self.next()? {
            Some((Token::MaxLen(n), span)) => {
                if !(1..=MAX_LEN_LIMIT as i64).contains(&n) {
                    let n_str = if n == i64::MAX || n == i64::MIN { "a too large value".to_string() } else { n.to_string() };
                    return Err(CompileError::new(span, format!(
                        "maximum length [M] of {what} must be in 1..={MAX_LEN_LIMIT}, got {n_str}")));
                }
                Ok(Some(n as u16))
            }
            other => { self.peeked = other; Ok(None) }
        }
    }
    /// Doc comment lines collected since the last call.
    fn take_doc(&mut self) -> Option<String> {
        if self.docs.is_empty() { None } else { Some(std::mem::take(&mut self.docs).join("\n")) }
    }
}

/// Largest allowed maximum length `[M]` (spec 2.16).
pub const MAX_LEN_LIMIT: u16 = 65535;

fn unexpected(t: &Token, span: SrcSpan, expected: &str) -> CompileError {
    if let Token::MaxLen(_) = t {
        return CompileError::new(span, format!(
            "a {} is only allowed directly after astr_dY, binary_dY or repeated_dyn_Y", token_desc(t)));
    }
    CompileError::new(span, format!("expected {expected}, found {}", token_desc(t)))
}
fn check_range(n: u32, min: u32, max: u32, span: SrcSpan, what: &str) -> Result<u8, CompileError> {
    if n < min || n > max {
        let n_str = if n == u32::MAX { "a too large value".to_string() } else { n.to_string() };
        Err(CompileError::new(span, format!("{what} must be in {min}..={max}, got {n_str}")))
    } else { Ok(n as u8) }
}

/// Parses one bitis source. `file` is the index used in the spans of the result.
pub fn parse_source(content: &str, file: usize) -> Result<Vec<Value>, CompileError> {
    let mut p = Parser::new(content, file);
    parse_values(&mut p)
}

/// Parses all sources (in the given order) into one list of definitions.
pub fn parse_sources(files: &[SourceFile]) -> Result<Vec<Value>, CompileError> {
    let mut values = Vec::new();
    for (i, f) in files.iter().enumerate() {
        values.extend(parse_source(&f.content, i)?);
    }
    Ok(values)
}

/// Parses the tokens of a lexer (file index 0).
pub fn parse_root(lexer: &mut Lexer<'_, Token>) -> Result<Vec<Value>, CompileError> {
    let source = lexer.source();
    let mut p = Parser::new(source, 0);
    p.lexer = lexer.clone();
    let r = parse_values(&mut p);
    *lexer = p.lexer;
    r
}

fn parse_values(p: &mut Parser) -> Result<Vec<Value>, CompileError> {
    let mut list = Vec::new();
    while let Some((t, span)) = p.next()? {
        let doc = p.take_doc();
        match t {
            Token::Msg => list.push(parse_msg(p, doc)?),
            Token::Enum => list.push(parse_enum(p, doc)?),
            t => return Err(unexpected(&t, span, "'msg' or 'enum'")),
        }
    }
    Ok(list)
}

fn parse_msg(p: &mut Parser, comment: Option<String>) -> Result<Value, CompileError> {
    let (name, name_span) = p.expect_ident("message name")?;
    let (parent, parent_span) = match p.expect_next(&format!("'{{' or ':' after message name '{name}'"))? {
        (Token::CBraceOpen, _) => (None, None),
        (Token::Colon, _) => {
            let (parent, span) = p.expect_ident(&format!("parent message name of '{name}'"))?;
            p.expect(Token::CBraceOpen, &format!("'{{' after parent of message '{name}'"))?;
            (Some(parent), Some(span))
        }
        (t, span) => return Err(unexpected(&t, span, &format!("'{{' or ':' after message name '{name}'"))),
    };

    let mut attributes = Vec::new();
    loop {
        let (t, span) = p.expect_next(&format!("attribute or '}}' of message '{name}'"))?;
        let doc = p.take_doc();
        match t {
            // doc comments directly before '}' document nothing and are dropped
            Token::CBraceClose => break,
            t => attributes.push(parse_attribute(p, t, span, doc, &name, false)?),
        }
    }
    Ok(Value::Message(Message { name, comment, parent, attributes, name_span, parent_span }))
}

fn parse_attribute(p: &mut Parser, first: Token, first_span: SrcSpan, comment: Option<String>,
                   parent_name: &str, in_oneof: bool) -> Result<Attribute, CompileError> {
    let mut is_optional = false;
    let mut is_repeated_and_size: Option<DynOrFixedType> = None;
    let mut repeated_max_len: Option<u16> = None;
    let (mut t, mut span) = (first, first_span);
    let ctx = if in_oneof { format!("oneof '{parent_name}'") } else { format!("message '{parent_name}'") };

    let details = loop {
        let has_modifier = is_optional || is_repeated_and_size.is_some();
        match t {
            Token::Optional | Token::RepeatedDyn(_) | Token::RepeatedFixed(_) if in_oneof =>
                return Err(CompileError::new(span, "modifiers (optional, repeated) are not allowed inside a oneof")),
            Token::Optional | Token::RepeatedDyn(_) | Token::RepeatedFixed(_) if has_modifier =>
                return Err(CompileError::new(span, "only one modifier (optional, repeated_fixed_N, repeated_dyn_N) is allowed per attribute")),
            Token::Optional => is_optional = true,
            Token::RepeatedDyn(b) => {
                is_repeated_and_size = Some(DynOrFixedType::Dyn(check_range(b, 1, 32, span, "dyn bit size of repeated_dyn")?));
                repeated_max_len = p.opt_max_len("repeated_dyn_Y")?;
            }
            Token::RepeatedFixed(n) =>
                is_repeated_and_size = Some(DynOrFixedType::Fixed(check_range(n, 1, 255, span, "number of elements of repeated_fixed")?)),
            Token::OneOf if has_modifier =>
                return Err(CompileError::new(span, "a oneof can not be used with modifiers (optional, repeated)")),
            Token::OneOf => return parse_oneof(p, span, comment, parent_name),
            Token::StringVal(s) => break AttributeDetails::AttributeEnumOrMsg(s),
            t => break AttributeDetails::AttributeSimple(simple_type(p, t, span, &ctx)?),
        }
        (t, span) = p.expect_next(&format!("attribute type in {ctx}"))?;
    };
    let type_span = span;
    let (name, name_span) = p.expect_ident(&format!("attribute name in {ctx}"))?;
    p.expect(Token::SemiColon, &format!("';' after attribute '{name}'"))?;
    Ok(Attribute { name, comment, is_repeated_and_size, repeated_max_len, is_optional, specific_details: details,
        name_span, type_span })
}

fn simple_type(p: &mut Parser, t: Token, span: SrcSpan, ctx: &str) -> Result<SimpleType, CompileError> {
    let dyn_int = |x: u32, y: u32, kind: &str| -> Result<(u8, u8), CompileError> {
        let x = check_range(x, 1, 64, span, &format!("bit size of {kind}_XdY (X)"))?;
        let y = check_range(y, 1, x as u32, span, &format!("chunk bit size of {kind}_XdY (Y)"))?;
        Ok((x, y))
    };
    Ok(match t {
        Token::Bool => SimpleType::Bool,
        Token::UIntFixed(b) => SimpleType::UIntFixed(check_range(b, 1, 64, span, "bit size of uint_X")?),
        Token::IntFixed(b) => SimpleType::IntFixed(check_range(b, 1, 64, span, "bit size of int_X")?),
        Token::UIntDyn((x, y)) => SimpleType::UIntDyn(dyn_int(x, y, "uint")?),
        Token::IntDyn((x, y)) => SimpleType::IntDyn(dyn_int(x, y, "int")?),
        Token::Float => SimpleType::Float,
        Token::Double => SimpleType::Double,
        Token::AStr(b) => {
            let y = check_range(b, 1, 32, span, "dyn bit size of astr_dY")?;
            SimpleType::AString(y, p.opt_max_len("astr_dY")?)
        }
        Token::Binary(b) => {
            let y = check_range(b, 1, 32, span, "dyn bit size of binary_dY")?;
            SimpleType::Binary(y, p.opt_max_len("binary_dY")?)
        }
        Token::FixedPoint((bits, min, max)) => {
            let bits = check_range(bits, 2, 53, span, "bit size of fp_N")?;
            let range = i64::MIN as i128..=i64::MAX as i128;
            if !range.contains(&min) || !range.contains(&max) {
                return Err(CompileError::new(span, "bounds of fp_N[min,max] must fit into a 64 bit signed integer"));
            }
            if min >= max {
                return Err(CompileError::new(span, format!("fp_N[min,max] requires min < max, got [{min},{max}]")));
            }
            if (max as i64).checked_sub(min as i64).is_none() {
                return Err(CompileError::new(span, "range max - min of fp_N[min,max] must fit into a 64 bit signed integer"));
            }
            SimpleType::FixedPrecision(FixedPrecisionProperties { bits, min_val: min as i64, max_val: max as i64 })
        }
        t => return Err(unexpected(&t, span, &format!("attribute type or modifier in {ctx}"))),
    })
}

fn parse_oneof(p: &mut Parser, oneof_span: SrcSpan, comment: Option<String>, parent_name: &str)
    -> Result<Attribute, CompileError> {
    let (oo_name, name_span) = p.expect_ident(&format!("oneof name in '{parent_name}'"))?;
    let dyn_bits = p.expect_dyn_size(&format!("oneof '{oo_name}'"))?;
    p.expect(Token::CBraceOpen, &format!("'{{' after oneof '{oo_name}'"))?;

    let mut attributes: Vec<Attribute> = Vec::new();
    let mut default_name: Option<String> = None;
    loop {
        let (mut t, mut span) = p.expect_next(&format!("oneof element or '}}' of oneof '{oo_name}'"))?;
        let mut doc = p.take_doc();
        let is_default = t == Token::Star;
        if is_default {
            let star_span = span;
            (t, span) = p.expect_next(&format!("oneof element after '*' in oneof '{oo_name}'"))?;
            doc = match (doc, p.take_doc()) { (Some(a), Some(b)) => Some(a + "\n" + &b), (a, b) => a.or(b) };
            if default_name.is_some() {
                return Err(CompileError::new(star_span, format!("multiple elements of oneof '{oo_name}' are marked as default")));
            }
        }
        if t == Token::CBraceClose {
            if is_default {
                return Err(unexpected(&t, span, &format!("oneof element after '*' in oneof '{oo_name}'")));
            }
            break;
        }
        let attr = parse_attribute(p, t, span, doc, &oo_name, true)?;
        if is_default { default_name = Some(attr.name.clone()); }
        attributes.push(attr);
    }
    let Some(default_attrib_name) = default_name else {
        return Err(CompileError::new(name_span, format!(
            "no default element in oneof '{oo_name}' (mark one element with a leading '*')")));
    };

    Ok(Attribute {
        name: oo_name.clone(), comment, is_repeated_and_size: None, repeated_max_len: None, is_optional: false,
        specific_details: AttributeDetails::AttributeOneOf(OneOfInfo {
            name: oneof_type_name(parent_name, &oo_name), dyn_bits, attributes, default_attrib_name,
        }),
        name_span, type_span: oneof_span,
    })
}

/// Generated type name of a oneof.
pub fn oneof_type_name(parent_name: &str, oo_name: &str) -> String {
    format!("OO_{}_{}", parent_name.to_pascal_case(), oo_name.to_pascal_case())
}

fn parse_enum(p: &mut Parser, comment: Option<String>) -> Result<Value, CompileError> {
    let (name, name_span) = p.expect_ident("enum name")?;
    let bit_size = p.expect_dyn_size(&format!("enum '{name}'"))?;
    p.expect(Token::CBraceOpen, &format!("'{{' after enum '{name}'"))?;

    let mut values = Vec::new();
    let mut value_comments = Vec::new();
    let mut value_spans = Vec::new();
    let mut default: Option<String> = None;
    loop {
        // value (optionally preceded by '*') or '}'
        let (mut t, mut span) = p.expect_next(&format!("enum value or '}}' of enum '{name}'"))?;
        let mut doc = p.take_doc();
        let is_default = t == Token::Star;
        if is_default {
            if default.is_some() {
                return Err(CompileError::new(span, format!("multiple values of enum '{name}' are marked as default")));
            }
            (t, span) = p.expect_next(&format!("enum value after '*' in enum '{name}'"))?;
            doc = match (doc, p.take_doc()) { (Some(a), Some(b)) => Some(a + "\n" + &b), (a, b) => a.or(b) };
        }
        match t {
            Token::CBraceClose if !is_default => break,
            Token::StringVal(v) => {
                if is_default { default = Some(v.clone()); }
                values.push(v); value_comments.push(doc); value_spans.push(span);
            }
            t => return Err(unexpected(&t, span, &format!("enum value of enum '{name}'"))),
        }
        // ',' or '}'
        match p.expect_next(&format!("',' or '}}' in enum '{name}'"))? {
            (Token::Comma, _) => (),
            (Token::CBraceClose, _) => break,
            (t, span) => return Err(unexpected(&t, span, &format!("',' or '}}' after enum value in enum '{name}'"))),
        }
    }
    p.take_doc();
    let Some(default) = default else {
        return Err(CompileError::new(name_span, format!(
            "no default value in enum '{name}' (mark one value with a leading '*')")));
    };
    Ok(Value::Enum(Enum { name, comment, bit_size, values, default, value_comments, name_span, value_spans }))
}

// ************************************************************************
// Validation
// ************************************************************************

/// Words that are reserved in at least one of the target languages (Rust, C++, Python, Dart).
/// Identifiers (and their snake/camel/Pascal case forms used by the generators) must not be one of them.
pub const RESERVED_WORDS: &[&str] = &[
    // Rust (strict, reserved and edition 2024)
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern", "false",
    "fn", "for", "gen", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
    "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where",
    "while", "abstract", "become", "box", "do", "final", "macro", "override", "priv", "try", "typeof",
    "unsized", "virtual", "yield",
    // C++
    "alignas", "alignof", "and", "and_eq", "asm", "auto", "bitand", "bitor", "bool", "case", "catch", "char",
    "char8_t", "char16_t", "char32_t", "class", "compl", "concept", "consteval", "constexpr", "constinit",
    "const_cast", "co_await", "co_return", "co_yield", "decltype", "default", "delete", "double",
    "dynamic_cast", "explicit", "export", "float", "friend", "goto", "inline", "int", "long", "mutable",
    "namespace", "new", "noexcept", "not", "not_eq", "nullptr", "operator", "or", "or_eq", "private",
    "protected", "public", "register", "reinterpret_cast", "requires", "short", "signed", "sizeof",
    "static_assert", "static_cast", "switch", "template", "this", "thread_local", "throw", "typedef",
    "typeid", "typename", "union", "unsigned", "using", "void", "volatile", "wchar_t", "xor", "xor_eq",
    // Python
    "False", "None", "True", "assert", "def", "del", "elif", "except", "finally", "from", "global",
    "import", "is", "lambda", "nonlocal", "pass", "raise", "with",
    // Dart
    "extends", "null", "rethrow", "var",
];

/// Names used by the runtimes / generated code that must not be used as message or enum names.
pub const RESERVED_TYPE_NAMES: &[&str] = &[
    // generated constants
    "BITIS_VERSION", "BITIS_SCHEMA_HASH",
    // Rust runtime and prelude
    "Biseri", "Bides", "BiserdiTrait", "BiserdiTraitVarBitSize", "BiserSizes", "IntegerBaseFunctions",
    "ValFromInto", "IntWithGivenBitSize", "DynInteger", "FixPrecisionMinMax", "FixPrecisionVal", "Binary",
    "BitisAString", "BitisOption", "FixedArray", "DynArray", "MessageManager", "DecodeError", "EncodeError",
    "BoundedBinary", "BoundedAString", "BoundedDynArray",
    "BiserdiMsg", "BiserdiEnum", "BiserdiOneOf", "AsRefStr", "Option", "Some", "Result", "Ok", "Err", "Vec",
    "String", "Box", "Default", "Clone", "Debug", "PartialEq", "Copy",
    // C++ runtime
    "BitisEnum", "BitisBool", "BitisOptional", "IntgralWithGivenBitSize", "BitisFloatingPoint",
    "BitisSerializer", "BitisDeserializer", "BitisSize", "BitisError", "oneof_helper", "message_helper",
    "bitis_helper", "std",
    // Python bindings (pyo3)
    "Py", "PyResult", "PyObject", "PyBytes", "PyErr", "PyException", "Python", "messages",
    "Bound", "PyClass", "BitisPyCheck", "bitis_py_errors",
    // Python stub (.pyi): builtins used in the annotations
    "str", "bytes", "list", "tuple", "object", "ValueError", "staticmethod", "property", "_typing",
    // Dart: generated top level names, names used by the generated code, built-in identifiers
    "bitisVersion", "bitisSchemaHash", "bitisSchemaHashHex", "bitisEncode", "bitisDecode", "bitisDecodeExact",
    "bitisKeep", "bitisKeepOr", "identical", "override", "BitWriter", "BitReader", "BitisMessage", "BitisJson",
    "BitisEq", "FixedPoint", "FixedPointType", "Int64", "BigInt", "Uint8List", "List", "Map", "Object",
    "StateError", "FormatException", "dynamic", "Function", "get", "set", "late", "required", "covariant",
    "deferred", "extension", "external", "factory", "implements", "interface", "library", "mixin", "part",
];

/// Members generated into every C++ message struct (besides the attributes).
const CPP_MSG_MEMBERS: &[&str] = &["MsgT", "msg_attr", "serialize", "deserialize", "print", "is_equal"];
/// Methods generated into every Python (pyo3) message class (besides the attributes).
const PY_MSG_MEMBERS: &[&str] = &["default", "serialize", "deserialize", "to_rust", "from_rust_obj",
    "try_serialize", "try_deserialize", "try_deserialize_exact", "min_bits", "max_bits"];
/// Members of every Dart message class (and members of `Object`), compared with the Dart field names
/// (lowerCamelCase of the attribute names).
const DART_MSG_MEMBERS: &[&str] = &[
    "bitisWrite", "bitisRead", "bitisMinBits", "bitisMaxBits", "toBytes", "fromBytes", "fromBytesExact", "toJson", "fromJson", "copyWith",
    "hashCode", "runtimeType", "toString", "noSuchMethod",
];
/// Members of every Dart enum (and of `Enum`), compared with the Dart value names (lowerCamelCase).
const DART_ENUM_MEMBERS: &[&str] = &[
    "values", "index", "name", "bitisName", "defaultValue", "bitisWrite", "bitisRead", "toJson", "fromJson",
    "hashCode", "runtimeType", "toString", "noSuchMethod",
];

/// Kind of a name, determines which case conversions of it the generators emit.
#[derive(Clone, Copy, PartialEq)]
enum NameKind {
    /// message / enum names: used as is
    Type,
    /// attributes: as is (Rust, C++, Python) and camelCase (Dart)
    Attribute,
    /// oneof names and elements: additionally snake_case (Python) and PascalCase (Rust/C++ variants)
    OneOf,
    /// enum values: PascalCase (Rust, C++, Python) and camelCase (Dart)
    EnumValue,
}

/// The identifier forms the generators derive from a name.
fn name_forms(name: &str, kind: NameKind) -> Vec<String> {
    match kind {
        NameKind::Type => vec![name.to_string()],
        NameKind::Attribute => vec![name.to_string(), name.to_camel_case()],
        NameKind::OneOf => vec![name.to_string(), name.to_snake_case(), name.to_camel_case(), name.to_pascal_case()],
        NameKind::EnumValue => vec![name.to_string(), name.to_pascal_case(), name.to_camel_case()],
    }
}

struct Validator<'a> {
    errors: Vec<CompileError>,
    files: Option<&'a [SourceFile]>,
}

impl<'a> Validator<'a> {
    fn err(&mut self, span: SrcSpan, msg: impl Into<String>) { self.errors.push(CompileError::new(span, msg)); }

    fn check_reserved(&mut self, name: &str, span: SrcSpan, what: &str, kind: NameKind) {
        for form in name_forms(name, kind) {
            if RESERVED_WORDS.contains(&form.as_str()) {
                let as_form = if form == name { String::new() } else { format!(" (as '{form}')") };
                self.err(span, format!("{what} '{name}'{as_form} is a reserved word in Rust, C++, Python or Dart"));
                return;
            }
        }
    }

    /// Reports duplicate names and names that are equal after one of the given case conversions.
    /// `items`: (name, span); `conversions`: (label, conversion).
    fn check_unique_forms(&mut self, items: &[(String, SrcSpan)], what: &str, scope: &str,
                          conversions: &[(&str, NameConversion)]) {
        let mut seen: HashMap<&str, SrcSpan> = HashMap::new();
        for (name, span) in items {
            match seen.get(name.as_str()) {
                Some(first) => {
                    let msg = format!("duplicate {what} '{name}' in {scope} (first defined at {})", pos_hint(*first, self.files));
                    self.err(*span, msg);
                }
                None => { seen.insert(name, *span); }
            }
        }
        for (label, conv) in conversions {
            let mut seen: HashMap<String, (&str, SrcSpan)> = HashMap::new();
            for (name, span) in items {
                let c = conv(name);
                match seen.get(&c) {
                    Some((first, first_span)) if *first != name.as_str() => {
                        let msg = format!("{what} '{name}' and '{first}' (at {}) in {scope} both map to '{c}' ({label})",
                                          pos_hint(*first_span, self.files));
                        self.err(*span, msg);
                    }
                    Some(_) => (),
                    None => { seen.insert(c, (name, *span)); }
                }
            }
        }
    }
}

type NameConversion = fn(&str) -> String;
fn conv_pascal(s: &str) -> String { s.to_pascal_case() }
fn conv_camel(s: &str) -> String { s.to_camel_case() }
fn conv_snake(s: &str) -> String { s.to_snake_case() }

/// Validates the parsed definitions. `files` is only used for "first defined at" hints.
pub fn validate_bitis(values: &[Value], files: Option<&[SourceFile]>) -> Vec<CompileError> {
    let mut v = Validator { errors: vec![], files };

    let msgs: Vec<&Message> = values.iter().filter_map(|x| match x { Value::Message(m) => Some(m), _ => None }).collect();
    let enums: Vec<&Enum> = values.iter().filter_map(|x| match x { Value::Enum(e) => Some(e), _ => None }).collect();
    let msg_by_name: HashMap<&str, &Message> = msgs.iter().map(|m| (m.name.as_str(), *m)).collect();
    let enum_names: HashSet<&str> = enums.iter().map(|e| e.name.as_str()).collect();

    // *** type names (messages and enums share one namespace) and generated type names
    {
        let mut generated: HashMap<String, (String, SrcSpan)> = HashMap::new();
        let mut add = |v: &mut Validator, gen_name: String, origin: String, span: SrcSpan| {
            if RESERVED_TYPE_NAMES.contains(&gen_name.as_str()) {
                v.err(span, format!("{origin} uses the name '{gen_name}' which is reserved by the generated code / bitis runtime"));
                return;
            }
            if let Some((first_origin, first_span)) = generated.get(&gen_name) {
                let msg = if *first_origin == origin {
                    format!("duplicate definition of {origin} (first defined at {})", pos_hint(*first_span, v.files))
                } else {
                    format!("{origin} clashes with {first_origin} (at {}): both generate the type name '{gen_name}'",
                            pos_hint(*first_span, v.files))
                };
                v.err(span, msg);
            } else {
                generated.insert(gen_name, (origin, span));
            }
        };
        for x in values {
            match x {
                Value::Enum(e) => {
                    add(&mut v, e.name.clone(), format!("type '{}'", e.name), e.name_span);
                    add(&mut v, format!("{}Enum", e.name), format!("C++ enum namespace of enum '{}'", e.name), e.name_span);
                }
                Value::Message(m) => {
                    add(&mut v, m.name.clone(), format!("type '{}'", m.name), m.name_span);
                    for (oo, _) in collect_oneofs(&m.attributes) {
                        let origin = format!("oneof '{}' of message '{}'", oo.0, m.name);
                        add(&mut v, oo.1.name.clone(), origin.clone(), oo.2);
                        add(&mut v, format!("{}Enum", oo.1.name), format!("Python selector enum of {origin}"), oo.2);
                        // duplicates within the oneof are reported by the oneof element checks
                        let mut seen_elements = HashSet::new();
                        for va in oo.1.attributes.iter().filter(|va| seen_elements.insert(va.name.to_pascal_case())) {
                            add(&mut v, format!("{}_{}", oo.1.name, va.name.to_pascal_case()),
                                format!("Dart class of element '{}' of {origin}", va.name), va.name_span);
                        }
                    }
                }
            }
        }
    }

    // *** enums
    for e in &enums {
        v.check_reserved(&e.name, e.name_span, "enum name", NameKind::Type);
        let items: Vec<(String, SrcSpan)> = e.values.iter().cloned().zip(e.value_spans.iter().cloned()).collect();
        v.check_unique_forms(&items, "enum value", &format!("enum '{}'", e.name),
                             &[("PascalCase, used by Rust/C++/Python", conv_pascal), ("camelCase, used by Dart", conv_camel)]);
        for (val, span) in &items { v.check_reserved(val, *span, "enum value", NameKind::EnumValue); }
        for (val, span) in &items {
            let dart = val.to_camel_case();
            if DART_ENUM_MEMBERS.contains(&dart.as_str()) || dart == e.name {
                v.err(*span, format!("enum value '{val}' (Dart name '{dart}') clashes with a member of the generated Dart enum '{}'", e.name));
            }
        }
    }

    // *** messages
    for m in &msgs {
        v.check_reserved(&m.name, m.name_span, "message name", NameKind::Type);
        let unknown_parent = m.parent.as_ref().zip(m.parent_span).filter(|(p, _)| !msg_by_name.contains_key(p.as_str()));
        if let Some((parent, span)) = unknown_parent {
            if enum_names.contains(parent.as_str()) {
                v.err(span, format!("parent '{parent}' of message '{}' is an enum, expected a message", m.name));
            } else {
                v.err(span, format!("unknown parent message '{parent}' of message '{}'", m.name));
            }
        }
        validate_attributes(&mut v, &m.attributes, &format!("message '{}'", m.name), &msg_by_name, &enum_names, false);

        // generated members of the C++ message struct and of the Python class:
        // (member name, span, attribute name)
        let mut cpp_members: Vec<(String, SrcSpan, &str)> = vec![];
        let mut py_members: Vec<(String, SrcSpan, &str)> = vec![];
        for a in &m.attributes {
            if CPP_MSG_MEMBERS.contains(&a.name.as_str()) {
                v.err(a.name_span, format!("attribute name '{}' clashes with a generated member of the C++ message struct", a.name));
            }
            if PY_MSG_MEMBERS.contains(&a.name.as_str()) {
                v.err(a.name_span, format!("attribute name '{}' clashes with a generated method of the Python message class", a.name));
            }
            let an = a.name.as_str();
            cpp_members.push((a.name.clone(), a.name_span, an));
            cpp_members.push((format!("{}_T", a.name.to_pascal_case()), a.name_span, an));
            cpp_members.push((format!("str_{}", a.name), a.name_span, an));
            py_members.push((a.name.clone(), a.name_span, an));
            if let AttributeDetails::AttributeOneOf(oo) = &a.specific_details {
                let base = a.name.to_snake_case();
                py_members.push((format!("{base}_oo"), a.name_span, an));
                for va in &oo.attributes {
                    let vs = va.name.to_snake_case();
                    py_members.push((format!("{base}_{vs}"), va.name_span, an));
                    py_members.push((format!("{base}_get_{vs}"), va.name_span, an));
                    py_members.push((format!("{base}_set_{vs}"), va.name_span, an));
                }
            }
        }
        // Dart: fields are the lowerCamelCase attribute names, fixed point attributes also get `<field>FpType`
        let mut dart_members: Vec<(String, SrcSpan, &str)> = vec![];
        for a in &m.attributes {
            let dart = a.name.to_camel_case();
            if DART_MSG_MEMBERS.contains(&dart.as_str()) {
                v.err(a.name_span, format!("attribute name '{}' (Dart name '{dart}') clashes with a member of the generated Dart message class", a.name));
            } else if dart == m.name || msg_by_name.contains_key(dart.as_str()) || enum_names.contains(dart.as_str()) {
                v.err(a.name_span, format!("attribute name '{}' (Dart name '{dart}') clashes with the type name '{dart}' in the generated Dart code", a.name));
            }
            dart_members.push((dart.clone(), a.name_span, &a.name));
            if let AttributeDetails::AttributeSimple(SimpleType::FixedPrecision(_)) = a.specific_details {
                dart_members.push((format!("{dart}FpType"), a.name_span, &a.name));
            }
        }
        for (label, members) in [("C++ message struct", cpp_members), ("Python message class", py_members),
                                 ("Dart message class", dart_members)] {
            let mut seen: HashMap<&str, (SrcSpan, &str)> = HashMap::new();
            for (member, span, attr) in &members {
                match seen.get(member.as_str()) {
                    // equal names / case conversions are already reported above
                    Some((first, first_attr)) if *first_attr != *attr && first_attr.to_pascal_case() != attr.to_pascal_case() => {
                        let msg = format!("attribute '{attr}' generates the member '{member}' in the {label} of message '{}', \
                                           which is already generated for attribute '{first_attr}' (at {})", m.name, pos_hint(*first, v.files));
                        v.err(*span, msg);
                    }
                    Some(_) => (),
                    None => { seen.insert(member, (*span, *attr)); }
                }
            }
        }
    }

    // *** cycles between messages
    let cycles = find_cycles(&msgs, &msg_by_name);
    let has_cycles = !cycles.is_empty();
    for (span, path) in cycles {
        v.err(span, format!("recursive message definition: {}", path.join(" -> ")));
    }

    // *** repeated_dyn of messages without any bits (spec section 3)
    if !has_cycles {
        let mut zero_cache: HashMap<String, bool> = HashMap::new();
        for m in &msgs {
            check_repeated_dyn_zero(&mut v, &m.attributes, &msg_by_name, &mut zero_cache);
        }
    }

    // stable sort by position; only the first error per token is reported
    v.errors.sort_by_key(|e| e.span);
    v.errors.dedup_by_key(|e| e.span);
    v.errors
}

fn validate_attributes(v: &mut Validator, attrs: &[Attribute], scope: &str, msg_by_name: &HashMap<&str, &Message>,
                       enum_names: &HashSet<&str>, in_oneof: bool) {
    let items: Vec<(String, SrcSpan)> = attrs.iter().map(|a| (a.name.clone(), a.name_span)).collect();
    if in_oneof {
        v.check_unique_forms(&items, "oneof element", scope, &[
            ("PascalCase, used by Rust/C++", conv_pascal), ("snake_case, used by Python", conv_snake),
            ("camelCase, used by Dart", conv_camel)]);
    } else {
        v.check_unique_forms(&items, "attribute", scope, &[
            ("PascalCase, used by C++", conv_pascal), ("camelCase, used by Dart", conv_camel)]);
    }
    for a in attrs {
        let is_oo = in_oneof || matches!(a.specific_details, AttributeDetails::AttributeOneOf(_));
        v.check_reserved(&a.name, a.name_span, if in_oneof { "oneof element name" } else { "attribute name" },
                         if is_oo { NameKind::OneOf } else { NameKind::Attribute });
        match &a.specific_details {
            AttributeDetails::AttributeEnumOrMsg(t) => {
                if !msg_by_name.contains_key(t.as_str()) && !enum_names.contains(t.as_str()) {
                    v.err(a.type_span, format!("unknown type '{t}' of attribute '{}' in {scope}", a.name));
                }
            }
            AttributeDetails::AttributeOneOf(oo) => {
                validate_attributes(v, &oo.attributes, &format!("oneof '{}' ({})", a.name, scope),
                                    msg_by_name, enum_names, true);
            }
            AttributeDetails::AttributeSimple(_) => (),
        }
    }
}

/// All oneofs (also nested ones) of a list of attributes: ((oneof attribute name, info, name span), depth).
fn collect_oneofs(attrs: &[Attribute]) -> Vec<((String, &OneOfInfo, SrcSpan), usize)> {
    let mut r = vec![];
    for a in attrs {
        if let AttributeDetails::AttributeOneOf(oo) = &a.specific_details {
            r.push(((a.name.clone(), oo, a.name_span), 0));
            r.extend(collect_oneofs(&oo.attributes).into_iter().map(|(x, d)| (x, d + 1)));
        }
    }
    r
}

/// Message types referenced by a list of attributes (including oneof elements), with the span of the reference.
fn referenced_msgs<'m>(attrs: &'m [Attribute], msg_by_name: &HashMap<&str, &Message>, out: &mut Vec<(&'m str, SrcSpan)>) {
    for a in attrs {
        match &a.specific_details {
            AttributeDetails::AttributeEnumOrMsg(t) if msg_by_name.contains_key(t.as_str()) => out.push((t.as_str(), a.type_span)),
            AttributeDetails::AttributeOneOf(oo) => referenced_msgs(&oo.attributes, msg_by_name, out),
            _ => (),
        }
    }
}

/// Finds cycles in the message dependency graph. Returns (span of the reference closing the cycle, cycle path).
fn find_cycles(msgs: &[&Message], msg_by_name: &HashMap<&str, &Message>) -> Vec<(SrcSpan, Vec<String>)> {
    #[derive(Clone, Copy, PartialEq)]
    enum State { New, Active, Done }
    let mut state: HashMap<&str, State> = msgs.iter().map(|m| (m.name.as_str(), State::New)).collect();
    let mut cycles = vec![];

    fn visit<'m>(name: &'m str, msg_by_name: &HashMap<&str, &'m Message>, state: &mut HashMap<&'m str, State>,
                 stack: &mut Vec<&'m str>, cycles: &mut Vec<(SrcSpan, Vec<String>)>) {
        state.insert(name, State::Active);
        stack.push(name);
        let mut refs = vec![];
        referenced_msgs(&msg_by_name[name].attributes, msg_by_name, &mut refs);
        for (r, span) in refs {
            match state.get(r).copied().unwrap_or(State::Done) {
                State::New => visit(r, msg_by_name, state, stack, cycles),
                State::Active => {
                    let pos = stack.iter().position(|s| *s == r).unwrap_or(0);
                    let mut path: Vec<String> = stack[pos..].iter().map(|s| s.to_string()).collect();
                    path.push(r.to_string());
                    cycles.push((span, path));
                }
                State::Done => (),
            }
        }
        stack.pop();
        state.insert(name, State::Done);
    }
    for m in msgs {
        if state.get(m.name.as_str()) == Some(&State::New) {
            visit(m.name.as_str(), msg_by_name, &mut state, &mut vec![], &mut cycles);
        }
    }
    cycles
}

/// True if a message always encodes to 0 bits (min_bits == 0). Requires an acyclic graph.
fn is_zero_bit_msg(name: &str, msg_by_name: &HashMap<&str, &Message>, cache: &mut HashMap<String, bool>) -> bool {
    if let Some(r) = cache.get(name) { return *r; }
    let Some(m) = msg_by_name.get(name) else { return false; };
    let r = m.attributes.iter().all(|a| {
        if a.is_optional || matches!(a.is_repeated_and_size, Some(DynOrFixedType::Dyn(_))) { return false; }
        match &a.specific_details {
            AttributeDetails::AttributeEnumOrMsg(t) => msg_by_name.contains_key(t.as_str()) && is_zero_bit_msg(t, msg_by_name, cache),
            _ => false,
        }
    });
    cache.insert(name.to_string(), r);
    r
}

fn check_repeated_dyn_zero(v: &mut Validator, attrs: &[Attribute], msg_by_name: &HashMap<&str, &Message>,
                           cache: &mut HashMap<String, bool>) {
    for a in attrs {
        match &a.specific_details {
            AttributeDetails::AttributeEnumOrMsg(t) if matches!(a.is_repeated_and_size, Some(DynOrFixedType::Dyn(_))) => {
                if is_zero_bit_msg(t, msg_by_name, cache) {
                    v.err(a.type_span, format!(
                        "repeated_dyn of message '{t}' is not allowed: '{t}' encodes to 0 bits (empty message)"));
                }
            }
            AttributeDetails::AttributeOneOf(oo) => check_repeated_dyn_zero(v, &oo.attributes, msg_by_name, cache),
            _ => (),
        }
    }
}

// ************************************************************************
// Processing
// ************************************************************************

#[derive(Debug, Clone)]
pub struct BitisProcessed {
    pub max_version_number: u16,
    pub msgs: Vec<Message>,
    pub enums: Vec<Enum>,
    /// All oneofs in declaration order: (message name, oneof).
    pub oo_enums: Vec<(String, OneOfInfo)>,
}

/// Validates the parsed definitions and prepares messages and enums for rendering.
pub fn process_and_validate_bitis(parsed_bitis: &[Value]) -> Result<BitisProcessed, Vec<CompileError>> {
    process_and_validate_bitis_with_sources(parsed_bitis, None)
}

/// Like [`process_and_validate_bitis`], `files` improves "first defined at" hints in error messages.
pub fn process_and_validate_bitis_with_sources(parsed_bitis: &[Value], files: Option<&[SourceFile]>)
    -> Result<BitisProcessed, Vec<CompileError>> {
    let errors = validate_bitis(parsed_bitis, files);
    if !errors.is_empty() { return Err(errors); }

    let msgs: Vec<_> = parsed_bitis.iter().filter_map(|v| match v { Value::Message(m) => Some(m.clone()), _ => None }).collect();
    let enums: Vec<_> = parsed_bitis.iter().filter_map(|v| match v { Value::Enum(e) => Some(e.clone()), _ => None }).collect();

    fn get_oneofs(msg_name: &str, attrs: &[Attribute], out: &mut Vec<(String, OneOfInfo)>) {
        let direct: Vec<&OneOfInfo> = attrs.iter().filter_map(|a| match &a.specific_details {
            AttributeDetails::AttributeOneOf(oo) => Some(oo), _ => None }).collect();
        out.extend(direct.iter().map(|oo| (msg_name.to_string(), (*oo).clone())));
        for oo in direct { get_oneofs(msg_name, &oo.attributes, out); }
    }
    let mut oo_enums = vec![];
    for m in &msgs { get_oneofs(&m.name, &m.attributes, &mut oo_enums); }

    Ok(BitisProcessed { max_version_number: 0, msgs, enums, oo_enums })
}

/// Parses and validates all sources.
pub fn compile_sources(files: &[SourceFile]) -> Result<BitisProcessed, Vec<CompileError>> {
    let values = parse_sources(files).map_err(|e| vec![e])?;
    process_and_validate_bitis_with_sources(&values, Some(files))
}

/// Human readable precision of all fixed point attributes (debug output of the compiler).
pub fn fixed_point_summary(p: &BitisProcessed) -> String {
    let mut s = String::new();
    for m in &p.msgs {
        for a in &m.attributes {
            if let AttributeDetails::AttributeSimple(SimpleType::FixedPrecision(fpp)) = a.specific_details {
                let values_number = (1_u64 << fpp.bits) - 3;
                let prec = (fpp.max_val as f64 - fpp.min_val as f64) / values_number as f64;
                let _ = writeln!(s, "{}::{} => [{} : {}] precision: {}", m.name, a.name, fpp.min_val, fpp.max_val, prec);
            }
        }
    }
    s
}

// ************************************************************************
// Schema hash
// ************************************************************************

/// Schema text of a simple type, e.g. `uint_12` or `fp_10[-2,3]`.
pub fn canonical_simple_type(t: &SimpleType) -> String {
    match t {
        SimpleType::Bool => "bool".into(),
        SimpleType::UIntFixed(b) => format!("uint_{b}"),
        SimpleType::IntFixed(b) => format!("int_{b}"),
        SimpleType::UIntDyn((x, y)) => format!("uint_{x}d{y}"),
        SimpleType::IntDyn((x, y)) => format!("int_{x}d{y}"),
        SimpleType::Float => "float".into(),
        SimpleType::Double => "double".into(),
        SimpleType::FixedPrecision(f) => format!("fp_{}[{},{}]", f.bits, f.min_val, f.max_val),
        SimpleType::Binary(b, None) => format!("binary_d{b}"),
        SimpleType::Binary(b, Some(m)) => format!("binary_d{b}[{m}]"),
        SimpleType::AString(b, None) => format!("astr_d{b}"),
        SimpleType::AString(b, Some(m)) => format!("astr_d{b}[{m}]"),
    }
}
fn canonical_attribute(a: &Attribute, out: &mut String) {
    match &a.specific_details {
        AttributeDetails::AttributeOneOf(oo) => {
            let _ = write!(out, "oneof {}({}){{", a.name, oo.dyn_bits);
            for va in &oo.attributes {
                if va.name == oo.default_attrib_name { out.push('*'); }
                canonical_attribute(va, out);
                out.push(';');
            }
            out.push('}');
        }
        d => {
            if a.is_optional { out.push_str("optional "); }
            match a.is_repeated_and_size {
                Some(DynOrFixedType::Fixed(n)) => { let _ = write!(out, "repeated_fixed_{n} "); }
                Some(DynOrFixedType::Dyn(n)) => {
                    let _ = write!(out, "repeated_dyn_{n}");
                    if let Some(m) = a.repeated_max_len { let _ = write!(out, "[{m}]"); }
                    out.push(' ');
                }
                None => (),
            }
            match d {
                AttributeDetails::AttributeSimple(t) => out.push_str(&canonical_simple_type(t)),
                AttributeDetails::AttributeEnumOrMsg(t) => out.push_str(t),
                AttributeDetails::AttributeOneOf(_) => unreachable!(),
            }
            let _ = write!(out, " {}", a.name);
        }
    }
}

/// Canonical description of a schema, the input of the schema hash (see docs/WIRE_FORMAT.md, "Schema hash").
///
/// First line `bitis-schema-v1`, then one line per message / enum, sorted by name (byte order):
/// `enum NAME(Y){A,*B,C}` and `msg NAME{ATTR;...}` / `msg NAME:PARENT{ATTR;...}` with
/// `ATTR = [optional |repeated_fixed_N |repeated_dyn_Y[[M]] ]TYPE NAME` or `oneof NAME(Y){[*]ATTR;...}`
/// (bounds `[M]` only where declared, so schemas without bounds keep their hash).
/// Every line ends with '\n'. Comments, whitespace and the split into input files do not matter.
pub fn canonical_schema(p: &BitisProcessed) -> String {
    let mut defs: Vec<(&str, String)> = vec![];
    for e in &p.enums {
        let vals: Vec<String> = e.values.iter().map(|v| if *v == e.default { format!("*{v}") } else { v.clone() }).collect();
        defs.push((&e.name, format!("enum {}({}){{{}}}", e.name, e.bit_size, vals.join(","))));
    }
    for m in &p.msgs {
        let mut s = format!("msg {}", m.name);
        if let Some(parent) = &m.parent { let _ = write!(s, ":{parent}"); }
        s.push('{');
        for a in &m.attributes { canonical_attribute(a, &mut s); s.push(';'); }
        s.push('}');
        defs.push((&m.name, s));
    }
    defs.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    let mut out = String::from("bitis-schema-v1\n");
    for (_, d) in defs { out.push_str(&d); out.push('\n'); }
    out
}

/// Schema hash: the first 8 bytes (big endian) of SHA-256 over [`canonical_schema`].
pub fn schema_hash(p: &BitisProcessed) -> u64 {
    let digest = Sha256::digest(canonical_schema(p).as_bytes());
    u64::from_be_bytes(digest[..8].try_into().unwrap())
}
/// [`schema_hash`] as 16 lower case hex digits without prefix (template variable `schema_hash`).
pub fn schema_hash_hex(p: &BitisProcessed) -> String {
    format!("{:016x}", schema_hash(p))
}

// ************************************************************************
// min_bits (docs/WIRE_FORMAT.md section 4)
// ************************************************************************

/// `min_bits` of a simple type (spec section 4).
pub fn min_bits_simple(t: &SimpleType) -> u64 {
    match t {
        SimpleType::Bool => 1,
        SimpleType::UIntFixed(x) => *x as u64,
        SimpleType::IntFixed(x) => *x as u64 + 1,
        SimpleType::UIntDyn(_) => 1,
        SimpleType::IntDyn(_) => 2,
        SimpleType::Float => 32,
        SimpleType::Double => 64,
        SimpleType::FixedPrecision(f) => f.bits as u64,
        SimpleType::AString(..) | SimpleType::Binary(..) => 1,
    }
}
/// `min_bits` of a type without its modifier (an element of a `repeated_dyn`): simple types, enums (1),
/// messages (sum of their attributes) and oneofs (1 + minimum over the elements).
/// The message graph must be acyclic (guaranteed by the validation).
pub fn min_bits_details(d: &AttributeDetails, msgs: &[Message]) -> u64 {
    match d {
        AttributeDetails::AttributeSimple(t) => min_bits_simple(t),
        AttributeDetails::AttributeEnumOrMsg(n) => match msgs.iter().find(|m| m.name == *n) {
            Some(m) => min_bits_message(m, msgs),
            None => 1,
        },
        AttributeDetails::AttributeOneOf(oo) =>
            1 + oo.attributes.iter().map(|a| min_bits_attribute(a, msgs)).min().unwrap_or(0),
    }
}
/// `min_bits` of an attribute including its modifier.
pub fn min_bits_attribute(a: &Attribute, msgs: &[Message]) -> u64 {
    if a.is_optional { return 1; }
    match a.is_repeated_and_size {
        Some(DynOrFixedType::Dyn(_)) => 1,
        Some(DynOrFixedType::Fixed(n)) => n as u64 * min_bits_details(&a.specific_details, msgs),
        None => min_bits_details(&a.specific_details, msgs),
    }
}
/// `min_bits` of a message: the sum over its attributes.
pub fn min_bits_message(m: &Message, msgs: &[Message]) -> u64 {
    m.attributes.iter().map(|a| min_bits_attribute(a, msgs)).sum()
}

// ************************************************************************
// max_bits (docs/WIRE_FORMAT.md section 4)
// ************************************************************************

pub use super::berde::{dyn_bits, dyn_max_bits};

/// `max_bits` of a simple type (spec section 4), `None` = unbounded.
pub fn max_bits_simple(t: &SimpleType) -> Option<u64> {
    Some(match t {
        SimpleType::Bool => 1,
        SimpleType::UIntFixed(x) => *x as u64,
        SimpleType::IntFixed(x) => *x as u64 + 1,
        SimpleType::UIntDyn((x, y)) => dyn_max_bits(*x, *y),
        SimpleType::IntDyn((x, y)) => 1 + dyn_max_bits(*x, *y),
        SimpleType::Float => 32,
        SimpleType::Double => 64,
        SimpleType::FixedPrecision(f) => f.bits as u64,
        SimpleType::AString(y, m) | SimpleType::Binary(y, m) => {
            let m = (*m)? as u64;
            dyn_bits(m, 32, *y) + 8 * m
        }
    })
}
/// `max_bits` of a type without its modifier: simple types, enums (`dyn(k - 1, 32, Y)`),
/// messages (sum of their attributes) and oneofs. `None` = unbounded (or overflow).
/// The message graph must be acyclic (guaranteed by the validation).
pub fn max_bits_details(d: &AttributeDetails, msgs: &[Message], enums: &[Enum]) -> Option<u64> {
    match d {
        AttributeDetails::AttributeSimple(t) => max_bits_simple(t),
        AttributeDetails::AttributeEnumOrMsg(n) => match msgs.iter().find(|m| m.name == *n) {
            Some(m) => max_bits_message(m, msgs, enums),
            None => {
                let e = enums.iter().find(|e| e.name == *n)?;
                Some(dyn_bits(e.values.len().saturating_sub(1) as u64, 32, e.bit_size))
            }
        },
        AttributeDetails::AttributeOneOf(oo) => {
            let mut m: u64 = 0;
            for a in &oo.attributes { m = m.max(max_bits_attribute(a, msgs, enums)?); }
            dyn_bits(oo.attributes.len().saturating_sub(1) as u64, 32, oo.dyn_bits).checked_add(m)
        }
    }
}
/// `max_bits` of an attribute including its modifier, `None` = unbounded.
pub fn max_bits_attribute(a: &Attribute, msgs: &[Message], enums: &[Enum]) -> Option<u64> {
    let t = max_bits_details(&a.specific_details, msgs, enums)?;
    let r = match a.is_repeated_and_size {
        Some(DynOrFixedType::Dyn(y)) => {
            let m = a.repeated_max_len? as u64;
            dyn_bits(m, 32, y).checked_add(t.checked_mul(m)?)?
        }
        Some(DynOrFixedType::Fixed(n)) => t.checked_mul(n as u64)?,
        None => t,
    };
    if a.is_optional { r.checked_add(1) } else { Some(r) }
}
/// `max_bits` of a message: the sum over its attributes, `None` = unbounded.
pub fn max_bits_message(m: &Message, msgs: &[Message], enums: &[Enum]) -> Option<u64> {
    let mut s: u64 = 0;
    for a in &m.attributes { s = s.checked_add(max_bits_attribute(a, msgs, enums)?)?; }
    Some(s)
}

// ************************************************************************
// Data for the templates
// ************************************************************************

fn integer_bit_size(bit_size: &u8) -> u8 {
    match bit_size {
        0..=16 => 16,
        17..=32 => 32,
        _ => 64,
    }
}

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct AttributeEx {
    pub base: Attribute,
    pub rust_type_str: String,
    pub base_type_str: String,
    pub is_py_wrapped: bool,
    pub is_msg: bool,
    pub is_enum: bool,
    pub is_oo: bool,
    pub add_val: bool,
    /// The attribute has a maximum length `[M]` (on the type or on its `repeated_dyn` modifier).
    pub is_bounded: bool,
    /// Full Rust type of the attribute including its modifier (e.g. `BitisOption<Binary<4>>`), with
    /// message / enum / oneof types qualified as `messages::NAME` (used by the Python bindings);
    /// only set for the Rust target.
    pub rust_attr_type: String,
}
impl AttributeEx {
    /// Python type of one element (without optional / repeated), as used in the `.pyi` stub.
    pub fn py_elem_type(&self) -> String {
        match &self.base.specific_details {
            AttributeDetails::AttributeSimple(t) => match t {
                SimpleType::Bool => "bool",
                SimpleType::UIntFixed(_) | SimpleType::IntFixed(_) | SimpleType::UIntDyn(_) | SimpleType::IntDyn(_) => "int",
                SimpleType::Float | SimpleType::Double | SimpleType::FixedPrecision(_) => "float",
                SimpleType::Binary(..) => "bytes",
                SimpleType::AString(..) => "str",
            }.to_string(),
            AttributeDetails::AttributeEnumOrMsg(name) => name.clone(),
            AttributeDetails::AttributeOneOf(oo) => oo.name.clone(),
        }
    }
    /// Python type of the attribute (`list[T]` for repeated, `T | None` for optional).
    pub fn py_type(&self) -> String {
        let mut t = self.py_elem_type();
        if self.base.is_repeated_and_size.is_some() { t = format!("list[{t}]"); }
        if self.base.is_optional { t = format!("{t} | None"); }
        t
    }
    /// Comment for the `.pyi` stub: the schema type and, for fixed point, how Overflow / Underflow are read back.
    pub fn py_comment(&self) -> String {
        let mut s = String::new();
        canonical_attribute(&self.base, &mut s);
        let mut s = s.strip_suffix(&format!(" {}", self.base.name)).unwrap_or(&s).to_string();
        if let AttributeDetails::AttributeSimple(SimpleType::FixedPrecision(f)) = &self.base.specific_details {
            let _ = write!(s, "; above {}: Overflow, read back as {}.0; below {} or NaN: Underflow, read back as {}.0",
                           f.max_val, f.max_val as i128 + 1, f.min_val, f.min_val as i128 - 1);
        }
        s
    }
}
#[derive(Debug, Clone)]
pub struct MessageR {
    pub name: String,
    pub comment: Option<String>,
    pub parent: Option<String>,
    pub attributes: Vec<AttributeEx>,
    /// At least one attribute has a maximum length `[M]` (Python: checked setters).
    pub has_bounds: bool,
}
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct OneOfInfoR {
    pub msg_name: String,
    pub name: String,
    pub dyn_bits: u8,
    pub attributes: Vec<AttributeEx>,
    pub default_attrib_name: String,
}

#[derive(Clone, Copy, PartialEq)]
enum TargetLang { Rust, Cpp }

fn to_target_attribute(attribute: &Attribute, msg_names: &[String], lang: TargetLang) -> AttributeEx {
    let (mut is_py_wrapped, mut is_enum, mut is_msg, mut is_oo, mut add_val) = (false, false, false, false, false);
    let int_base = |signed: bool, bits: &u8| match (lang, signed) {
        (TargetLang::Rust, false) => format!("u{}", integer_bit_size(bits)),
        (TargetLang::Rust, true) => format!("i{}", integer_bit_size(bits)),
        (TargetLang::Cpp, false) => format!("uint{}_t", integer_bit_size(bits)),
        (TargetLang::Cpp, true) => format!("int{}_t", integer_bit_size(bits)),
    };
    let int_fixed = if lang == TargetLang::Rust { "IntWithGivenBitSize" } else { "IntgralWithGivenBitSize" };
    let (rtype, base_type) = match &attribute.specific_details {
        AttributeDetails::AttributeSimple(a) => match a {
            SimpleType::Bool => match lang {
                TargetLang::Rust => ("bool".to_string(), "bool".to_string()),
                TargetLang::Cpp => ("BitisBool".to_string(), "bool".to_string()),
            },
            SimpleType::UIntFixed(b) | SimpleType::IntFixed(b) => {
                add_val = true;
                let base = int_base(matches!(a, SimpleType::IntFixed(_)), b);
                (format!("{int_fixed}<{base}, {b}>"), base)
            }
            SimpleType::UIntDyn((x, y)) | SimpleType::IntDyn((x, y)) => {
                add_val = true;
                let base = int_base(matches!(a, SimpleType::IntDyn(_)), x);
                (format!("DynInteger<{base}, {x}, {y}>"), base)
            }
            SimpleType::Float => {
                add_val = true;
                match lang {
                    TargetLang::Rust => ("f32".to_string(), "f32".to_string()),
                    TargetLang::Cpp => ("BitisFloatingPoint<float>".to_string(), "float".to_string()),
                }
            }
            SimpleType::Double => match lang {
                TargetLang::Rust => ("f64".to_string(), "f64".to_string()),
                TargetLang::Cpp => ("BitisFloatingPoint<double>".to_string(), "double".to_string()),
            },
            SimpleType::FixedPrecision(fpp) => {
                add_val = true;
                (format!("FixPrecisionMinMax<{}, {}, {}>", fpp.bits, fpp.min_val, fpp.max_val),
                 if lang == TargetLang::Rust { "f64" } else { "double" }.to_string())
            }
            SimpleType::Binary(b, max_len) => {
                add_val = true;
                (match max_len { None => format!("Binary<{b}>"), Some(m) => format!("BoundedBinary<{b}, {m}>") },
                 "Vec<u8>".to_string())
            }
            SimpleType::AString(b, max_len) => {
                add_val = true;
                (match max_len { None => format!("BitisAString<{b}>"), Some(m) => format!("BoundedAString<{b}, {m}>") },
                 if lang == TargetLang::Rust { "String" } else { "char *" }.to_string())
            }
        },
        AttributeDetails::AttributeEnumOrMsg(em) => {
            is_py_wrapped = true;
            is_msg = msg_names.contains(em);
            is_enum = !is_msg;
            (em.clone(), em.clone())
        }
        AttributeDetails::AttributeOneOf(ooi) => {
            is_py_wrapped = true;
            is_oo = true;
            (ooi.name.clone(), ooi.name.clone())
        }
    };
    let is_bounded = attribute.repeated_max_len.is_some()
        || matches!(attribute.specific_details,
                    AttributeDetails::AttributeSimple(SimpleType::AString(_, Some(_)) | SimpleType::Binary(_, Some(_))));
    let rust_attr_type = if lang == TargetLang::Rust {
        let rtype = if is_py_wrapped { format!("messages::{rtype}") } else { rtype.clone() };
        let t = match (&attribute.is_repeated_and_size, attribute.repeated_max_len) {
            (Some(DynOrFixedType::Dyn(y)), Some(m)) => format!("BoundedDynArray<{rtype}, {y}, {m}>"),
            (Some(DynOrFixedType::Dyn(y)), None) => format!("DynArray<{rtype}, {y}>"),
            (Some(DynOrFixedType::Fixed(n)), _) => format!("FixedArray<{rtype}, {n}>"),
            (None, _) => rtype.clone(),
        };
        if attribute.is_optional { format!("BitisOption<{t}>") } else { t }
    } else { String::new() };
    AttributeEx { base: attribute.clone(), rust_type_str: rtype, base_type_str: base_type,
        is_py_wrapped, is_msg, is_enum, is_oo, add_val, is_bounded, rust_attr_type }
}

fn to_target_messages(msgs: &[Message], lang: TargetLang) -> Vec<MessageR> {
    let msg_names: Vec<_> = msgs.iter().map(|m| m.name.clone()).collect();
    msgs.iter().map(|msg| {
        let attributes: Vec<AttributeEx> = msg.attributes.iter().map(|a| to_target_attribute(a, &msg_names, lang)).collect();
        MessageR {
            name: msg.name.clone(), comment: msg.comment.clone(), parent: msg.parent.clone(),
            has_bounds: attributes.iter().any(|a| a.is_bounded), attributes,
        }
    }).collect()
}
fn to_target_oneofs(oos: &[(String, OneOfInfo)], msgs: &[Message], lang: TargetLang) -> IndexMap<String, OneOfInfoR> {
    let msg_names: Vec<_> = msgs.iter().map(|m| m.name.clone()).collect();
    oos.iter().map(|(msg_name, oo)| {
        (oo.name.clone(), OneOfInfoR {
            msg_name: msg_name.clone(), name: oo.name.clone(), dyn_bits: oo.dyn_bits,
            attributes: oo.attributes.iter().map(|a| to_target_attribute(a, &msg_names, lang)).collect(),
            default_attrib_name: oo.default_attrib_name.clone(),
        })
    }).collect()
}

pub fn to_rust_messages(msgs: &[Message]) -> Vec<MessageR> { to_target_messages(msgs, TargetLang::Rust) }
pub fn to_rust_oneofs(oos: &[(String, OneOfInfo)], msgs: &[Message]) -> IndexMap<String, OneOfInfoR> {
    to_target_oneofs(oos, msgs, TargetLang::Rust)
}
pub fn to_cpp_messages(msgs: &[Message]) -> Vec<MessageR> { to_target_messages(msgs, TargetLang::Cpp) }
pub fn to_cpp_oneofs(oos: &[(String, OneOfInfo)], msgs: &[Message]) -> IndexMap<String, OneOfInfoR> {
    to_target_oneofs(oos, msgs, TargetLang::Cpp)
}

#[derive(Clone, Debug)]
pub struct JinjaData {
    pub enums: Vec<Enum>,
    pub msgs: Vec<MessageR>,
    /// Oneofs in declaration order, keyed by their generated type name.
    pub oos: IndexMap<String, OneOfInfoR>,
}
impl JinjaData {
    pub fn for_rust(p: &BitisProcessed) -> Self {
        JinjaData { enums: p.enums.clone(), msgs: to_rust_messages(&p.msgs), oos: to_rust_oneofs(&p.oo_enums, &p.msgs) }
    }
    pub fn for_cpp(p: &BitisProcessed) -> Self {
        JinjaData { enums: p.enums.clone(), msgs: to_cpp_messages(&p.msgs), oos: to_cpp_oneofs(&p.oo_enums, &p.msgs) }
    }
}

#[derive(Template, Clone, Debug)]
#[template(path = "data_objects.rs.jinja", escape = "none")]
pub struct RustDataObjects {
    pub d: JinjaData,
    pub bitis_version: String,
    pub schema_hash: String,
}
#[derive(Template, Clone, Debug)]
#[template(path = "pyclasses.py.rs.jinja", escape = "none")]
pub struct RustPyDataObjects {
    pub d: JinjaData,
    /// Python module name (used as `__module__` of the exception types).
    pub lib_name: String,
}
#[derive(Template, Clone, Debug)]
#[template(path = "pylib.py.rs.jinja", escape = "none")]
pub struct RustPyLib {
    pub d: JinjaData,
    pub lib_name: String,
}
#[derive(Template, Clone, Debug)]
#[template(path = "py_type_hints.pyi.jinja", escape = "none")]
pub struct PyTypeHints {
    pub d: JinjaData,
    pub bitis_version: String,
    pub schema_hash: String,
}
#[derive(Template, Clone, Debug)]
#[template(path = "data_objects.cpp.jinja", escape = "none")]
pub struct CppDataObjects {
    pub d: JinjaData,
    pub object_order: Vec<String>,
    pub bitis_header_lib_file_name: String,
    pub bitis_version: String,
    pub schema_hash: String,
    /// Suffix of the namespace `bitis_schema_<stamp_name>` holding the schema stamp.
    pub stamp_name: String,
}

#[derive(Template, Clone, Debug)]
#[template(path = "data_objects.dart.jinja", escape = "none")]
pub struct DartDataObjects {
    pub d: super::dart::DartData,
    pub bitis_version: String,
    pub schema_hash: String,
}

/// Generated files of the Python target.
#[derive(Clone, Debug)]
pub struct PythonOutput {
    /// `src/messages.rs`
    pub messages_rs: String,
    /// `src/pyrust.rs`
    pub pyrust_rs: String,
    /// `src/lib.rs`
    pub lib_rs: String,
    /// `<lib_name>/<lib_name>.pyi`
    pub pyi: String,
}

fn render_err(e: askama::Error) -> CompileError { CompileError::without_span(format!("internal template error: {e}")) }

/// Renders the Rust code for a schema.
pub fn render_rust(p: &BitisProcessed) -> Result<String, CompileError> {
    RustDataObjects { d: JinjaData::for_rust(p), bitis_version: BITIS_COMPILER_VERSION.to_string(),
        schema_hash: schema_hash_hex(p) }.render().map_err(render_err)
}
/// Renders the C++ header for a schema. `stamp_name` must be a valid C++ identifier part (see [`stamp_name_from`]).
pub fn render_cpp(p: &BitisProcessed, bitis_header_lib_file_name: &str, stamp_name: &str) -> Result<String, CompileError> {
    let d = JinjaData::for_cpp(p);
    let object_order = dependencies_process(d.clone());
    CppDataObjects { d, object_order, bitis_header_lib_file_name: bitis_header_lib_file_name.to_string(),
        bitis_version: BITIS_COMPILER_VERSION.to_string(), schema_hash: schema_hash_hex(p),
        stamp_name: stamp_name.to_string() }.render().map_err(render_err)
}
/// Renders the files of the Python (pyo3/maturin) target.
pub fn render_python(p: &BitisProcessed, lib_name: &str) -> Result<PythonOutput, CompileError> {
    let d = JinjaData::for_rust(p);
    Ok(PythonOutput {
        messages_rs: render_rust(p)?,
        pyrust_rs: RustPyDataObjects { d: d.clone(), lib_name: lib_name.to_string() }.render().map_err(render_err)?,
        lib_rs: RustPyLib { d: d.clone(), lib_name: lib_name.to_string() }.render().map_err(render_err)?,
        pyi: PyTypeHints { d, bitis_version: BITIS_COMPILER_VERSION.to_string(), schema_hash: schema_hash_hex(p) }
            .render().map_err(render_err)?,
    })
}
/// Renders the Dart library for a schema (one file, imports only `package:bitis/bitis.dart` and `dart:typed_data`).
pub fn render_dart(p: &BitisProcessed) -> Result<String, CompileError> {
    let mut s = DartDataObjects { d: super::dart::dart_data(p, schema_hash(p)),
        bitis_version: BITIS_COMPILER_VERSION.to_string(), schema_hash: schema_hash_hex(p) }.render().map_err(render_err)?;
    s.push('\n');
    Ok(s)
}
/// Derives the identifier used in the C++ stamp namespace from a file name (non alphanumeric chars -> '_').
pub fn stamp_name_from(file_stem: &str) -> String {
    let s: String = file_stem.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect();
    if s.is_empty() { "messages".to_string() } else { s }
}

mod filters {
    use std::fmt::Display;

    fn doc_lines(s: &str) -> impl Iterator<Item = &str> { s.split('\n').map(|l| l.trim_end()) }

    /// Rust doc comment (`/// ...` lines), every line prefixed with `indent` and terminated by '\n'.
    pub fn rs_doc<T: Display>(s: T, _: &dyn askama::Values, indent: &str) -> askama::Result<String> {
        let mut out = String::new();
        for l in doc_lines(&s.to_string()) {
            out.push_str(indent);
            out.push_str("///");
            if !l.is_empty() { out.push(' '); out.push_str(l); }
            out.push('\n');
        }
        Ok(out)
    }
    /// C++ doc comment, like `rs_doc` but trailing backslashes (line splicing) are removed.
    pub fn cpp_doc<T: Display>(s: T, v: &dyn askama::Values, indent: &str) -> askama::Result<String> {
        let s = s.to_string();
        let cleaned: Vec<&str> = doc_lines(&s).map(|l| l.trim_end_matches(|c: char| c == '\\' || c.is_whitespace())).collect();
        rs_doc(cleaned.join("\n"), v, indent)
    }
    /// Python doc string (`"""..."""`), every line prefixed with `indent` and terminated by '\n'.
    pub fn py_doc<T: Display>(s: T, _: &dyn askama::Values, indent: &str) -> askama::Result<String> {
        let s = s.to_string().replace('\\', "\\\\").replace("\"\"\"", "\\\"\\\"\\\"");
        let mut out = format!("{indent}\"\"\"\n");
        for l in doc_lines(&s) {
            if !l.is_empty() { out.push_str(indent); out.push_str(l); }
            out.push('\n');
        }
        out.push_str(indent);
        out.push_str("\"\"\"\n");
        Ok(out)
    }
    /// Dart doc comment (`/// ...` lines), every line prefixed with `indent` spaces, no trailing newline.
    pub fn dart_doc<T: Display>(s: T, _: &dyn askama::Values, indent: usize) -> askama::Result<String> {
        Ok(super::super::dart::dart_doc(&s.to_string(), indent))
    }
    pub fn snake_case<T: Display>(s: T, _: &dyn askama::Values) -> askama::Result<String> {
        Ok(stringcase::snake_case(s.to_string().as_str()))
    }
    #[allow(dead_code)]
    pub fn camel_case<T: Display>(s: T, _: &dyn askama::Values) -> askama::Result<String> {
        Ok(stringcase::camel_case(s.to_string().as_str()))
    }
    pub fn pascal_case<T: Display>(s: T, _: &dyn askama::Values) -> askama::Result<String> {
        Ok(stringcase::pascal_case(s.to_string().as_str()))
    }
}

// ************************************************************************
// C++ declaration order
// ************************************************************************

/// Order of the C++ declarations: every type after the types it depends on.
/// Enums first (declaration order), then messages in declaration order, each preceded by its
/// not yet emitted dependencies (depth first); a oneof directly precedes its message.
/// The message graph must be acyclic (guaranteed by the validation); revisits are skipped.
pub fn dependencies_process(jd: JinjaData) -> Vec<String> {
    let msgs: HashMap<&str, &MessageR> = jd.msgs.iter().map(|m| (m.name.as_str(), m)).collect();
    let mut order: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for e in &jd.enums {
        if seen.insert(e.name.clone()) { order.push(e.name.clone()); }
    }
    fn visit_attrs(attrs: &[AttributeEx], jd: &JinjaData, msgs: &HashMap<&str, &MessageR>,
                   seen: &mut HashSet<String>, order: &mut Vec<String>) {
        for a in attrs {
            if a.is_oo {
                let unseen = jd.oos.get(&a.rust_type_str).filter(|oo| seen.insert(oo.name.clone()));
                if let Some(oo) = unseen {
                    visit_attrs(&oo.attributes, jd, msgs, seen, order);
                    order.push(oo.name.clone());
                }
            } else if a.is_msg {
                visit_msg(&a.rust_type_str, jd, msgs, seen, order);
            }
        }
    }
    fn visit_msg(name: &str, jd: &JinjaData, msgs: &HashMap<&str, &MessageR>, seen: &mut HashSet<String>,
                 order: &mut Vec<String>) {
        if !seen.insert(name.to_string()) { return; }
        if let Some(m) = msgs.get(name) { visit_attrs(&m.attributes, jd, msgs, seen, order); }
        order.push(name.to_string());
    }
    for m in &jd.msgs {
        visit_msg(&m.name, &jd, &msgs, &mut seen, &mut order);
    }
    order
}

// ************************************************************************
// Tests
// ************************************************************************

#[cfg(test)]
mod bitis_semantic {
    use super::*;

    fn parse(s: &str) -> Vec<Value> {
        match parse_source(s, 0) {
            Ok(v) => v,
            Err(e) => panic!("{}", e.render(&[SourceFile { name: "test".into(), content: s.into() }])),
        }
    }

    #[test]
    fn msg_empty_msg() {
        let parsed = parse("msg Lala { }");
        assert_eq!(parsed.len(), 1);
        let Value::Message(msg) = &parsed[0] else { panic!("expected message") };
        assert_eq!(msg.name, "Lala");
        let p = process_and_validate_bitis(&parsed).unwrap();
        assert_eq!(p.msgs.len(), 1);
        assert_eq!(p.enums.len(), 0);
    }

    #[test]
    fn msg_simple_msg() {
        let parsed = parse("msg Lala { uint_7 a1; }");
        let Value::Message(msg) = &parsed[0] else { panic!("expected message") };
        assert_eq!(msg.attributes.len(), 1);
        assert_eq!(msg.attributes[0].name, "a1");
        let AttributeDetails::AttributeSimple(s) = &msg.attributes[0].specific_details else { panic!() };
        assert_eq!(*s, SimpleType::UIntFixed(7));
    }

    #[test]
    fn msg_simple_enum() {
        let parsed = parse("enum Lala(4) { *one, two }");
        let Value::Enum(enm) = &parsed[0] else { panic!("expected enum") };
        assert_eq!(enm.values, vec!["one".to_string(), "two".to_string()]);
        assert_eq!(enm.default, "one");
    }

    #[test]
    fn various_attribute_types() {
        for (src, expected) in [
            ("float", SimpleType::Float), ("uint_12", SimpleType::UIntFixed(12)),
            ("uint_32d4", SimpleType::UIntDyn((32, 4))), ("int_64d64", SimpleType::IntDyn((64, 64))),
            ("bool", SimpleType::Bool), ("binary_d4", SimpleType::Binary(4, None)), ("astr_d32", SimpleType::AString(32, None)),
            ("binary_d4[1]", SimpleType::Binary(4, Some(1))), ("astr_d32 [ 65535 ]", SimpleType::AString(32, Some(65535))),
            ("fp_10[ -2 , 3 ]", SimpleType::FixedPrecision(FixedPrecisionProperties { bits: 10, min_val: -2, max_val: 3 })),
        ] {
            let parsed = parse(&format!("msg Lala {{ {src} attr; }}"));
            let Value::Message(msg) = &parsed[0] else { panic!() };
            let AttributeDetails::AttributeSimple(s) = &msg.attributes[0].specific_details else { panic!() };
            assert_eq!(*s, expected, "{src}");
        }
    }

    #[test]
    fn doc_comments() {
        let parsed = parse("//| line one\n//| line two\n// plain\nmsg Mm {\n //| a doc\n // plain\n bool aa;\n //| dangling\n}\n\
                            enum Ee(2) {\n //| first\n *Aa,\n Bb, //| ignored after last\n}");
        let Value::Message(m) = &parsed[0] else { panic!() };
        assert_eq!(m.comment.as_deref(), Some("line one\nline two"));
        assert_eq!(m.attributes[0].comment.as_deref(), Some("a doc"));
        let Value::Enum(e) = &parsed[1] else { panic!() };
        assert_eq!(e.value_comments, vec![Some("first".to_string()), None]);
    }

    #[test]
    fn canonical_form_independent_of_layout() {
        let a = parse("// c\nenum E(2) { *A, B }\nmsg M { optional uint_3 x; repeated_dyn_2 E e; oneof o(2) { *bool b; M2 m; } }\nmsg M2 { fp_8[-1,1] f; }");
        let b = parse("msg M2{fp_8[ -1 , 1 ] f;}\n//| doc\nmsg M{optional uint_3 x;repeated_dyn_2 E e;oneof o(2){*bool b;M2 m;}}\nenum E(2){*A,B,}");
        let pa = process_and_validate_bitis(&a).unwrap();
        let pb = process_and_validate_bitis(&b).unwrap();
        assert_eq!(canonical_schema(&pa), canonical_schema(&pb));
        assert_eq!(canonical_schema(&pa), "bitis-schema-v1\nenum E(2){*A,B}\nmsg M{optional uint_3 x;repeated_dyn_2 E e;oneof o(2){*bool b;M2 m;};}\nmsg M2{fp_8[-1,1] f;}\n");
        assert_eq!(schema_hash_hex(&pa).len(), 16);
        let c = parse("enum E(2) { A, *B }\nmsg M { optional uint_3 x; repeated_dyn_2 E e; oneof o(2) { *bool b; M2 m; } }\nmsg M2 { fp_8[-1,1] f; }");
        assert_ne!(schema_hash(&pa), schema_hash(&process_and_validate_bitis(&c).unwrap()));
    }

    #[test]
    fn canonical_form_with_bounds() {
        let a = parse("msg M { astr_d4 [ 32 ] s; binary_d2[7] b; repeated_dyn_2 [8] astr_d3[16] l; optional astr_d4[1] o; \
                       repeated_dyn_3 bool u; oneof x(1) { *binary_d5[3] y; } }");
        let pa = process_and_validate_bitis(&a).unwrap();
        let canon = "bitis-schema-v1\nmsg M{astr_d4[32] s;binary_d2[7] b;repeated_dyn_2[8] astr_d3[16] l;optional astr_d4[1] o;\
                     repeated_dyn_3 bool u;oneof x(1){*binary_d5[3] y;};}\n";
        assert_eq!(canonical_schema(&pa), canon);
        // sha256 of the canonical text above (independently computed with sha256sum)
        assert_eq!(schema_hash_hex(&pa), "aab049e4128269e7");
        // the bound is part of the hash, schemas without bounds keep their text
        let unbounded = process_and_validate_bitis(&parse("msg M { astr_d4 s; }")).unwrap();
        let bounded = process_and_validate_bitis(&parse("msg M { astr_d4[32] s; }")).unwrap();
        let bounded2 = process_and_validate_bitis(&parse("msg M { astr_d4[33] s; }")).unwrap();
        assert_eq!(canonical_schema(&unbounded), "bitis-schema-v1\nmsg M{astr_d4 s;}\n");
        assert_ne!(schema_hash(&unbounded), schema_hash(&bounded));
        assert_ne!(schema_hash(&bounded), schema_hash(&bounded2));
        let rep_unbounded = process_and_validate_bitis(&parse("msg M { repeated_dyn_2 bool s; }")).unwrap();
        let rep_bounded = process_and_validate_bitis(&parse("msg M { repeated_dyn_2[4] bool s; }")).unwrap();
        assert_eq!(canonical_schema(&rep_unbounded), "bitis-schema-v1\nmsg M{repeated_dyn_2 bool s;}\n");
        assert_ne!(schema_hash(&rep_unbounded), schema_hash(&rep_bounded));
    }

    #[test]
    fn max_bits_rules() {
        // dyn(v, X, Y) and dynmax(X, Y) = 1 + X + ceil(X / Y) - 1 (spec section 4)
        assert_eq!(dyn_bits(0, 32, 3), 1);
        assert_eq!(dyn_bits(10, 32, 3), 9);
        assert_eq!(dyn_bits(32, 32, 4), 1 + 4 + 1 + 4 + 1);
        for (x, y) in [(32_u8, 4_u8), (8, 3), (64, 64), (64, 7), (1, 1), (16, 5)] {
            assert_eq!(dyn_max_bits(x, y), 1 + x as u64 + (x as u64).div_ceil(y as u64) - 1, "{x}d{y}");
        }
        let p = process_and_validate_bitis(&parse(
            "enum E(2) { *A, B, C, D, E5 }\nenum One(3) { *A }\n\
             msg Inner { uint_3 a; int_7 b; }\n\
             msg M { astr_d4[32] s; repeated_dyn_2[3] Inner i; optional binary_d8[2] o; E e; One one; \
                     oneof x(2) { *bool b; Inner inn; int_32d4 d; } repeated_fixed_2 uint_8d3 f; }\n\
             msg U { repeated_dyn_2 bool u; }\nmsg V { U u; }")).unwrap();
        let m = |n: &str| p.msgs.iter().find(|m| m.name == n).unwrap();
        assert_eq!(max_bits_message(m("Inner"), &p.msgs, &p.enums), Some(3 + 8));
        let expected = (dyn_bits(32, 32, 4) + 8 * 32)          // astr_d4[32]
            + (dyn_bits(3, 32, 2) + 3 * 11)                     // repeated_dyn_2[3] Inner
            + (1 + dyn_bits(2, 32, 8) + 16)                     // optional binary_d8[2]
            + dyn_bits(4, 32, 2)                                // enum with 5 values
            + 1                                                 // enum with 1 value: dyn(0) = 1
            + (dyn_bits(2, 32, 2) + (1 + 32 + 8 - 1 + 1))       // oneof: max(1, 11, int_32d4 = 1 + dynmax)
            + 2 * dyn_max_bits(8, 3);                           // repeated_fixed_2 uint_8d3
        assert_eq!(max_bits_message(m("M"), &p.msgs, &p.enums), Some(expected));
        assert_eq!(max_bits_message(m("U"), &p.msgs, &p.enums), None);
        assert_eq!(max_bits_message(m("V"), &p.msgs, &p.enums), None);
    }

    #[test]
    fn line_col_positions() {
        assert_eq!(line_col("ab\ncd", 0), (1, 1));
        assert_eq!(line_col("ab\ncd", 3), (2, 1));
        assert_eq!(line_col("ab\ncäd", 6), (2, 3));
        assert_eq!(line_col("ab", 10), (1, 3));
    }

    #[test]
    fn deterministic_cpp_order() {
        let p = process_and_validate_bitis(&parse(
            "msg A { B b; oneof o(2) { *C c; E e; } }\nmsg B { E e; }\nmsg C { bool x; }\nenum E(2) { *X }")).unwrap();
        let order = dependencies_process(JinjaData::for_cpp(&p));
        assert_eq!(order, vec!["E", "B", "C", "OO_A_O", "A"]);
    }
}
