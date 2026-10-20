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
  pyo3::create_exception!(all_types, DecodeError, pyo3::exceptions::PyValueError,
    "Decoding failed. Attribute `kind`: UnexpectedEnd, InvalidValue, InvalidDiscriminant or TrailingData.");
  pyo3::create_exception!(all_types, EncodeError, pyo3::exceptions::PyValueError,
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
pub enum Color {
  Red,
  Green,
  BlueLight,
}
impl std::fmt::Display for Color {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      Color::Red => { write!(f, "Color::Red") }, 
      Color::Green => { write!(f, "Color::Green") }, 
      Color::BlueLight => { write!(f, "Color::BlueLight") }, 
    }
} }

impl BitisPyCheck for Color { fn bitis_ok(&self, _py: Python<'_>) -> bool { true } }
impl ValFromInto<Color> for messages::Color {
  fn val_into(&self) -> Color {
    match self.clone() {
      messages::Color::Red => Color::Red, 
      messages::Color::Green => Color::Green, 
      messages::Color::BlueLight => Color::BlueLight, 
    }
  }
  fn val_from(v: &Color) -> Self {
    match v {
      Color::Red => messages::Color::Red, 
      Color::Green => messages::Color::Green, 
      Color::BlueLight => messages::Color::BlueLight, 
    }
  }
}

// *****************************************************************
// *****************************************************************
// *** Enums for oneof

#[pyclass]
#[derive(Debug)]
#[allow(dead_code)]
pub enum OO_AllTypes_Choice {
  Flag(bool),
  Number(u16),
  InnerVal(Py<Inner>),
  ColorVal(Color),
  Text(String),
  Nothing(Py<Empty>),
}

#[pyclass(eq, eq_int, hash, frozen)]
#[derive(Debug, PartialEq, Hash)]
pub enum OO_AllTypes_ChoiceEnum {
  Flag,
  Number,
  InnerVal,
  ColorVal,
  Text,
  Nothing,
}

#[pymethods]
impl OO_AllTypes_Choice {
  #[staticmethod]
  fn new_flag(py: Python<'_>, flag: bool) -> PyResult<Py<Self>> {
    Py::new(py, Self::Flag(flag))
  }
  #[staticmethod]
  fn new_number(py: Python<'_>, number: u16) -> PyResult<Py<Self>> {
    Py::new(py, Self::Number(number))
  }
  #[staticmethod]
  fn new_inner_val(py: Python<'_>, inner_val: Py<Inner>) -> PyResult<Py<Self>> {
    Py::new(py, Self::InnerVal(inner_val))
  }
  #[staticmethod]
  fn new_color_val(py: Python<'_>, color_val: Color) -> PyResult<Py<Self>> {
    Py::new(py, Self::ColorVal(color_val))
  }
  #[staticmethod]
  fn new_text(py: Python<'_>, text: String) -> PyResult<Py<Self>> {
    Py::new(py, Self::Text(text))
  }
  #[staticmethod]
  fn new_nothing(py: Python<'_>, nothing: Py<Empty>) -> PyResult<Py<Self>> {
    Py::new(py, Self::Nothing(nothing))
  }
  fn __repr__(&self) -> String { format!("{}", self) }
}
impl BitisPyCheck for OO_AllTypes_Choice {
  fn bitis_ok(&self, py: Python<'_>) -> bool {
    match self {
      Self::Flag(v) => v.bitis_ok(py),
      Self::Number(v) => v.bitis_ok(py),
      Self::InnerVal(v) => v.bitis_ok(py),
      Self::ColorVal(v) => v.bitis_ok(py),
      Self::Text(v) => v.bitis_ok(py),
      Self::Nothing(v) => v.bitis_ok(py),
    }
  }
}
impl OO_AllTypes_Choice {
  pub fn to_rust(&self) -> messages::OO_AllTypes_Choice {
    Python::attach(|_py| {
      match self {Self::Flag(v) => messages::OO_AllTypes_Choice::Flag(do_val_from(v)),
Self::Number(v) => messages::OO_AllTypes_Choice::Number(do_val_from(v)),
Self::InnerVal(v) => messages::OO_AllTypes_Choice::InnerVal(do_val_from(v)),
Self::ColorVal(v) => messages::OO_AllTypes_Choice::ColorVal(do_val_from(v)),
Self::Text(v) => messages::OO_AllTypes_Choice::Text(do_val_from(v)),
Self::Nothing(v) => messages::OO_AllTypes_Choice::Nothing(do_val_from(v)),

      }
    })
  }
  pub fn from_rust_obj(d: &messages::OO_AllTypes_Choice) -> PyResult<Self> {
    let r = Python::attach(|_py| {
      match d {messages::OO_AllTypes_Choice::Flag(v) => Self::Flag(v.val_into()),
messages::OO_AllTypes_Choice::Number(v) => Self::Number(v.val_into()),
messages::OO_AllTypes_Choice::InnerVal(v) => Self::InnerVal(v.val_into()),
messages::OO_AllTypes_Choice::ColorVal(v) => Self::ColorVal(v.val_into()),
messages::OO_AllTypes_Choice::Text(v) => Self::Text(v.val_into()),
messages::OO_AllTypes_Choice::Nothing(v) => Self::Nothing(v.val_into()),

      }
    });
    Ok(r)
  }
}
impl std::fmt::Display for OO_AllTypes_Choice {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      OO_AllTypes_Choice::Flag(v) => { write!(f, "OO_AllTypes_Choice(Flag({}))", v) },
      OO_AllTypes_Choice::Number(v) => { write!(f, "OO_AllTypes_Choice(Number({}))", v) },
      OO_AllTypes_Choice::InnerVal(v) => {
        Python::attach(|py| {
          write!(f, "OO_AllTypes_Choice(InnerVal({})", v.borrow(py).__repr__()) }
        )},
      OO_AllTypes_Choice::ColorVal(v) => { write!(f, "OO_AllTypes_Choice(ColorVal({}))", v) },
      OO_AllTypes_Choice::Text(v) => { write!(f, "OO_AllTypes_Choice(Text({}))", v) },
      OO_AllTypes_Choice::Nothing(v) => {
        Python::attach(|py| {
          write!(f, "OO_AllTypes_Choice(Nothing({})", v.borrow(py).__repr__()) }
        )},
    }
  }
}

