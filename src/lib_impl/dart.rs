//! Dart target: type mapping and a small code layouter.
//!
//! The data prepared here is rendered by the `*.dart.jinja` templates (see [`super::compiler::render_dart`]).
//! Code that may exceed the line width is laid out by [`E`] / [`render`] the way `dart format`
//! (tall style, Dart >= 3.7, page width 80) formats it, so the generated file is already formatted.
//! The generated file additionally contains `// dart format off`, so that projects with another
//! page width or language version do not reformat it.

use std::collections::HashMap;

use stringcase::Caser;

use super::compiler::{
    canonical_simple_type, max_bits_message, min_bits_details, min_bits_message, AttributeDetails, Attribute, BitisProcessed, DynOrFixedType, Enum,
    FixedPrecisionProperties,
    Message, OneOfInfo, SimpleType,
};

/// Page width of `dart format`.
const WIDTH: usize = 80;

// ************************************************************************
// Layouter
// ************************************************************************
//
// The rules below reproduce the output of `dart format` (tall style) for the constructs used by
// the generated code: a construct stays on one line if it fits, otherwise argument / parameter /
// collection lists get one element per line (+2 indentation, trailing comma) and values that can
// not be split move to the next line (+4). The generated code avoids constructs for which the
// formatter chooses between alternatives by cost (e.g. long `=>` bodies are emitted as block bodies).

/// A Dart expression for the layouter.
#[derive(Clone, Debug)]
pub enum E {
    /// Text that is never split.
    Atom(String),
    /// `callee(args)`.
    Call(String, Vec<E>),
    /// `prefix` followed by an expression: lambda `(v) => `, `throw `.
    Pre(String, Box<E>),
    /// Named argument `name: `, map entry `'key': `, parameter default / initializer `x = `; the value
    /// moves to the next line (+4) if it can not be split after the prefix.
    Named(String, Box<E>),
    /// Switch expression case `pattern => `; the value moves to the next line (+2) if it can not be
    /// split after the prefix.
    Case(String, Box<E>),
    /// Binary operator `lhs op rhs`.
    Bin(String, Box<E>, Box<E>),
    /// Collection-if element: `if (...)` followed by an element.
    If(String, Box<E>),
    /// Declaration `Type name` (parameters, fields): split between type and name.
    Decl(String, String),
    /// List literal with a prefix (`const ` or empty).
    List(String, Vec<E>),
    /// Map literal.
    Map(Vec<E>),
}

pub fn atom(s: impl Into<String>) -> E { E::Atom(s.into()) }
pub fn call(callee: impl Into<String>, args: Vec<E>) -> E { E::Call(callee.into(), args) }
pub fn pre(prefix: impl Into<String>, e: E) -> E { E::Pre(prefix.into(), Box::new(e)) }
pub fn lambda(params: &str, body: E) -> E { pre(format!("{params} => "), body) }
pub fn named(name: impl Into<String>, e: E) -> E { E::Named(format!("{}: ", name.into()), Box::new(e)) }
pub fn assign(name: impl Into<String>, e: E) -> E { E::Named(format!("{} = ", name.into()), Box::new(e)) }
pub fn case(pattern: impl Into<String>, e: E) -> E { E::Case(format!("{} => ", pattern.into()), Box::new(e)) }
pub fn bin(op: &str, l: E, r: E) -> E { E::Bin(op.to_string(), Box::new(l), Box::new(r)) }
pub fn decl(ty: impl Into<String>, name: impl Into<String>) -> E { E::Decl(ty.into(), name.into()) }

fn sp(n: usize) -> String { " ".repeat(n) }

impl E {
    /// The expression on one line.
    pub fn flat(&self) -> String {
        let join = |v: &[E]| v.iter().map(|e| e.flat()).collect::<Vec<_>>().join(", ");
        match self {
            E::Atom(s) => s.clone(),
            E::Call(c, a) => format!("{c}({})", join(a)),
            E::Pre(p, e) | E::Named(p, e) | E::Case(p, e) => format!("{p}{}", e.flat()),
            E::Bin(op, l, r) => format!("{} {op} {}", l.flat(), r.flat()),
            E::If(c, e) => format!("{c} {}", e.flat()),
            E::Decl(t, n) => format!("{t} {n}"),
            E::List(p, a) => format!("{p}[{}]", join(a)),
            E::Map(a) => format!("{{{}}}", join(a)),
        }
    }
    fn is_collection(&self) -> bool { matches!(self, E::List(..) | E::Map(..)) }
    /// True if the expression can stay after a prefix ending at column `col` and be split there.
    fn hugs_at(&self, col: usize) -> bool {
        match self {
            E::Call(c, a) => !a.is_empty() && col + c.len() + 1 <= WIDTH,
            E::List(..) | E::Map(..) => true,
            _ => false,
        }
    }
}

/// Lays out `e` starting at column `col` with block indentation `indent`; `suffix` characters
/// follow the last line. Returns the text of the first line (to be appended at `col`) and the
/// complete continuation lines.
pub fn render(e: &E, indent: usize, col: usize, suffix: usize) -> Vec<String> {
    let flat = e.flat();
    if col + flat.len() + suffix <= WIDTH { return vec![flat]; }
    match e {
        // `Type.member` that does not fit: the member access moves to the next line (+4)
        E::Atom(a) if is_qualified_name(a) => {
            let dot = a.rfind('.').unwrap();
            vec![a[..dot].to_string(), format!("{}{}", sp(indent + 4), &a[dot..])]
        }
        E::Atom(_) => vec![flat],
        E::Call(_, args) if args.is_empty() => vec![flat],
        E::Pre(p, inner) | E::Named(p, inner) | E::Case(p, inner) => {
            let is_case = matches!(e, E::Case(..));
            let hug = inner.hugs_at(col + p.len()) && !(is_case && inner.is_collection());
            if hug {
                let mut l = render(inner, indent, col + p.len(), suffix);
                l[0] = format!("{p}{}", l[0]);
                return l;
            }
            let mut l = vec![p.trim_end().to_string()];
            let ind = indent + if is_case { 2 } else { 4 };
            match &**inner {
                // `x =` / `a ??` / `b`: the operands are aligned
                E::Bin(op, a, b) if op != "is" && ind + inner.flat().len() + suffix > WIDTH => {
                    l.push(format!("{}{} {op}", sp(ind), a.flat()));
                    push_lines(&mut l, ind, render(b, ind, ind, suffix));
                }
                _ => push_lines(&mut l, ind, render(inner, ind, ind, suffix)),
            }
            l
        }
        E::Bin(op, a, b) if op == "is" => vec![a.flat(), format!("{}is {}", sp(indent + 4), b.flat())],
        E::Bin(op, a, b) => vec![format!("{} {op}", a.flat()), format!("{}{}", sp(indent + 4), b.flat())],
        E::If(c, inner) => {
            let mut l = vec![c.clone()];
            let sub = render(inner, indent + 2, indent + 2, suffix);
            push_lines(&mut l, indent + 2, sub);
            l
        }
        E::Decl(t, n) => vec![t.clone(), format!("{}{n}", sp(indent))],
        E::Call(c, args) if args.len() == 1 && args[0].is_collection() => {
            let mut l = render(&args[0], indent, col + c.len() + 1, suffix + 1);
            l[0] = format!("{c}({}", l[0]);
            l.last_mut().unwrap().push(')');
            l
        }
        E::Call(c, args) => split_items(format!("{c}("), args, ")", indent),
        E::List(p, items) => split_items(format!("{p}["), items, "]", indent),
        E::Map(items) => split_items("{".to_string(), items, "}", indent),
    }
}

