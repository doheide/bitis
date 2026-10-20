#![allow(dead_code, non_snake_case, nonstandard_style)]

use bitis_lib::*;

use pyo3::prelude::*;
use pyo3::types::PyBytes;
use pyo3::PyClass;

use super::messages;

// Base function
fn do_val_from<T: ValFromInto<U>, U>(v: &U) -> T {
    T::val_from(v)
}
// Attributes with a maximum length [M]: the value is accepted if its Rust value encodes without error
fn check_bounds<T: BiserdiTrait + ValFromInto<U>, U>(v: &U, name: &str) -> PyResult<()> {
    match try_serialize(&T::val_from(v)) {
        Ok(_) => Ok(()),
        Err(e) => Err(encode_error(&format!("{:?}", e), &format!("{name}: {e} (longer than its maximum length or out of range)"))),
    }
}

// *****************************************************************
// *****************************************************************
// Exceptions (docs/WIRE_FORMAT.md section 3); `kind` is the spec error name
pub mod bitis_py_errors {
  pyo3::create_exception!(bounds, DecodeError, pyo3::exceptions::PyValueError,
    "Decoding failed. Attribute `kind`: UnexpectedEnd, InvalidValue, InvalidDiscriminant or TrailingData.");
  pyo3::create_exception!(bounds, EncodeError, pyo3::exceptions::PyValueError,
    "Encoding failed (try_serialize). Attribute `kind`: ValueOutOfRange.");
}
/// DecodeError with attribute `kind` (spec error name, e.g. "InvalidValue"); message "<kind>: <detail>".
pub fn decode_error(kind: &str, detail: &str) -> PyErr {
  let err = bitis_py_errors::DecodeError::new_err(format!("{kind}: {detail}"));
  Python::attach(|py| { let _ = err.value(py).setattr("kind", kind); });
  err
}
/// EncodeError with attribute `kind` (spec error name, e.g. "ValueOutOfRange"); message "<kind>: <detail>".
pub fn encode_error(kind: &str, detail: &str) -> PyErr {
  let err = bitis_py_errors::EncodeError::new_err(format!("{kind}: {detail}"));
  Python::attach(|py| { let _ = err.value(py).setattr("kind", kind); });
  err
}
fn decode_err(e: DecodeError) -> PyErr { decode_error(&format!("{:?}", e), &e.to_string()) }
fn encode_err(e: EncodeError) -> PyErr { encode_error(&format!("{:?}", e), &e.to_string()) }