impl ValFromInto<Py<OO_AllTypes_Choice>> for messages::OO_AllTypes_Choice {
  fn val_into(self: &Self) -> Py<OO_AllTypes_Choice> {
    Python::attach(|_py| { Py::new(_py, OO_AllTypes_Choice::from_rust_obj(self).unwrap()).unwrap() })
  }
  fn val_from(val: &Py<OO_AllTypes_Choice>) -> Self {
    Python::attach(|_py| { val.borrow(_py).to_rust() })
  }
}


// *****************************************************************
// *****************************************************************
// *** Messages

#[pyclass]
#[derive(Debug)]
pub struct Inner {
  #[pyo3(get, set)]
  pub a: u16,
}
#[pymethods]
impl Inner {
  #[new]  #[allow(non_snake_case)]
  pub fn __new__( a: u16,) -> Self {
    Self{ a: a.into(), }
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
    let v = try_deserialize::<messages::Inner>(data.as_bytes()).map_err(|e| decode_err(e))?;
    Self::from_rust_obj(&v.0)
  }
  /// Like deserialize, but returns (value, total_bits, total_bytes) with the consumed size.
  #[staticmethod]
  pub fn try_deserialize(py: Python<'_>, data: Bound<'_, PyBytes>) -> PyResult<(Self, u64, u64)> {
    let v = try_deserialize::<messages::Inner>(data.as_bytes()).map_err(|e| decode_err(e))?;
    Ok((Self::from_rust_obj(&v.0)?, v.1.total_bits, v.1.total_bytes))
  }
  /// Like deserialize, but raises DecodeError (kind TrailingData) if data is longer than the message.
  #[staticmethod]
  pub fn try_deserialize_exact(py: Python<'_>, data: Bound<'_, PyBytes>) -> PyResult<Self> {
    let v = try_deserialize_exact::<messages::Inner>(data.as_bytes()).map_err(|e| decode_err(e))?;
    Self::from_rust_obj(&v)
  }
  /// Lower bound of the encoded size in bits (spec section 4).
  #[staticmethod]
  pub fn min_bits() -> u64 { <messages::Inner as BiserdiTrait>::min_bits() }
  /// Upper bound of the encoded size in bits, None if unbounded (spec section 4).
  #[staticmethod]
  pub fn max_bits() -> Option<u64> { <messages::Inner as BiserdiTrait>::max_bits() }
  pub fn __repr__(&self) -> String {
    format!("{}", self)
  }

}
impl BitisPyCheck for Inner {
  fn bitis_ok(&self, _py: Python<'_>) -> bool {
    true && self.a.bitis_ok(_py)
  }
}
impl Inner {
  pub fn to_rust(&self) -> messages::Inner {
    Python::attach(|_py| {
      messages::Inner{ a: do_val_from(&self.a),}
    })
  }
  pub fn from_rust_obj(_d: &messages::Inner) -> PyResult<Self> {
    let r = Python::attach(|_py| { Self{ a: _d.a.val_into(),} });
    Ok(r)
  }
}
impl std::fmt::Display for Inner {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    Python::attach(|_py| {
      let t = self.to_rust();
      write!(f, "{}", t)
    })
  }
}
impl ValFromInto<Py<Inner>> for messages::Inner {
  fn val_into(self: &Self) -> Py<Inner> {
    Python::attach(|_py| { Py::new(_py, Inner::from_rust_obj(self).unwrap()).unwrap() })
  }
  fn val_from(val: &Py<Inner>) -> Self {
    Python::attach(|_py| { val.borrow(_py).to_rust() })
  }
}