/// `Ident.ident` (e.g. an enum value or a static member).
fn is_qualified_name(s: &str) -> bool {
    let mut parts = s.split('.');
    let ok = |p: Option<&str>| p.is_some_and(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'));
    ok(parts.next()) && ok(parts.next()) && parts.next().is_none()
}

/// Appends `sub` (first line without indentation) at `indent`.
fn push_lines(out: &mut Vec<String>, indent: usize, sub: Vec<String>) {
    let mut it = sub.into_iter();
    if let Some(first) = it.next() { out.push(format!("{}{first}", sp(indent))); }
    out.extend(it);
}

/// One item per line with trailing commas.
fn split_items(open: String, items: &[E], close: &str, indent: usize) -> Vec<String> {
    let mut l = vec![open];
    for it in items {
        let sub = render(it, indent + 2, indent + 2, 1);
        push_lines(&mut l, indent + 2, sub);
        l.last_mut().unwrap().push(',');
    }
    l.push(format!("{}{close}", sp(indent)));
    l
}

/// A statement `prefix expr end` at `indent`, e.g. `return Foo(...);`.
pub fn stmt(indent: usize, prefix: &str, e: &E, end: &str) -> String {
    let mut l = render(e, indent, indent + prefix.len(), end.len());
    l[0] = format!("{}{prefix}{}", sp(indent), l[0]);
    l.last_mut().unwrap().push_str(end);
    l.join("\n")
}

fn flat_list(v: &[E]) -> String { v.iter().map(|e| e.flat()).collect::<Vec<_>>().join(", ") }

/// Signature of a member: optional return type, `name` + `open` + parameters + `close`, e.g.
/// `static Foo` `fromBytes(` `List<int> data` `)`.
#[derive(Clone, Debug)]
pub struct Sig {
    ret: Option<String>,
    name: String,
    open: &'static str,
    params: Vec<E>,
    close: &'static str,
}
impl Sig {
    /// A signature without parameters (getters, methods without parameters); `ret` is the part
    /// before the name (`static Foo`), the formatter may split after it.
    pub fn plain(ret: &str, name: &str) -> Sig {
        Sig { ret: Some(ret.to_string()), name: name.to_string(), open: "", params: vec![], close: "" }
    }
    /// `ret name(params)`; `name` ends with `(`.
    pub fn method(ret: &str, name: &str, params: Vec<E>) -> Sig {
        let ret = if ret.is_empty() { None } else { Some(ret.to_string()) };
        Sig { ret, name: name.to_string(), open: "", params, close: ")" }
    }
    fn head(&self) -> String {
        match &self.ret { Some(r) => format!("{r} {}", self.name), None => self.name.clone() }
    }
    fn flat(&self) -> String { format!("{}{}{}{}", self.head(), self.open, flat_list(&self.params), self.close) }
    /// Lines of the signature; the last line is returned separately (the caller appends ` => ...`
    /// or ` {`). `extra` is the length of what follows the signature on its last line if it is not split.
    fn lines(&self, indent: usize, extra: usize) -> (Vec<String>, String) {
        let flat = format!("{}{}", sp(indent), self.flat());
        if flat.len() + extra <= WIDTH { return (vec![], flat); }
        if let Some(r) = &self.ret {
            if indent + self.head().len() + self.open.len() > WIDTH {
                // the formatter splits after the return type first
                let rest = Sig { ret: None, ..self.clone() };
                let (mut l, last) = rest.lines(indent, extra);
                l.insert(0, format!("{}{r}", sp(indent)));
                return (l, last);
            }
        }
        if self.params.is_empty() { return (vec![], flat); }
        let mut l = vec![format!("{}{}{}", sp(indent), self.head(), self.open)];
        for p in &self.params {
            push_lines(&mut l, indent + 2, render(p, indent + 2, indent + 2, 1));
            l.last_mut().unwrap().push(',');
        }
        (l, format!("{}{}", sp(indent), self.close))
    }
}

/// Expression body member `sig => body;` at `indent`.
pub fn arrow(indent: usize, sig: &Sig, body: &E) -> String {
    let flat = body.flat();
    let (mut l, last) = sig.lines(indent, 3);
    let sig_split = !l.is_empty();
    let head = format!("{last} => ");
    let hug = |head: &str, mut before: Vec<String>| {
        let mut l = render(body, indent, head.len(), 1);
        l[0] = format!("{head}{}", l[0]);
        l.last_mut().unwrap().push(';');
        before.extend(l);
        before.join("\n")
    };
    if head.len() + flat.len() + 1 <= WIDTH {
        l.push(format!("{head}{flat};"));
        return l.join("\n");
    }
    if body.is_collection() || (sig_split && body.hugs_at(head.len())) { return hug(&head, l); }
    let cont = indent + 4;
    l.push(format!("{last} =>"));
    if cont + flat.len() + 1 <= WIDTH || !body.hugs_at(head.len()) {
        push_lines(&mut l, cont, render(body, cont, cont, 1));
        l.last_mut().unwrap().push(';');
        return l.join("\n");
    }
    l.pop();
    hug(&head, l)
}

/// `sig => body;` if the body fits on the line of the signature or on the next line, otherwise
/// a block body `{ return body; }` (whose layout does not depend on `dart format`'s cost model).
pub fn arrow_or_block(indent: usize, sig: &Sig, body: &E) -> String {
    let flat = body.flat();
    let (l, last) = sig.lines(indent, 3);
    if l.is_empty() && (last.len() + 4 + flat.len() + 1 <= WIDTH || indent + 4 + flat.len() + 1 <= WIDTH) {
        return arrow(indent, sig, body);
    }
    block_method(indent, sig, &stmt(indent + 2, "return ", body, ";"))
}

/// Method with a block body: signature and `body` (complete lines).
pub fn block_method(indent: usize, sig: &Sig, body: &str) -> String {
    let (mut l, last) = sig.lines(indent, 2);
    l.push(format!("{last} {{"));
    l.push(body.to_string());
    l.push(format!("{}}}", sp(indent)));
    l.join("\n")
}

/// Class header `decl {`, e.g. `final class Foo implements BitisMessage {`; `clause` is
/// `extends` / `implements` and `ty` the super type (empty: none).
pub fn class_header(decl: &str, clause: &str, ty: &str) -> String {
    if clause.is_empty() { return format!("{decl} {{"); }
    let flat = format!("{decl} {clause} {ty} {{");
    if flat.len() <= WIDTH { return flat; }
    if 4 + clause.len() + ty.len() + 3 <= WIDTH { return format!("{decl}\n    {clause} {ty} {{"); }
    format!("{decl}\n    {clause}\n        {ty} {{")
}

/// Expression body member `sig => a && b && c;` at `indent`.
pub fn arrow_and(indent: usize, sig: &str, ops: &[E]) -> String {
    let flat = ops.iter().map(|o| o.flat()).collect::<Vec<_>>().join(" && ");
    let head = format!("{}{sig} => ", sp(indent));
    if head.len() + flat.len() + 1 <= WIDTH { return format!("{head}{flat};"); }
    let cont = indent + 4;
    if cont + flat.len() + 1 <= WIDTH { return format!("{}{sig} =>\n{}{flat};", sp(indent), sp(cont)); }
    let mut l = vec![format!("{}{sig} =>", sp(indent))];
    for (i, o) in ops.iter().enumerate() {
        let end = if i + 1 == ops.len() { ";" } else { " &&" };
        push_lines(&mut l, cont, render(o, cont, cont, end.len()));
        l.last_mut().unwrap().push_str(end);
    }
    l.join("\n")
}

/// `sig => 'text';` for a string literal given as parts; if it does not fit, the parts become
/// adjacent string literals on separate lines.
pub fn string_arrow(indent: usize, sig: &str, parts: &[String]) -> String {
    let joined = format!("'{}'", parts.concat());
    let one = format!("{}{sig} => {joined};", sp(indent));
    if one.len() <= WIDTH { return one; }
    let cont = indent + 4;
    if cont + joined.len() + 1 <= WIDTH || parts.len() < 2 {
        return format!("{}{sig} =>\n{}{joined};", sp(indent), sp(cont));
    }
    let mut l = vec![format!("{}{sig} =>", sp(indent))];
    for p in parts { l.push(format!("{}'{p}'", sp(cont))); }
    l.last_mut().unwrap().push(';');
    l.join("\n")
}

/// Parameter list `head` + `open` + params + `close` + `tail` (e.g. `const Foo(` `{` ... `})` `;`).
pub fn params_sig(indent: usize, head: &str, open: &'static str, params: Vec<E>, close: &'static str, tail: &str) -> String {
    let sig = Sig { ret: None, name: head.to_string(), open, params, close };
    let (mut l, last) = sig.lines(indent, tail.len());
    l.push(format!("{last}{tail}"));
    l.join("\n")
}

/// Constructor with an initializer list (`inits` may be empty).
pub fn ctor(indent: usize, head: &str, open: &'static str, params: Vec<E>, close: &'static str, inits: &[E]) -> String {
    if inits.is_empty() { return params_sig(indent, head, open, params, close, ";"); }
    let sig = format!("{}{head}{open}{}{close}", sp(indent), flat_list(&params));
    let flat = format!("{sig} : {};", flat_list(inits));
    if flat.len() <= WIDTH { return flat; }
    let (mut l, init_col, rest_indent) = if sig.len() <= WIDTH {
        (vec![sig, format!("{}: ", sp(indent + 2))], indent + 4, indent + 4)
    } else {
        let s = Sig { ret: None, name: head.to_string(), open, params, close };
        let (mut l, last) = s.lines(indent, WIDTH);
        l.push(format!("{last} : "));
        (l, indent + close.len() + 3, indent + close.len() + 3)
    };
    for (i, init) in inits.iter().enumerate() {
        let col = if i == 0 { init_col } else { rest_indent };
        let mut sub = render(init, rest_indent, col, 1).into_iter();
        if i == 0 {
            let first = sub.next().unwrap();
            l.last_mut().unwrap().push_str(&first);
            l.extend(sub);
        } else {
            push_lines(&mut l, rest_indent, sub.collect());
        }
        l.last_mut().unwrap().push(if i + 1 == inits.len() { ';' } else { ',' });
    }
    l.join("\n")
}

/// Declaration `ty name = value;` (e.g. `static const FixedPointType xFpType = FixedPointType(...)`).
pub fn decl_init(indent: usize, ty: &str, name: &str, value: &E) -> String {
    let decl = if ty.is_empty() { name.to_string() } else { format!("{ty} {name}") };
    let flat = format!("{}{decl} = {};", sp(indent), value.flat());
    if flat.len() <= WIDTH { return flat; }
    if !ty.is_empty() && indent + decl.len() + 2 > WIDTH {
        return format!("{}{ty}\n{}", sp(indent), decl_init(indent, "", name, value));
    }
    let head = format!("{}{decl} = ", sp(indent));
    if value.hugs_at(head.len()) {
        let mut l = render(value, indent, head.len(), 1);
        l[0] = format!("{head}{}", l[0]);
        l.last_mut().unwrap().push(';');
        return l.join("\n");
    }
    let mut l = vec![format!("{}{decl} =", sp(indent))];
    push_lines(&mut l, indent + 4, render(value, indent + 4, indent + 4, 1));
    l.last_mut().unwrap().push(';');
    l.join("\n")
}

/// Field declaration `final T name;`.
pub fn field_decl(indent: usize, ty: &str, name: &str) -> String {
    let flat = format!("{}final {ty} {name};", sp(indent));
    if flat.len() <= WIDTH { flat } else { format!("{}final {ty}\n{}{name};", sp(indent), sp(indent)) }
}

// ************************************************************************
// Type mapping
// ************************************************************************

/// Dart representation of integers.
#[derive(Clone, Copy, Debug, PartialEq)]
enum IntRepr { Int, Int64, Big }

fn uint_repr(bits: u8) -> IntRepr {
    match bits { 0..=32 => IntRepr::Int, 33..=63 => IntRepr::Int64, _ => IntRepr::Big }
}
fn int_repr(bits: u8) -> IntRepr { if bits <= 32 { IntRepr::Int } else { IntRepr::Int64 } }

/// An element type (without modifier).
#[derive(Clone, Debug)]
enum Ty {
    Bool,
    UInt(u8),
    Int(u8),
    DynUInt(u8, u8),
    DynInt(u8, u8),
    Float,
    Double,
    /// fixed point type and the name of the `FixedPointType` constant
    Fp(FixedPrecisionProperties, String),
    /// `astr_dY[M]`: dyn bits and optional maximum length.
    AStr(u8, Option<u16>),
    /// `binary_dY[M]`: dyn bits and optional maximum length.
    Binary(u8, Option<u16>),
    Enum(String),
    Msg(String),
    OneOf(String),
}

/// Information about the schema needed for the type mapping.
pub struct Ctx<'a> {
    msgs: HashMap<&'a str, &'a Message>,
    enums: HashMap<&'a str, &'a Enum>,
    all_msgs: &'a [Message],
    all_enums: &'a [Enum],
    /// Oneof type name -> oneof.
    oneofs: HashMap<String, &'a OneOfInfo>,
    const_cache: std::cell::RefCell<HashMap<String, bool>>,
}