// *****************************************************************
// *****************************************************************
// Checks values that the conversion to the Rust types would silently change
// (non-ASCII astr becomes the empty string); used by try_serialize.
trait BitisPyCheck { fn bitis_ok(&self, py: Python<'_>) -> bool; }
impl BitisPyCheck for String { fn bitis_ok(&self, _py: Python<'_>) -> bool { self.is_ascii() } }
macro_rules! bitis_py_check_always_ok {
  ($($t:ty),*) => { $(impl BitisPyCheck for $t { fn bitis_ok(&self, _py: Python<'_>) -> bool { true } })* }
}
bitis_py_check_always_ok!(bool, u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);
impl<T: BitisPyCheck> BitisPyCheck for Option<T> {
  fn bitis_ok(&self, py: Python<'_>) -> bool { self.as_ref().map_or(true, |v| v.bitis_ok(py)) }
}
impl<T: BitisPyCheck> BitisPyCheck for Vec<T> {
  fn bitis_ok(&self, py: Python<'_>) -> bool { self.iter().all(|v| v.bitis_ok(py)) }
}
impl<T: BitisPyCheck, const N: usize> BitisPyCheck for [T; N] {
  fn bitis_ok(&self, py: Python<'_>) -> bool { self.iter().all(|v| v.bitis_ok(py)) }
}
impl<T: BitisPyCheck + PyClass> BitisPyCheck for Py<T> {
  fn bitis_ok(&self, py: Python<'_>) -> bool { self.bind(py).borrow().bitis_ok(py) }
}

// *****************************************************************
// *****************************************************************
// Enums

#[pyclass(eq, eq_int, hash, frozen, from_py_object)]
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum Kind {
  Small,
  Large,
}
impl std::fmt::Display for Kind {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      Kind::Small => { write!(f, "Kind::Small") }, 
      Kind::Large => { write!(f, "Kind::Large") }, 
    }
} }

impl BitisPyCheck for Kind { fn bitis_ok(&self, _py: Python<'_>) -> bool { true } }
impl ValFromInto<Kind> for messages::Kind {
  fn val_into(&self) -> Kind {
    match self.clone() {
      messages::Kind::Small => Kind::Small, 
      messages::Kind::Large => Kind::Large, 
    }
  }
  fn val_from(v: &Kind) -> Self {
    match v {
      Kind::Small => messages::Kind::Small, 
      Kind::Large => messages::Kind::Large, 
    }
  }
}

// *****************************************************************
// *****************************************************************
// *** Enums for oneof

#[pyclass]
#[derive(Debug)]
#[allow(dead_code)]
pub enum OO_Bounded_Payload {
  Text(String),
  Raw(String),
  P(Py<Point>),
}

#[pyclass(eq, eq_int, hash, frozen)]
#[derive(Debug, PartialEq, Hash)]
pub enum OO_Bounded_PayloadEnum {
  Text,
  Raw,
  P,
}

#[pymethods]
impl OO_Bounded_Payload {
  #[staticmethod]
  fn new_text(py: Python<'_>, text: String) -> PyResult<Py<Self>> {
    check_bounds::<BoundedAString<2, 5>, _>(&text, "OO_Bounded_Payload.text")?;
    Py::new(py, Self::Text(text))
  }
  #[staticmethod]
  fn new_raw(py: Python<'_>, raw: String) -> PyResult<Py<Self>> {
    check_bounds::<BoundedAString<4, 6>, _>(&raw, "OO_Bounded_Payload.raw")?;
    Py::new(py, Self::Raw(raw))
  }
  #[staticmethod]
  fn new_p(py: Python<'_>, p: Py<Point>) -> PyResult<Py<Self>> {
    Py::new(py, Self::P(p))
  }
  fn __repr__(&self) -> String { format!("{}", self) }
}
impl BitisPyCheck for OO_Bounded_Payload {
  fn bitis_ok(&self, py: Python<'_>) -> bool {
    match self {
      Self::Text(v) => v.bitis_ok(py),
      Self::Raw(v) => v.bitis_ok(py),
      Self::P(v) => v.bitis_ok(py),
    }
  }
}
impl OO_Bounded_Payload {
  pub fn to_rust(&self) -> messages::OO_Bounded_Payload {
    Python::attach(|_py| {
      match self {Self::Text(v) => messages::OO_Bounded_Payload::Text(do_val_from(v)),
Self::Raw(v) => messages::OO_Bounded_Payload::Raw(do_val_from(v)),
Self::P(v) => messages::OO_Bounded_Payload::P(do_val_from(v)),

      }
    })
  }
  pub fn from_rust_obj(d: &messages::OO_Bounded_Payload) -> PyResult<Self> {
    let r = Python::attach(|_py| {
      match d {messages::OO_Bounded_Payload::Text(v) => Self::Text(v.val_into()),
messages::OO_Bounded_Payload::Raw(v) => Self::Raw(v.val_into()),
messages::OO_Bounded_Payload::P(v) => Self::P(v.val_into()),

      }
    });
    Ok(r)
  }
}
impl std::fmt::Display for OO_Bounded_Payload {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      OO_Bounded_Payload::Text(v) => { write!(f, "OO_Bounded_Payload(Text({}))", v) },
      OO_Bounded_Payload::Raw(v) => { write!(f, "OO_Bounded_Payload(Raw({}))", v) },
      OO_Bounded_Payload::P(v) => {
        Python::attach(|py| {
          write!(f, "OO_Bounded_Payload(P({})", v.borrow(py).__repr__()) }
        )},
    }
  }
}

impl ValFromInto<Py<OO_Bounded_Payload>> for messages::OO_Bounded_Payload {
  fn val_into(self: &Self) -> Py<OO_Bounded_Payload> {
    Python::attach(|_py| { Py::new(_py, OO_Bounded_Payload::from_rust_obj(self).unwrap()).unwrap() })
  }
  fn val_from(val: &Py<OO_Bounded_Payload>) -> Self {
    Python::attach(|_py| { val.borrow(_py).to_rust() })
  }
}


// *****************************************************************
// *****************************************************************
// *** Messages

#[pyclass]
#[derive(Debug)]
pub struct Point {
  #[pyo3(get, set)]
  pub x: u16,
  #[pyo3(get, set)]
  pub y: u16,
}
#[pymethods]
impl Point {
  #[new]  #[allow(non_snake_case)]
  pub fn __new__( x: u16, y: u16,) -> Self {
    Self{ x: x.into(), y: y.into(), }
  }
  #[staticmethod]
  pub fn default() -> PyResult<Self> { Self::from_rust_obj(&Default::default()) }

  /// Legacy: values outside their declared range are truncated, a non-ASCII astr becomes empty.
  /// Returns (data, total_bits, total_bytes).
  pub fn serialize<'py>(&self, py: Python<'py>) -> (Bound<'py, PyBytes>, u64, u64) {
    let msg = self.to_rust();
    let r = serialize(&msg);
    (PyBytes::new(py, &r.0), r.1.total_bits, r.1.total_bytes)
  }
  /// Like serialize, but raises EncodeError (kind ValueOutOfRange) instead of truncating.
  pub fn try_serialize<'py>(&self, py: Python<'py>) -> PyResult<(Bound<'py, PyBytes>, u64, u64)> {
    if !self.bitis_ok(py) { return Err(encode_err(EncodeError::ValueOutOfRange)); }
    let r = try_serialize(&self.to_rust()).map_err(|e| encode_err(e))?;
    Ok((PyBytes::new(py, &r.0), r.1.total_bits, r.1.total_bytes))
  }
  /// Decodes from the start of data (trailing bytes are ignored); raises DecodeError.
  #[staticmethod]
  pub fn deserialize(py: Python<'_>, data: Bound<'_, PyBytes>) -> PyResult<Self> {
    let v = try_deserialize::<messages::Point>(data.as_bytes()).map_err(|e| decode_err(e))?;
    Self::from_rust_obj(&v.0)
  }
  /// Like deserialize, but returns (value, total_bits, total_bytes) with the consumed size.
  #[staticmethod]
  pub fn try_deserialize(py: Python<'_>, data: Bound<'_, PyBytes>) -> PyResult<(Self, u64, u64)> {
    let v = try_deserialize::<messages::Point>(data.as_bytes()).map_err(|e| decode_err(e))?;
    Ok((Self::from_rust_obj(&v.0)?, v.1.total_bits, v.1.total_bytes))
  }
  /// Like deserialize, but raises DecodeError (kind TrailingData) if data is longer than the message.
  #[staticmethod]
  pub fn try_deserialize_exact(py: Python<'_>, data: Bound<'_, PyBytes>) -> PyResult<Self> {
    let v = try_deserialize_exact::<messages::Point>(data.as_bytes()).map_err(|e| decode_err(e))?;
    Self::from_rust_obj(&v)
  }
  /// Lower bound of the encoded size in bits (spec section 4).
  #[staticmethod]
  pub fn min_bits() -> u64 { <messages::Point as BiserdiTrait>::min_bits() }
  /// Upper bound of the encoded size in bits, None if unbounded (spec section 4).
  #[staticmethod]
  pub fn max_bits() -> Option<u64> { <messages::Point as BiserdiTrait>::max_bits() }
  pub fn __repr__(&self) -> String {
    format!("{}", self)
  }

}
impl BitisPyCheck for Point {
  fn bitis_ok(&self, _py: Python<'_>) -> bool {
    true && self.x.bitis_ok(_py) && self.y.bitis_ok(_py)
  }
}
impl Point {
  pub fn to_rust(&self) -> messages::Point {
    Python::attach(|_py| {
      messages::Point{ x: do_val_from(&self.x), y: do_val_from(&self.y),}
    })
  }
  pub fn from_rust_obj(_d: &messages::Point) -> PyResult<Self> {
    let r = Python::attach(|_py| { Self{ x: _d.x.val_into(), y: _d.y.val_into(),} });
    Ok(r)
  }
}
impl std::fmt::Display for Point {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    Python::attach(|_py| {
      let t = self.to_rust();
      write!(f, "{}", t)
    })
  }
}
impl ValFromInto<Py<Point>> for messages::Point {
  fn val_into(self: &Self) -> Py<Point> {
    Python::attach(|_py| { Py::new(_py, Point::from_rust_obj(self).unwrap()).unwrap() })
  }
  fn val_from(val: &Py<Point>) -> Self {
    Python::attach(|_py| { val.borrow(_py).to_rust() })
  }
}


/// Bounded strings, bytes and lists
#[pyclass]
#[derive(Debug)]
pub struct Bounded {
  #[pyo3(get)]
  pub name: String,
  #[pyo3(get)]
  pub blob: Vec<u8>,
  #[pyo3(get)]
  pub tags: Vec<String>,
  #[pyo3(get)]
  pub note: Option<String>,
  #[pyo3(get)]
  pub points: Vec<Py<Point>>,
  #[pyo3(get)]
  pub kinds: Vec<Kind>,
  #[pyo3(get)]
  pub pair: [Vec<u8>;2],
  #[pyo3(get, set)]
  pub unbounded: String,
  pub payload: Py<OO_Bounded_Payload>,
}
#[pymethods]
impl Bounded {
  #[new]  #[allow(non_snake_case)]
  pub fn __new__( name: String, blob: Vec<u8>, tags: Vec<String>, note: Option<String>, points: Vec<Py<Point>>, kinds: Vec<Kind>, pair: [Vec<u8>;2], unbounded: String, payload: Py<OO_Bounded_Payload>,) -> PyResult<Self> {
    check_bounds::<BoundedAString<4, 16>, _>(&name, "Bounded.name")?;
    check_bounds::<BoundedBinary<3, 8>, _>(&blob, "Bounded.blob")?;
    check_bounds::<BoundedDynArray<BoundedAString<4, 8>, 2, 4>, _>(&tags, "Bounded.tags")?;
    check_bounds::<BitisOption<BoundedAString<4, 32>>, _>(&note, "Bounded.note")?;
    check_bounds::<BoundedDynArray<messages::Point, 3, 3>, _>(&points, "Bounded.points")?;
    check_bounds::<BoundedDynArray<messages::Kind, 1, 2>, _>(&kinds, "Bounded.kinds")?;
    check_bounds::<FixedArray<BoundedBinary<2, 3>, 2>, _>(&pair, "Bounded.pair")?;
    Ok(Self{ name: name.into(), blob: blob.into(), tags: tags.into(), note: note.into(), points: points.into(), kinds: kinds.into(), pair: pair.into(), unbounded: unbounded.into(), payload: payload.into(), })
  }
  #[staticmethod]
  pub fn default() -> PyResult<Self> { Self::from_rust_obj(&Default::default()) }

  /// Legacy: values outside their declared range are truncated, a non-ASCII astr becomes empty.
  /// Returns (data, total_bits, total_bytes).
  pub fn serialize<'py>(&self, py: Python<'py>) -> (Bound<'py, PyBytes>, u64, u64) {
    let msg = self.to_rust();
    let r = serialize(&msg);
    (PyBytes::new(py, &r.0), r.1.total_bits, r.1.total_bytes)
  }
  /// Like serialize, but raises EncodeError (kind ValueOutOfRange) instead of truncating.
  pub fn try_serialize<'py>(&self, py: Python<'py>) -> PyResult<(Bound<'py, PyBytes>, u64, u64)> {
    if !self.bitis_ok(py) { return Err(encode_err(EncodeError::ValueOutOfRange)); }
    let r = try_serialize(&self.to_rust()).map_err(|e| encode_err(e))?;
    Ok((PyBytes::new(py, &r.0), r.1.total_bits, r.1.total_bytes))
  }
  /// Decodes from the start of data (trailing bytes are ignored); raises DecodeError.
  #[staticmethod]
  pub fn deserialize(py: Python<'_>, data: Bound<'_, PyBytes>) -> PyResult<Self> {
    let v = try_deserialize::<messages::Bounded>(data.as_bytes()).map_err(|e| decode_err(e))?;
    Self::from_rust_obj(&v.0)
  }
  /// Like deserialize, but returns (value, total_bits, total_bytes) with the consumed size.
  #[staticmethod]
  pub fn try_deserialize(py: Python<'_>, data: Bound<'_, PyBytes>) -> PyResult<(Self, u64, u64)> {
    let v = try_deserialize::<messages::Bounded>(data.as_bytes()).map_err(|e| decode_err(e))?;
    Ok((Self::from_rust_obj(&v.0)?, v.1.total_bits, v.1.total_bytes))
  }
  /// Like deserialize, but raises DecodeError (kind TrailingData) if data is longer than the message.
  #[staticmethod]
  pub fn try_deserialize_exact(py: Python<'_>, data: Bound<'_, PyBytes>) -> PyResult<Self> {
    let v = try_deserialize_exact::<messages::Bounded>(data.as_bytes()).map_err(|e| decode_err(e))?;
    Self::from_rust_obj(&v)
  }
  /// Lower bound of the encoded size in bits (spec section 4).
  #[staticmethod]
  pub fn min_bits() -> u64 { <messages::Bounded as BiserdiTrait>::min_bits() }
  /// Upper bound of the encoded size in bits, None if unbounded (spec section 4).
  #[staticmethod]
  pub fn max_bits() -> Option<u64> { <messages::Bounded as BiserdiTrait>::max_bits() }
  pub fn __repr__(&self) -> String {
    format!("{}", self)
  }
  #[setter(name)]
  fn __bitis_set_name(&mut self, v: String,) -> PyResult<()> {
    check_bounds::<BoundedAString<4, 16>, _>(&v, "Bounded.name")?;
    self.name = v;
    Ok(())
  }
  #[setter(blob)]
  fn __bitis_set_blob(&mut self, v: Vec<u8>,) -> PyResult<()> {
    check_bounds::<BoundedBinary<3, 8>, _>(&v, "Bounded.blob")?;
    self.blob = v;
    Ok(())
  }
  #[setter(tags)]
  fn __bitis_set_tags(&mut self, v: Vec<String>,) -> PyResult<()> {
    check_bounds::<BoundedDynArray<BoundedAString<4, 8>, 2, 4>, _>(&v, "Bounded.tags")?;
    self.tags = v;
    Ok(())
  }
  #[setter(note)]
  fn __bitis_set_note(&mut self, v: Option<String>,) -> PyResult<()> {
    check_bounds::<BitisOption<BoundedAString<4, 32>>, _>(&v, "Bounded.note")?;
    self.note = v;
    Ok(())
  }
  #[setter(points)]
  fn __bitis_set_points(&mut self, v: Vec<Py<Point>>,) -> PyResult<()> {
    check_bounds::<BoundedDynArray<messages::Point, 3, 3>, _>(&v, "Bounded.points")?;
    self.points = v;
    Ok(())
  }
  #[setter(kinds)]
  fn __bitis_set_kinds(&mut self, v: Vec<Kind>,) -> PyResult<()> {
    check_bounds::<BoundedDynArray<messages::Kind, 1, 2>, _>(&v, "Bounded.kinds")?;
    self.kinds = v;
    Ok(())
  }
  #[setter(pair)]
  fn __bitis_set_pair(&mut self, v: [Vec<u8>;2],) -> PyResult<()> {
    check_bounds::<FixedArray<BoundedBinary<2, 3>, 2>, _>(&v, "Bounded.pair")?;
    self.pair = v;
    Ok(())
  }

  #[getter]
  pub fn payload_oo(&self) -> OO_Bounded_PayloadEnum {
    match self.payload.get() {
      OO_Bounded_Payload::Text(_) => OO_Bounded_PayloadEnum::Text,
      OO_Bounded_Payload::Raw(_) => OO_Bounded_PayloadEnum::Raw,
      OO_Bounded_Payload::P(_) => OO_Bounded_PayloadEnum::P,
  } }
  #[getter(payload_text)]
  fn payload_get_text(&self) -> Option<&String> {
    match self.payload.get() {
      OO_Bounded_Payload::Text(v) => Some(v), _ => None
  } }
  #[setter(payload_text)]
  fn payload_set_text(&mut self, v:String) -> PyResult<()> {
    Python::attach(|py| {
      check_bounds::<BoundedAString<2, 5>, _>(&v, "OO_Bounded_Payload.text")?;
      self.payload = Py::new(py, OO_Bounded_Payload::Text(v))?;
      Ok(())
    })
  }
  #[getter(payload_raw)]
  fn payload_get_raw(&self) -> Option<&String> {
    match self.payload.get() {
      OO_Bounded_Payload::Raw(v) => Some(v), _ => None
  } }
  #[setter(payload_raw)]
  fn payload_set_raw(&mut self, v:String) -> PyResult<()> {
    Python::attach(|py| {
      check_bounds::<BoundedAString<4, 6>, _>(&v, "OO_Bounded_Payload.raw")?;
      self.payload = Py::new(py, OO_Bounded_Payload::Raw(v))?;
      Ok(())
    })
  }
  #[getter(payload_p)]
  fn payload_get_p(&self) -> Option<&Py<Point>> {
    match self.payload.get() {
      OO_Bounded_Payload::P(v) => Some(v), _ => None
  } }
  #[setter(payload_p)]
  fn payload_set_p(&mut self, v:Py<Point>) -> PyResult<()> {
    Python::attach(|py| {
      self.payload = Py::new(py, OO_Bounded_Payload::P(v))?;
      Ok(())
    })
  }
}
impl BitisPyCheck for Bounded {
  fn bitis_ok(&self, _py: Python<'_>) -> bool {
    true && self.name.bitis_ok(_py) && self.blob.bitis_ok(_py) && self.tags.bitis_ok(_py) && self.note.bitis_ok(_py) && self.points.bitis_ok(_py) && self.kinds.bitis_ok(_py) && self.pair.bitis_ok(_py) && self.unbounded.bitis_ok(_py) && self.payload.bitis_ok(_py)
  }
}
impl Bounded {
  pub fn to_rust(&self) -> messages::Bounded {
    Python::attach(|_py| {
      messages::Bounded{ name: do_val_from(&self.name), blob: do_val_from(&self.blob), tags: do_val_from(&self.tags), note: do_val_from(&self.note), points: do_val_from(&self.points), kinds: do_val_from(&self.kinds), pair: do_val_from(&self.pair), unbounded: do_val_from(&self.unbounded), payload: do_val_from(&self.payload),}
    })
  }
  pub fn from_rust_obj(_d: &messages::Bounded) -> PyResult<Self> {
    let r = Python::attach(|_py| { Self{ name: _d.name.val_into(), blob: _d.blob.val_into(), tags: _d.tags.val_into(), note: _d.note.val_into(), points: _d.points.val_into(), kinds: _d.kinds.val_into(), pair: _d.pair.val_into(), unbounded: _d.unbounded.val_into(), payload: _d.payload.val_into(),} });
    Ok(r)
  }
}
impl std::fmt::Display for Bounded {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    Python::attach(|_py| {
      let t = self.to_rust();
      write!(f, "{}", t)
    })
  }
}
impl ValFromInto<Py<Bounded>> for messages::Bounded {
  fn val_into(self: &Self) -> Py<Bounded> {
    Python::attach(|_py| { Py::new(_py, Bounded::from_rust_obj(self).unwrap()).unwrap() })
  }
  fn val_from(val: &Py<Bounded>) -> Self {
    Python::attach(|_py| { val.borrow(_py).to_rust() })
  }
}