#[pyclass]
#[derive(Debug)]
pub struct Empty {
}
#[pymethods]
impl Empty {
  #[new]  #[allow(non_snake_case)]
  pub fn __new__() -> Self {
    Self{ }
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
    let v = try_deserialize::<messages::Empty>(data.as_bytes()).map_err(|e| decode_err(e))?;
    Self::from_rust_obj(&v.0)
  }
  /// Like deserialize, but returns (value, total_bits, total_bytes) with the consumed size.
  #[staticmethod]
  pub fn try_deserialize(py: Python<'_>, data: Bound<'_, PyBytes>) -> PyResult<(Self, u64, u64)> {
    let v = try_deserialize::<messages::Empty>(data.as_bytes()).map_err(|e| decode_err(e))?;
    Ok((Self::from_rust_obj(&v.0)?, v.1.total_bits, v.1.total_bytes))
  }
  /// Like deserialize, but raises DecodeError (kind TrailingData) if data is longer than the message.
  #[staticmethod]
  pub fn try_deserialize_exact(py: Python<'_>, data: Bound<'_, PyBytes>) -> PyResult<Self> {
    let v = try_deserialize_exact::<messages::Empty>(data.as_bytes()).map_err(|e| decode_err(e))?;
    Self::from_rust_obj(&v)
  }
  /// Lower bound of the encoded size in bits (spec section 4).
  #[staticmethod]
  pub fn min_bits() -> u64 { <messages::Empty as BiserdiTrait>::min_bits() }
  /// Upper bound of the encoded size in bits, None if unbounded (spec section 4).
  #[staticmethod]
  pub fn max_bits() -> Option<u64> { <messages::Empty as BiserdiTrait>::max_bits() }
  pub fn __repr__(&self) -> String {
    format!("{}", self)
  }

}
impl BitisPyCheck for Empty {
  fn bitis_ok(&self, _py: Python<'_>) -> bool {
    true
  }
}
impl Empty {
  pub fn to_rust(&self) -> messages::Empty {
    Python::attach(|_py| {
      messages::Empty{}
    })
  }
  pub fn from_rust_obj(_d: &messages::Empty) -> PyResult<Self> {
    let r = Python::attach(|_py| { Self{} });
    Ok(r)
  }
}
impl std::fmt::Display for Empty {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    Python::attach(|_py| {
      let t = self.to_rust();
      write!(f, "{}", t)
    })
  }
}
impl ValFromInto<Py<Empty>> for messages::Empty {
  fn val_into(self: &Self) -> Py<Empty> {
    Python::attach(|_py| { Py::new(_py, Empty::from_rust_obj(self).unwrap()).unwrap() })
  }
  fn val_from(val: &Py<Empty>) -> Self {
    Python::attach(|_py| { val.borrow(_py).to_rust() })
  }
}