impl<'a> Ctx<'a> {
    pub fn new(p: &'a BitisProcessed) -> Self {
        Ctx {
            msgs: p.msgs.iter().map(|m| (m.name.as_str(), m)).collect(),
            enums: p.enums.iter().map(|e| (e.name.as_str(), e)).collect(),
            all_msgs: &p.msgs,
            all_enums: &p.enums,
            oneofs: p.oo_enums.iter().map(|(_, oo)| (oo.name.clone(), oo)).collect(),
            const_cache: Default::default(),
        }
    }

    /// True if the default value of a message can be created by a const constructor.
    fn msg_is_const(&self, name: &str) -> bool {
        if let Some(r) = self.const_cache.borrow().get(name) { return *r; }
        let r = self.msgs.get(name).map(|m| m.attributes.iter().all(|a| self.attr_is_const(a))).unwrap_or(true);
        self.const_cache.borrow_mut().insert(name.to_string(), r);
        r
    }
    fn attr_is_const(&self, a: &Attribute) -> bool {
        if a.is_optional || matches!(a.is_repeated_and_size, Some(DynOrFixedType::Dyn(_))) { return true; }
        self.ty_is_const(&self.ty(a, ""))
    }
    fn oneof_is_const(&self, oo: &OneOfInfo) -> bool {
        oo.attributes.iter().find(|a| a.name == oo.default_attrib_name).map(|a| self.attr_is_const(a)).unwrap_or(true)
    }
    fn ty_is_const(&self, t: &Ty) -> bool {
        match t {
            Ty::UInt(b) | Ty::DynUInt(b, _) if uint_repr(*b) == IntRepr::Big => false,
            Ty::Binary(..) => false,
            Ty::Msg(m) => self.msg_is_const(m),
            Ty::OneOf(o) => self.oneofs.get(o).map(|oo| self.oneof_is_const(oo)).unwrap_or(true),
            _ => true,
        }
    }