#[pyclass]
#[derive(Debug)]
pub struct AllTypes {
  #[pyo3(get, set)]
  pub b: bool,
  #[pyo3(get, set)]
  pub u1: u16,
  #[pyo3(get, set)]
  pub u17: u32,
  #[pyo3(get, set)]
  pub u64v: u64,
  #[pyo3(get, set)]
  pub i5: i16,
  #[pyo3(get, set)]
  pub i33: i64,
  #[pyo3(get, set)]
  pub ud: u32,
  #[pyo3(get, set)]
  pub id: i16,
  #[pyo3(get, set)]
  pub f: f32,
  #[pyo3(get, set)]
  pub d: f64,
  #[pyo3(get, set)]
  pub fp: f64,
  #[pyo3(get, set)]
  pub lat: f64,
  #[pyo3(get, set)]
  pub s: String,
  #[pyo3(get, set)]
  pub data: Vec<u8>,
  #[pyo3(get, set)]
  pub color: Color,
  #[pyo3(get, set)]
  pub inner: Py<Inner>,
  #[pyo3(get, set)]
  pub empty: Py<Empty>,
  #[pyo3(get, set)]
  pub opt_u: Option<u16>,
  #[pyo3(get, set)]
  pub opt_inner: Option<Py<Inner>>,
  #[pyo3(get, set)]
  pub opt_empty: Option<Py<Empty>>,
  #[pyo3(get, set)]
  pub rf_b: [bool;3],
  #[pyo3(get, set)]
  pub rf_inner: [Py<Inner>;2],
  #[pyo3(get, set)]
  pub rd_u: Vec<u16>,
  #[pyo3(get, set)]
  pub rd_color: Vec<Color>,
  #[pyo3(get, set)]
  pub rd_inner: Vec<Py<Inner>>,
  pub choice: Py<OO_AllTypes_Choice>,
}
#[pymethods]
impl AllTypes {
  #[new]  #[allow(non_snake_case)]
  pub fn __new__( b: bool, u1: u16, u17: u32, u64v: u64, i5: i16, i33: i64, ud: u32, id: i16, f: f32, d: f64, fp: f64, lat: f64, s: String, data: Vec<u8>, color: Color, inner: Py<Inner>, empty: Py<Empty>, opt_u: Option<u16>, opt_inner: Option<Py<Inner>>, opt_empty: Option<Py<Empty>>, rf_b: [bool;3], rf_inner: [Py<Inner>;2], rd_u: Vec<u16>, rd_color: Vec<Color>, rd_inner: Vec<Py<Inner>>, choice: Py<OO_AllTypes_Choice>,) -> Self {
    Self{ b: b.into(), u1: u1.into(), u17: u17.into(), u64v: u64v.into(), i5: i5.into(), i33: i33.into(), ud: ud.into(), id: id.into(), f: f.into(), d: d.into(), fp: fp.into(), lat: lat.into(), s: s.into(), data: data.into(), color: color.into(), inner: inner.into(), empty: empty.into(), opt_u: opt_u.into(), opt_inner: opt_inner.into(), opt_empty: opt_empty.into(), rf_b: rf_b.into(), rf_inner: rf_inner.into(), rd_u: rd_u.into(), rd_color: rd_color.into(), rd_inner: rd_inner.into(), choice: choice.into(), }
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
    let v = try_deserialize::<messages::AllTypes>(data.as_bytes()).map_err(|e| decode_err(e))?;
    Self::from_rust_obj(&v.0)
  }
  /// Like deserialize, but returns (value, total_bits, total_bytes) with the consumed size.
  #[staticmethod]
  pub fn try_deserialize(py: Python<'_>, data: Bound<'_, PyBytes>) -> PyResult<(Self, u64, u64)> {
    let v = try_deserialize::<messages::AllTypes>(data.as_bytes()).map_err(|e| decode_err(e))?;
    Ok((Self::from_rust_obj(&v.0)?, v.1.total_bits, v.1.total_bytes))
  }
  /// Like deserialize, but raises DecodeError (kind TrailingData) if data is longer than the message.
  #[staticmethod]
  pub fn try_deserialize_exact(py: Python<'_>, data: Bound<'_, PyBytes>) -> PyResult<Self> {
    let v = try_deserialize_exact::<messages::AllTypes>(data.as_bytes()).map_err(|e| decode_err(e))?;
    Self::from_rust_obj(&v)
  }
  /// Lower bound of the encoded size in bits (spec section 4).
  #[staticmethod]
  pub fn min_bits() -> u64 { <messages::AllTypes as BiserdiTrait>::min_bits() }
  /// Upper bound of the encoded size in bits, None if unbounded (spec section 4).
  #[staticmethod]
  pub fn max_bits() -> Option<u64> { <messages::AllTypes as BiserdiTrait>::max_bits() }
  pub fn __repr__(&self) -> String {
    format!("{}", self)
  }

  #[getter]
  pub fn choice_oo(&self) -> OO_AllTypes_ChoiceEnum {
    match self.choice.get() {
      OO_AllTypes_Choice::Flag(_) => OO_AllTypes_ChoiceEnum::Flag,
      OO_AllTypes_Choice::Number(_) => OO_AllTypes_ChoiceEnum::Number,
      OO_AllTypes_Choice::InnerVal(_) => OO_AllTypes_ChoiceEnum::InnerVal,
      OO_AllTypes_Choice::ColorVal(_) => OO_AllTypes_ChoiceEnum::ColorVal,
      OO_AllTypes_Choice::Text(_) => OO_AllTypes_ChoiceEnum::Text,
      OO_AllTypes_Choice::Nothing(_) => OO_AllTypes_ChoiceEnum::Nothing,
  } }
  #[getter(choice_flag)]
  fn choice_get_flag(&self) -> Option<&bool> {
    match self.choice.get() {
      OO_AllTypes_Choice::Flag(v) => Some(v), _ => None
  } }
  #[setter(choice_flag)]
  fn choice_set_flag(&mut self, v:bool) -> PyResult<()> {
    Python::attach(|py| {
      self.choice = Py::new(py, OO_AllTypes_Choice::Flag(v))?;
      Ok(())
    })
  }
  #[getter(choice_number)]
  fn choice_get_number(&self) -> Option<&u16> {
    match self.choice.get() {
      OO_AllTypes_Choice::Number(v) => Some(v), _ => None
  } }
  #[setter(choice_number)]
  fn choice_set_number(&mut self, v:u16) -> PyResult<()> {
    Python::attach(|py| {
      self.choice = Py::new(py, OO_AllTypes_Choice::Number(v))?;
      Ok(())
    })
  }
  #[getter(choice_inner_val)]
  fn choice_get_inner_val(&self) -> Option<&Py<Inner>> {
    match self.choice.get() {
      OO_AllTypes_Choice::InnerVal(v) => Some(v), _ => None
  } }
  #[setter(choice_inner_val)]
  fn choice_set_inner_val(&mut self, v:Py<Inner>) -> PyResult<()> {
    Python::attach(|py| {
      self.choice = Py::new(py, OO_AllTypes_Choice::InnerVal(v))?;
      Ok(())
    })
  }
  #[getter(choice_color_val)]
  fn choice_get_color_val(&self) -> Option<Color> {
    match self.choice.get() {
      OO_AllTypes_Choice::ColorVal(v) => Some(v.clone()), _ => None
  } }
  #[setter(choice_color_val)]
  fn choice_set_color_val(&mut self, v:Color) -> PyResult<()> {
    Python::attach(|py| {
      self.choice = Py::new(py, OO_AllTypes_Choice::ColorVal(v))?;
      Ok(())
    })
  }
  #[getter(choice_text)]
  fn choice_get_text(&self) -> Option<&String> {
    match self.choice.get() {
      OO_AllTypes_Choice::Text(v) => Some(v), _ => None
  } }
  #[setter(choice_text)]
  fn choice_set_text(&mut self, v:String) -> PyResult<()> {
    Python::attach(|py| {
      self.choice = Py::new(py, OO_AllTypes_Choice::Text(v))?;
      Ok(())
    })
  }
  #[getter(choice_nothing)]
  fn choice_get_nothing(&self) -> Option<&Py<Empty>> {
    match self.choice.get() {
      OO_AllTypes_Choice::Nothing(v) => Some(v), _ => None
  } }
  #[setter(choice_nothing)]
  fn choice_set_nothing(&mut self, v:Py<Empty>) -> PyResult<()> {
    Python::attach(|py| {
      self.choice = Py::new(py, OO_AllTypes_Choice::Nothing(v))?;
      Ok(())
    })
  }
}
impl BitisPyCheck for AllTypes {
  fn bitis_ok(&self, _py: Python<'_>) -> bool {
    true && self.b.bitis_ok(_py) && self.u1.bitis_ok(_py) && self.u17.bitis_ok(_py) && self.u64v.bitis_ok(_py) && self.i5.bitis_ok(_py) && self.i33.bitis_ok(_py) && self.ud.bitis_ok(_py) && self.id.bitis_ok(_py) && self.f.bitis_ok(_py) && self.d.bitis_ok(_py) && self.fp.bitis_ok(_py) && self.lat.bitis_ok(_py) && self.s.bitis_ok(_py) && self.data.bitis_ok(_py) && self.color.bitis_ok(_py) && self.inner.bitis_ok(_py) && self.empty.bitis_ok(_py) && self.opt_u.bitis_ok(_py) && self.opt_inner.bitis_ok(_py) && self.opt_empty.bitis_ok(_py) && self.rf_b.bitis_ok(_py) && self.rf_inner.bitis_ok(_py) && self.rd_u.bitis_ok(_py) && self.rd_color.bitis_ok(_py) && self.rd_inner.bitis_ok(_py) && self.choice.bitis_ok(_py)
  }
}
impl AllTypes {
  pub fn to_rust(&self) -> messages::AllTypes {
    Python::attach(|_py| {
      messages::AllTypes{ b: do_val_from(&self.b), u1: do_val_from(&self.u1), u17: do_val_from(&self.u17), u64v: do_val_from(&self.u64v), i5: do_val_from(&self.i5), i33: do_val_from(&self.i33), ud: do_val_from(&self.ud), id: do_val_from(&self.id), f: do_val_from(&self.f), d: do_val_from(&self.d), fp: do_val_from(&self.fp), lat: do_val_from(&self.lat), s: do_val_from(&self.s), data: do_val_from(&self.data), color: do_val_from(&self.color), inner: do_val_from(&self.inner), empty: do_val_from(&self.empty), opt_u: do_val_from(&self.opt_u), opt_inner: do_val_from(&self.opt_inner), opt_empty: do_val_from(&self.opt_empty), rf_b: do_val_from(&self.rf_b), rf_inner: do_val_from(&self.rf_inner), rd_u: do_val_from(&self.rd_u), rd_color: do_val_from(&self.rd_color), rd_inner: do_val_from(&self.rd_inner), choice: do_val_from(&self.choice),}
    })
  }
  pub fn from_rust_obj(_d: &messages::AllTypes) -> PyResult<Self> {
    let r = Python::attach(|_py| { Self{ b: _d.b.val_into(), u1: _d.u1.val_into(), u17: _d.u17.val_into(), u64v: _d.u64v.val_into(), i5: _d.i5.val_into(), i33: _d.i33.val_into(), ud: _d.ud.val_into(), id: _d.id.val_into(), f: _d.f.val_into(), d: _d.d.val_into(), fp: _d.fp.val_into(), lat: _d.lat.val_into(), s: _d.s.val_into(), data: _d.data.val_into(), color: _d.color.val_into(), inner: _d.inner.val_into(), empty: _d.empty.val_into(), opt_u: _d.opt_u.val_into(), opt_inner: _d.opt_inner.val_into(), opt_empty: _d.opt_empty.val_into(), rf_b: _d.rf_b.val_into(), rf_inner: _d.rf_inner.val_into(), rd_u: _d.rd_u.val_into(), rd_color: _d.rd_color.val_into(), rd_inner: _d.rd_inner.val_into(), choice: _d.choice.val_into(),} });
    Ok(r)
  }
}
impl std::fmt::Display for AllTypes {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    Python::attach(|_py| {
      let t = self.to_rust();
      write!(f, "{}", t)
    })
  }
}
impl ValFromInto<Py<AllTypes>> for messages::AllTypes {
  fn val_into(self: &Self) -> Py<AllTypes> {
    Python::attach(|_py| { Py::new(_py, AllTypes::from_rust_obj(self).unwrap()).unwrap() })
  }
  fn val_from(val: &Py<AllTypes>) -> Self {
    Python::attach(|_py| { val.borrow(_py).to_rust() })
  }
}