    /// Element type of an attribute; `fp_const` is the name of its `FixedPointType` constant.
    fn ty(&self, a: &Attribute, fp_const: &str) -> Ty {
        match &a.specific_details {
            AttributeDetails::AttributeSimple(s) => match s {
                SimpleType::Bool => Ty::Bool,
                SimpleType::UIntFixed(b) => Ty::UInt(*b),
                SimpleType::IntFixed(b) => Ty::Int(*b),
                SimpleType::UIntDyn((x, y)) => Ty::DynUInt(*x, *y),
                SimpleType::IntDyn((x, y)) => Ty::DynInt(*x, *y),
                SimpleType::Float => Ty::Float,
                SimpleType::Double => Ty::Double,
                SimpleType::FixedPrecision(f) => Ty::Fp(*f, fp_const.to_string()),
                SimpleType::AString(y, m) => Ty::AStr(*y, *m),
                SimpleType::Binary(y, m) => Ty::Binary(*y, *m),
            },
            AttributeDetails::AttributeEnumOrMsg(n) => {
                if self.enums.contains_key(n.as_str()) { Ty::Enum(n.clone()) } else { Ty::Msg(n.clone()) }
            }
            AttributeDetails::AttributeOneOf(oo) => Ty::OneOf(oo.name.clone()),
        }
    }
}

/// Appends the named argument `maxLength: M` of a bounded type (spec 2.16).
fn with_max_len(mut args: Vec<E>, max_len: Option<u16>) -> Vec<E> {
    if let Some(m) = max_len { args.push(named("maxLength", atom(m.to_string()))); }
    args
}

/// Largest integer that is exact on every Dart platform (2^53 - 1, JavaScript).
const DART_MAX_SAFE_INT: u64 = (1 << 53) - 1;

fn repr_type(r: IntRepr) -> &'static str {
    match r { IntRepr::Int => "int", IntRepr::Int64 => "Int64", IntRepr::Big => "BigInt" }
}
fn repr_suffix(r: IntRepr) -> &'static str {
    match r { IntRepr::Int => "", IntRepr::Int64 => "Int64", IntRepr::Big => "BigInt" }
}

impl Ty {
    fn dart(&self) -> String {
        match self {
            Ty::Bool => "bool".into(),
            Ty::UInt(b) | Ty::DynUInt(b, _) => repr_type(uint_repr(*b)).into(),
            Ty::Int(b) | Ty::DynInt(b, _) => repr_type(int_repr(*b)).into(),
            Ty::Float | Ty::Double => "double".into(),
            Ty::Fp(..) => "FixedPoint".into(),
            Ty::AStr(..) => "String".into(),
            Ty::Binary(..) => "Uint8List".into(),
            Ty::Enum(n) | Ty::Msg(n) | Ty::OneOf(n) => n.clone(),
        }
    }

    /// `w.write...(v, ...)`
    fn write(&self, v: &str) -> E {
        let a = |s: String| atom(s);
        match self {
            Ty::Bool => call("w.writeBool", vec![a(v.into())]),
            Ty::UInt(b) => call(format!("w.writeUint{}", repr_suffix(uint_repr(*b))), vec![a(v.into()), a(b.to_string())]),
            Ty::Int(b) => call(format!("w.writeInt{}", repr_suffix(int_repr(*b))), vec![a(v.into()), a(b.to_string())]),
            Ty::DynUInt(x, y) => call(format!("w.writeDynUint{}", repr_suffix(uint_repr(*x))),
                                      vec![a(v.into()), a(x.to_string()), a(y.to_string())]),
            Ty::DynInt(x, y) => call(format!("w.writeDynInt{}", repr_suffix(int_repr(*x))),
                                     vec![a(v.into()), a(x.to_string()), a(y.to_string())]),
            Ty::Float => call("w.writeFloat32", vec![a(v.into())]),
            Ty::Double => call("w.writeFloat64", vec![a(v.into())]),
            Ty::Fp(_, c) => call("w.writeFixedPoint", vec![a(v.into()), a(c.clone())]),
            Ty::AStr(y, m) => call("w.writeAStr", with_max_len(vec![a(v.into()), a(y.to_string())], *m)),
            Ty::Binary(y, m) => call("w.writeBinary", with_max_len(vec![a(v.into()), a(y.to_string())], *m)),
            Ty::Enum(_) | Ty::Msg(_) | Ty::OneOf(_) => call(format!("{v}.bitisWrite"), vec![a("w".into())]),
        }
    }
    fn write_fn(&self) -> E { lambda("(w, v)", self.write("v")) }

    /// `r.read...(...)`
    fn read(&self) -> E {
        let n = |x: &u8| atom(x.to_string());
        match self {
            Ty::Bool => call("r.readBool", vec![]),
            Ty::UInt(b) => call(format!("r.readUint{}", repr_suffix(uint_repr(*b))), vec![n(b)]),
            Ty::Int(b) => call(format!("r.readInt{}", repr_suffix(int_repr(*b))), vec![n(b)]),
            Ty::DynUInt(x, y) => call(format!("r.readDynUint{}", repr_suffix(uint_repr(*x))), vec![n(x), n(y)]),
            Ty::DynInt(x, y) => call(format!("r.readDynInt{}", repr_suffix(int_repr(*x))), vec![n(x), n(y)]),
            Ty::Float => call("r.readFloat32", vec![]),
            Ty::Double => call("r.readFloat64", vec![]),
            Ty::Fp(_, c) => call("r.readFixedPoint", vec![atom(c.clone())]),
            Ty::AStr(y, m) => call("r.readAStr", with_max_len(vec![n(y)], *m)),
            Ty::Binary(y, m) => call("r.readBinary", with_max_len(vec![n(y)], *m)),
            Ty::Enum(t) | Ty::Msg(t) | Ty::OneOf(t) => call(format!("{t}.bitisRead"), vec![atom("r")]),
        }
    }
    fn read_fn(&self) -> E {
        match self {
            Ty::Enum(t) | Ty::Msg(t) | Ty::OneOf(t) => atom(format!("{t}.bitisRead")),
            _ => lambda("(r)", self.read()),
        }
    }

    /// JSON value of `v`; `None` if `v` itself is the JSON value.
    fn to_json(&self, v: &str) -> Option<E> {
        let h = |f: &str| Some(call(format!("BitisJson.{f}"), vec![atom(v)]));
        match self {
            Ty::Bool | Ty::AStr(..) => None,
            Ty::UInt(b) | Ty::DynUInt(b, _) => match uint_repr(*b) {
                IntRepr::Int => None, IntRepr::Int64 => h("int64ToJson"), IntRepr::Big => h("bigIntToJson"),
            },
            Ty::Int(b) | Ty::DynInt(b, _) => match int_repr(*b) {
                IntRepr::Int => None, _ => h("int64ToJson"),
            },
            Ty::Float | Ty::Double => h("doubleToJson"),
            Ty::Fp(..) => h("fixedPointToJson"),
            Ty::Binary(..) => h("binaryToJson"),
            Ty::Enum(_) | Ty::Msg(_) | Ty::OneOf(_) => Some(call(format!("{v}.toJson"), vec![])),
        }
    }
    fn to_json_expr(&self, v: &str) -> E { self.to_json(v).unwrap_or_else(|| atom(v)) }
    fn to_json_fn(&self) -> Option<E> {
        match self {
            Ty::Enum(_) | Ty::Msg(_) | Ty::OneOf(_) => Some(lambda("(v)", self.to_json("v")?)),
            _ => match self.to_json("v")? { E::Call(c, _) => Some(atom(c)), e => Some(e) },
        }
    }

    /// Function name converting a JSON value (`Object?`) into the Dart value (tear-off or lambda).
    fn from_json_fn(&self) -> E {
        match self {
            Ty::Msg(_) => lambda("(v)", self.from_json(atom("v"))),
            Ty::Enum(t) | Ty::OneOf(t) => atom(format!("{t}.fromJson")),
            _ => atom(self.from_json_name()),
        }
    }
    fn from_json_name(&self) -> String {
        let f = match self {
            Ty::Bool => "boolFromJson",
            Ty::UInt(b) | Ty::DynUInt(b, _) => match uint_repr(*b) {
                IntRepr::Int => "intFromJson", IntRepr::Int64 => "int64FromJson", IntRepr::Big => "bigIntFromJson",
            },
            Ty::Int(b) | Ty::DynInt(b, _) => if int_repr(*b) == IntRepr::Int { "intFromJson" } else { "int64FromJson" },
            Ty::Float | Ty::Double => "doubleFromJson",
            Ty::Fp(..) => "fixedPointFromJson",
            Ty::AStr(..) => "astrFromJson",
            Ty::Binary(..) => "binaryFromJson",
            Ty::Enum(_) | Ty::Msg(_) | Ty::OneOf(_) => unreachable!(),
        };
        format!("BitisJson.{f}")
    }
    fn from_json(&self, j: E) -> E {
        match self {
            Ty::Msg(t) => call(format!("{t}.fromJson"), vec![call("BitisJson.objectFromJson", vec![j])]),
            Ty::Enum(t) | Ty::OneOf(t) => call(format!("{t}.fromJson"), vec![j]),
            _ => call(self.from_json_name(), vec![j]),
        }
    }

    /// Equality of two values; `None` means `a == b`.
    fn eq_fn(&self) -> Option<E> {
        match self {
            Ty::Fp(_, c) => Some(atom(format!("{c}.sameEncoding"))),
            Ty::Binary(..) => Some(atom("BitisEq.listEquals<int>")),
            _ => None,
        }
    }
    fn eq(&self, a: &str, b: &str) -> E {
        match self.eq_fn() {
            Some(E::Atom(f)) => call(f, vec![atom(a), atom(b)]),
            _ => bin("==", atom(a), atom(b)),
        }
    }
    /// Value used for the hash code; `None` means the value itself.
    fn hash_fn(&self) -> Option<E> {
        match self {
            Ty::Fp(_, c) => Some(atom(format!("{c}.encodeRaw"))),
            Ty::Binary(..) => Some(atom("Object.hashAll")),
            _ => None,
        }
    }
    fn hash(&self, v: &str) -> E {
        match self.hash_fn() {
            Some(E::Atom(f)) => call(f, vec![atom(v)]),
            _ => atom(v),
        }
    }

    /// Default value (Rust `Default` semantics). `const_ctx`: expression is in a const context.
    fn default(&self, ctx: &Ctx, const_ctx: bool) -> String {
        let k = if const_ctx { "" } else { "const " };
        match self {
            Ty::Bool => "false".into(),
            Ty::UInt(b) | Ty::DynUInt(b, _) => match uint_repr(*b) {
                IntRepr::Int => "0".into(), IntRepr::Int64 => "Int64.ZERO".into(), IntRepr::Big => "BigInt.zero".into(),
            },
            Ty::Int(b) | Ty::DynInt(b, _) => if int_repr(*b) == IntRepr::Int { "0".into() } else { "Int64.ZERO".into() },
            Ty::Float | Ty::Double => "0.0".into(),
            // Rust and C++ default: Overflow
            Ty::Fp(..) => format!("{k}FixedPoint.overflow()"),
            Ty::AStr(..) => "''".into(),
            Ty::Binary(..) => "Uint8List(0)".into(),
            Ty::Enum(n) => {
                let e = ctx.enums[n.as_str()];
                format!("{n}.{}", e.default.to_camel_case())
            }
            Ty::Msg(n) => if ctx.msg_is_const(n) { format!("{k}{n}()") } else { format!("{n}()") },
            Ty::OneOf(n) => format!("{n}.defaultValue"),
        }
    }
}


// ************************************************************************
// Template data
// ************************************************************************

/// A field of a message or the value of a oneof element.
#[derive(Clone, Debug)]
pub struct DartField {
    /// Dart identifier (lowerCamelCase of the attribute name, `value` for oneof elements).
    pub dart_name: String,
    /// Doc comment of the attribute.
    pub comment: Option<String>,
    /// Attribute as written in the schema, e.g. `optional uint_12 lolo`.
    pub schema: String,
    /// Field declaration.
    pub decl: String,
    /// `static const FixedPointType ...` declaration of fixed point fields.
    pub fp_const: Option<String>,
}

#[derive(Clone, Debug)]
pub struct DartEnumValue {
    pub dart_name: String,
    pub bitis_name: String,
    pub comment: Option<String>,
}
#[derive(Clone, Debug)]
pub struct DartEnum {
    pub name: String,
    pub comment: Option<String>,
    pub values: Vec<DartEnumValue>,
    pub dyn_bits: u8,
    pub default_decl: String,
    pub write: String,
    pub read: String,
    pub from_json: String,
    pub by_name: String,
}

#[derive(Clone, Debug)]
pub struct DartVariant {
    pub class_name: String,
    pub class_header: String,
    pub bitis_name: String,
    pub comment: Option<String>,
    pub index: usize,
    pub field: DartField,
    pub ctor: String,
    pub write_index: String,
    pub write_value: String,
    pub to_json: String,
    pub eq: String,
    pub hash: String,
    pub to_string: String,
}
#[derive(Clone, Debug)]
pub struct DartOneOf {
    pub name: String,
    pub class_header: String,
    pub comment: Option<String>,
    /// Attribute name of the oneof in the schema.
    pub bitis_name: String,
    pub msg_name: String,
    pub default_decl: String,
    pub read: String,
    pub from_json: String,
    pub variants: Vec<DartVariant>,
}

#[derive(Clone, Debug)]
pub struct DartMsg {
    pub name: String,
    /// `min_bits` of the message (spec section 4).
    pub min_bits: u64,
    /// `max_bits` of the message (spec section 4) as Dart literal: the number or `null` if unbounded
    /// (or larger than 2^53 - 1, which is not exact on the web).
    pub max_bits: String,
    pub class_header: String,
    pub comment: Option<String>,
    pub parent: Option<String>,
    pub fields: Vec<DartField>,
    pub ctor: String,
    /// Statements of `bitisWrite` (empty for messages without attributes).
    pub write_body: String,
    pub read: String,
    pub to_bytes: String,
    pub from_bytes: String,
    pub from_bytes_exact: String,
    pub to_json: String,
    pub from_json: String,
    /// `copyWith` (empty for messages without attributes).
    pub copy_with: String,
    pub eq: String,
    pub hash: String,
    pub to_string: String,
    pub oneofs: Vec<DartOneOf>,
}

#[derive(Clone, Debug)]
pub struct DartData {
    pub enums: Vec<DartEnum>,
    pub msgs: Vec<DartMsg>,
    pub needs_typed_data: bool,
    pub schema_hash_hi: String,
    pub schema_hash_lo: String,
}

/// A field referenced inside a generated method that has a parameter / local with the same name
/// (`w` in `bitisWrite`, `other` in `==`) needs `this.`.
fn field_ref(name: &str, shadowed: &[&str]) -> String {
    if shadowed.contains(&name) { format!("this.{name}") } else { name.to_string() }
}

/// Schema text of an attribute, e.g. `optional uint_12 lolo`.
fn schema_text(a: &Attribute) -> String {
    let mut s = String::new();
    if a.is_optional { s.push_str("optional "); }
    match a.is_repeated_and_size {
        Some(DynOrFixedType::Fixed(n)) => s.push_str(&format!("repeated_fixed_{n} ")),
        Some(DynOrFixedType::Dyn(n)) => match a.repeated_max_len {
            None => s.push_str(&format!("repeated_dyn_{n} ")),
            Some(m) => s.push_str(&format!("repeated_dyn_{n}[{m}] ")),
        },
        None => (),
    }
    match &a.specific_details {
        AttributeDetails::AttributeSimple(t) => s.push_str(&canonical_simple_type(t)),
        AttributeDetails::AttributeEnumOrMsg(t) => s.push_str(t),
        AttributeDetails::AttributeOneOf(oo) => s.push_str(&format!("oneof({})", oo.dyn_bits)),
    }
    s.push(' ');
    s.push_str(&a.name);
    s
}

/// Per-field code of a message.
struct FieldCode {
    field: DartField,
    /// constructor parameter (`this.x = 0` or `Uint8List? x`)
    param: E,
    /// initializer for defaults that are not constant (`x = x ?? Uint8List(0)`)
    init: Option<E>,
    is_const: bool,
    write: E,
    read: E,
    json_entry: E,
    from_json: E,
    eq: E,
    hash: E,
    copy_param: E,
    copy_arg: E,
}

fn fp_type_expr(f: &FixedPrecisionProperties) -> E {
    call("FixedPointType", vec![atom(f.bits.to_string()), atom(f.min_val.to_string()), atom(f.max_val.to_string())])
}
fn fp_const_decl(ty: &Ty, name: &str) -> Option<String> {
    match ty {
        Ty::Fp(f, _) => Some(decl_init(2, "static const FixedPointType", name, &fp_type_expr(f))),
        _ => None,
    }
}

fn field_code(ctx: &Ctx, a: &Attribute) -> FieldCode {
    let name = a.name.to_camel_case();
    let ty = ctx.ty(a, &format!("{name}FpType"));
    let elem = ty.dart();
    let modifier = if a.is_optional { None } else { a.is_repeated_and_size.clone() };
    let dart_ty = if a.is_optional { format!("{elem}?") } else if modifier.is_some() { format!("List<{elem}>") } else { elem.clone() };
    let field = DartField {
        dart_name: name.clone(), comment: a.comment.clone(), schema: schema_text(a),
        decl: field_decl(2, &dart_ty, &name), fp_const: fp_const_decl(&ty, &format!("{name}FpType")),
    };

    // default value / constructor parameter
    let is_const = ctx.attr_is_const(a);
    let default = if a.is_optional { None } else {
        Some(match &modifier {
            Some(DynOrFixedType::Dyn(_)) => atom("const []"),
            Some(DynOrFixedType::Fixed(n)) if is_const =>
                E::List("const ".into(), vec![atom(ty.default(ctx, true)); *n as usize]),
            Some(DynOrFixedType::Fixed(n)) =>
                call("List.generate", vec![atom(n.to_string()), lambda("(_)", atom(ty.default(ctx, false)))]),
            None => atom(ty.default(ctx, false)),
        })
    };
    let (param, init) = match default {
        None => (atom(format!("this.{name}")), None),
        Some(d) if is_const => (assign(format!("this.{name}"), d), None),
        Some(d) => (decl(format!("{dart_ty}?"), &name), Some(assign(&name, bin("??", atom(&name), d)))),
    };

    // encoding
    let wname = field_ref(&name, &["w"]);
    let write = match &modifier {
        _ if a.is_optional => call("w.writeOptional", vec![atom(&wname), ty.write_fn()]),
        Some(DynOrFixedType::Fixed(n)) => call("w.writeFixedArray", vec![atom(&wname), atom(n.to_string()), ty.write_fn()]),
        Some(DynOrFixedType::Dyn(y)) => call("w.writeDynArray",
            with_max_len(vec![atom(&wname), atom(y.to_string()), ty.write_fn()], a.repeated_max_len)),
        None => ty.write(&wname),
    };
    let read = match &modifier {
        _ if a.is_optional => call("r.readOptional", vec![ty.read_fn()]),
        Some(DynOrFixedType::Fixed(n)) => call("r.readFixedArray", vec![atom(n.to_string()), ty.read_fn()]),
        Some(DynOrFixedType::Dyn(y)) => {
            let mb = min_bits_details(&a.specific_details, ctx.all_msgs);
            call("r.readDynArray", with_max_len(vec![atom(y.to_string()), atom(mb.to_string()), ty.read_fn()],
                                                a.repeated_max_len))
        }
        None => ty.read(),
    };
    let read = named(&name, read);

    // JSON
    let key = format!("'{}'", a.name);
    let json_entry = if a.is_optional {
        E::If(format!("if ({name} case final v?)"), Box::new(named(&key, ty.to_json_expr("v"))))
    } else if modifier.is_some() {
        match ty.to_json_fn() {
            None => named(&key, atom(&name)),
            Some(f) => named(&key, call("BitisJson.listToJson", vec![atom(&name), f])),
        }
    } else {
        named(&key, ty.to_json_expr(&name))
    };
    let j = atom(format!("json['{}']", a.name));
    let from_json = match &modifier {
        _ if a.is_optional => call("BitisJson.optionalFromJson", vec![j, ty.from_json_fn()]),
        Some(DynOrFixedType::Fixed(n)) => call("BitisJson.listFromJsonWith", vec![j, ty.from_json_fn(), atom(n.to_string())]),
        Some(DynOrFixedType::Dyn(_)) => call("BitisJson.listFromJsonWith", vec![j, ty.from_json_fn()]),
        None => ty.from_json(j),
    };
    let from_json = named(&name, from_json);

    // equality / hash
    let a_ref = field_ref(&name, &["other"]);
    let b_ref = format!("other.{name}");
    let eq = match &modifier {
        _ if a.is_optional => match ty.eq_fn() {
            None => bin("==", atom(&a_ref), atom(&b_ref)),
            Some(f) => call("BitisEq.optionalEquals", vec![atom(&a_ref), atom(&b_ref), f]),
        },
        Some(_) => {
            let mut args = vec![atom(&a_ref), atom(&b_ref)];
            args.extend(ty.eq_fn());
            call("BitisEq.listEquals", args)
        }
        None => ty.eq(&a_ref, &b_ref),
    };
    let hash = match &modifier {
        _ if a.is_optional => match ty.hash_fn() {
            None => atom(&name),
            Some(f) => call("BitisEq.optionalHash", vec![atom(&name), f]),
        },
        Some(_) => {
            let mut args = vec![atom(&name)];
            args.extend(ty.hash_fn());
            call("BitisEq.listHash", args)
        }
        None => ty.hash(&name),
    };

    let (copy_param, copy_arg) = if a.is_optional {
        (assign(format!("Object? {name}"), atom("bitisKeep")),
         named(&name, call("bitisKeepOr", vec![atom(&name), atom(format!("this.{name}"))])))
    } else {
        (decl(format!("{dart_ty}?"), &name), named(&name, bin("??", atom(&name), atom(format!("this.{name}")))))
    };
    FieldCode { field, param, init, is_const, write, read, json_entry, from_json, eq, hash, copy_param, copy_arg }
}

/// Doc comment lines (`/// ...`) at `indent`.
fn doc(s: &str, indent: usize) -> String {
    s.split('\n').map(|l| {
        let l = l.trim_end();
        if l.is_empty() { format!("{}///", sp(indent)) } else { format!("{}/// {l}", sp(indent)) }
    }).collect::<Vec<_>>().join("\n")
}

/// `return switch (scrutinee) { cases };` at `indent`.
fn return_switch(indent: usize, scrutinee: &str, cases: &[E]) -> String {
    let mut l = vec![format!("{}return switch ({scrutinee}) {{", sp(indent))];
    for c in cases { l.push(stmt(indent + 2, "", c, ",")); }
    l.push(format!("{}}};", sp(indent)));
    l.join("\n")
}

fn build_oneof(ctx: &Ctx, msg_name: &str, attr_name: &str, comment: Option<String>, oo: &OneOfInfo) -> DartOneOf {
    let n = oo.attributes.len();
    let mut variants = vec![];
    let mut read_cases = vec![];
    let mut json_cases = vec![];
    for (i, va) in oo.attributes.iter().enumerate() {
        let class_name = format!("{}_{}", oo.name, va.name.to_pascal_case());
        let ty = ctx.ty(va, "valueFpType");
        // the base class refers to the fixed point type of the subclass
        let ty_base = ctx.ty(va, &format!("{class_name}.valueFpType"));
        let field = DartField {
            dart_name: "value".into(), comment: None, schema: schema_text(va),
            decl: field_decl(2, &ty.dart(), "value"), fp_const: fp_const_decl(&ty, "valueFpType"),
        };
        let ctor = if ctx.attr_is_const(va) {
            ctor(2, &format!("const {class_name}("), "[", vec![assign("this.value", atom(ty.default(ctx, false)))], "])", &[])
        } else {
            ctor(2, &format!("{class_name}("), "[", vec![decl(format!("{}?", ty.dart()), "value")], "])",
                 &[assign("value", bin("??", atom("value"), atom(ty.default(ctx, false))))])
        };
        let index_args = vec![atom(i.to_string()), atom(n.to_string()), atom(oo.dyn_bits.to_string())];
        let to_json = arrow(2, &Sig::plain("Map<String, Object?>", "toJson()"),
                            &E::Map(vec![named(format!("'{}'", va.name), ty.to_json_expr("value"))]));
        variants.push(DartVariant {
            class_header: class_header(&format!("final class {class_name}"), "extends", &oo.name),
            bitis_name: va.name.clone(), comment: va.comment.clone(), index: i, field, ctor,
            write_index: stmt(4, "", &call("w.writeOneOfIndex", index_args), ";"),
            write_value: stmt(4, "", &ty.write("value"), ";"),
            to_json,
            eq: arrow_and(2, "bool operator ==(Object other)",
                          &[bin("is", atom("other"), atom(&class_name)), ty.eq("value", "other.value")]),
            hash: arrow(2, &Sig::plain("int", "get hashCode"), &call("Object.hash", vec![atom(&class_name), ty.hash("value")])),
            to_string: string_arrow(2, "String toString()", &[format!("{class_name}($value)")]),
            class_name: class_name.clone(),
        });
        read_cases.push(case(i.to_string(), call(&class_name, vec![ty_base.read()])));
        json_cases.push(case(format!("'{}'", va.name), call(&class_name, vec![ty.from_json(atom("value"))])));
    }
    let default_class = format!("{}_{}", oo.name, oo.default_attrib_name.to_pascal_case());
    let default_decl = if ctx.oneof_is_const(oo) {
        decl_init(2, &format!("static const {}", oo.name), "defaultValue", &call(&default_class, vec![]))
    } else {
        arrow(2, &Sig::plain(&format!("static {}", oo.name), "get defaultValue"), &call(&default_class, vec![]))
    };
    read_cases.push(case("_", pre("throw ", call("StateError", vec![atom("'unreachable'")]))));
    json_cases.push(case("_", pre("throw ", call("FormatException",
        vec![atom(format!("'unknown element of {}'", oo.name)), atom("variant")]))));
    let read = block_method(2, &Sig::method(&format!("static {}", oo.name), "bitisRead(", vec![atom("BitReader r")]),
                            &return_switch(4, &format!("r.readOneOfIndex({n}, {})", oo.dyn_bits), &read_cases));
    let from_json = block_method(2, &Sig::method(&format!("static {}", oo.name), "fromJson(", vec![atom("Object? json")]),
        &format!("    final (variant, value) = BitisJson.oneOfFromJson(json);\n{}", return_switch(4, "variant", &json_cases)));
    DartOneOf {
        name: oo.name.clone(), class_header: class_header(&format!("sealed class {}", oo.name), "", ""),
        comment, bitis_name: attr_name.to_string(), msg_name: msg_name.to_string(),
        default_decl, read, from_json, variants,
    }
}

fn build_msg(ctx: &Ctx, m: &Message, oneofs: &[(String, OneOfInfo)]) -> DartMsg {
    let name = &m.name;
    let codes: Vec<FieldCode> = m.attributes.iter().map(|a| field_code(ctx, a)).collect();
    let all_const = codes.iter().all(|c| c.is_const);
    let params: Vec<E> = codes.iter().map(|c| c.param.clone()).collect();
    let inits: Vec<E> = codes.iter().filter_map(|c| c.init.clone()).collect();
    let ctor_head = if all_const { format!("const {name}(") } else { format!("{name}(") };
    let ctor = if params.is_empty() { format!("  {ctor_head});") } else { ctor(2, &ctor_head, "{", params, "})", &inits) };

    let write_body = codes.iter().map(|c| stmt(4, "", &c.write, ";")).collect::<Vec<_>>().join("\n");
    let read_body = if codes.is_empty() { atom(format!("const {name}()")) } else {
        call(name.clone(), codes.iter().map(|c| c.read.clone()).collect())
    };
    let static_ret = format!("static {name}");
    let read = arrow_or_block(2, &Sig::method(&static_ret, "bitisRead(", vec![atom("BitReader r")]), &read_body);
    let to_bytes = arrow(2, &Sig::plain("Uint8List", "toBytes()"), &call("bitisEncode", vec![atom("this")]));
    let from_bytes = arrow(2, &Sig::method(&static_ret, "fromBytes(", vec![atom("List<int> data")]),
                           &atom("bitisDecode(data, bitisRead).value"));
    let from_bytes_exact = arrow(2, &Sig::method(&static_ret, "fromBytesExact(", vec![atom("List<int> data")]),
                                 &call("bitisDecodeExact", vec![atom("data"), atom("bitisRead")]));
    let to_json = arrow(2, &Sig::plain("Map<String, Object?>", "toJson()"),
                        &E::Map(codes.iter().map(|c| c.json_entry.clone()).collect()));
    let from_json_body = if codes.is_empty() { atom(format!("const {name}()")) } else {
        call(name.clone(), codes.iter().map(|c| c.from_json.clone()).collect())
    };
    let from_json = arrow_or_block(2, &Sig::method("", &format!("factory {name}.fromJson("),
                                                   vec![atom("Map<String, dynamic> json")]), &from_json_body);
    let copy_with = if codes.is_empty() { String::new() } else {
        let sig = params_sig(2, &format!("{name} copyWith("), "{",
                             codes.iter().map(|c| c.copy_param.clone()).collect(), "})", " {");
        let ret = stmt(4, "return ", &call(name.clone(), codes.iter().map(|c| c.copy_arg.clone()).collect()), ";");
        format!("{sig}\n{ret}\n  }}")
    };
    let mut eq_ops = vec![bin("is", atom("other"), atom(name))];
    eq_ops.extend(codes.iter().map(|c| c.eq.clone()));
    let eq = arrow_and(2, "bool operator ==(Object other)", &eq_ops);
    let hash = if codes.is_empty() { "  int get hashCode => 0;".to_string() } else {
        arrow(2, &Sig::plain("int", "get hashCode"),
              &call("Object.hashAll", vec![E::List(String::new(), codes.iter().map(|c| c.hash.clone()).collect())]))
    };
    let mut parts = vec![format!("{name}(")];
    for (i, c) in codes.iter().enumerate() {
        let sep = if i + 1 == codes.len() { "" } else { ", " };
        parts.push(format!("{0}: ${0}{sep}", c.field.dart_name));
    }
    parts.push(")".to_string());
    // the closing parenthesis joins the last part
    let close = parts.pop().unwrap();
    parts.last_mut().unwrap().push_str(&close);
    let to_string = string_arrow(2, "String toString()", &parts);

    let oneofs = oneofs.iter().filter(|(mn, _)| mn == name).map(|(_, oo)| {
        let (attr_name, comment) = find_oneof_attr(&m.attributes, &oo.name).unwrap_or_default();
        build_oneof(ctx, name, &attr_name, comment, oo)
    }).collect();

    DartMsg {
        name: name.clone(), min_bits: min_bits_message(m, ctx.all_msgs),
        max_bits: match max_bits_message(m, ctx.all_msgs, ctx.all_enums) {
            Some(b) if b <= DART_MAX_SAFE_INT => b.to_string(),
            _ => "null".to_string(),
        },
        class_header: class_header(&format!("final class {name}"), "implements", "BitisMessage"),
        comment: m.comment.clone(), parent: m.parent.clone(),
        fields: codes.into_iter().map(|c| c.field).collect(),
        ctor, write_body, read, to_bytes, from_bytes, from_bytes_exact, to_json, from_json, copy_with, eq, hash,
        to_string, oneofs,
    }
}

/// The attribute (direct or nested in a oneof) defining the oneof with the generated name `oo_name`.
fn find_oneof_attr(attrs: &[Attribute], oo_name: &str) -> Option<(String, Option<String>)> {
    for a in attrs {
        if let AttributeDetails::AttributeOneOf(oo) = &a.specific_details {
            if oo.name == oo_name { return Some((a.name.clone(), a.comment.clone())); }
            if let Some(r) = find_oneof_attr(&oo.attributes, oo_name) { return Some(r); }
        }
    }
    None
}

fn build_enum(e: &Enum) -> DartEnum {
    let values: Vec<DartEnumValue> = e.values.iter().enumerate().map(|(i, v)| DartEnumValue {
        dart_name: v.to_camel_case(), bitis_name: v.clone(), comment: e.value_comment(i).cloned(),
    }).collect();
    let n = values.len();
    let name = &e.name;
    DartEnum {
        name: name.clone(), comment: e.comment.clone(), dyn_bits: e.bit_size,
        default_decl: decl_init(2, &format!("static const {name}"), "defaultValue",
                                &atom(format!("{name}.{}", e.default.to_camel_case()))),
        write: arrow(2, &Sig::method("void", "bitisWrite(", vec![atom("BitWriter w")]),
                     &call("w.writeEnumIndex", vec![atom("index"), atom(n.to_string()), atom(e.bit_size.to_string())])),
        read: arrow(2, &Sig::method(&format!("static {name}"), "bitisRead(", vec![atom("BitReader r")]),
                    &atom(format!("values[r.readEnumIndex({n}, {})]", e.bit_size))),
        from_json: arrow(2, &Sig::method(&format!("static {name}"), "fromJson(", vec![atom("Object? json")]),
                         &call("BitisJson.enumFromJson", vec![atom("json"), atom("_byBitisName")])),
        by_name: decl_init(2, &format!("static final Map<String, {name}>"), "_byBitisName",
                           &E::Map(vec![atom("for (final v in values) v.bitisName: v")])),
        values,
    }
}

/// Prepares the data for the Dart templates.
pub fn dart_data(p: &BitisProcessed, schema_hash: u64) -> DartData {
    let ctx = Ctx::new(p);
    let msgs: Vec<DartMsg> = p.msgs.iter().map(|m| build_msg(&ctx, m, &p.oo_enums)).collect();
    DartData {
        enums: p.enums.iter().map(build_enum).collect(),
        needs_typed_data: !msgs.is_empty(),
        msgs,
        schema_hash_hi: format!("0x{:08x}", schema_hash >> 32),
        schema_hash_lo: format!("0x{:08x}", schema_hash & 0xffff_ffff),
    }
}

/// Doc comment filter of the Dart templates.
pub fn dart_doc(s: &str, indent: usize) -> String { doc(s, indent) }
