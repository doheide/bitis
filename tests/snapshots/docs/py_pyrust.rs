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
  pyo3::create_exception!(docs, DecodeError, pyo3::exceptions::PyValueError,
    "Decoding failed. Attribute `kind`: UnexpectedEnd, InvalidValue, InvalidDiscriminant or TrailingData.");
  pyo3::create_exception!(docs, EncodeError, pyo3::exceptions::PyValueError,
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

/// Enum doc with special chars: <html> & "quotes" \ 'single'
/// second line
#[pyclass(eq, eq_int, hash, frozen, from_py_object)]
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum Kind {
  /// first value
  KindA,
  /// second value
  KindB,
}
impl std::fmt::Display for Kind {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      Kind::KindA => { write!(f, "Kind::KindA") }, 
      Kind::KindB => { write!(f, "Kind::KindB") }, 
    }
} }

impl BitisPyCheck for Kind { fn bitis_ok(&self, _py: Python<'_>) -> bool { true } }
impl ValFromInto<Kind> for messages::Kind {
  fn val_into(&self) -> Kind {
    match self.clone() {
      messages::Kind::KindA => Kind::KindA, 
      messages::Kind::KindB => Kind::KindB, 
    }
  }
  fn val_from(v: &Kind) -> Self {
    match v {
      Kind::KindA => messages::Kind::KindA, 
      Kind::KindB => messages::Kind::KindB, 
    }
  }
}

// *****************************************************************
// *****************************************************************
// *** Enums for oneof

#[pyclass]
#[derive(Debug)]
#[allow(dead_code)]
pub enum OO_Documented_Sel {
  /// element doc
  Flag(bool),
  /// second element doc
  Num(u16),
}

#[pyclass(eq, eq_int, hash, frozen)]
#[derive(Debug, PartialEq, Hash)]
pub enum OO_Documented_SelEnum {
  Flag,
  Num,
}

#[pymethods]
impl OO_Documented_Sel {
  #[staticmethod]
  fn new_flag(py: Python<'_>, flag: bool) -> PyResult<Py<Self>> {
    Py::new(py, Self::Flag(flag))
  }
  #[staticmethod]
  fn new_num(py: Python<'_>, num: u16) -> PyResult<Py<Self>> {
    Py::new(py, Self::Num(num))
  }
  fn __repr__(&self) -> String { format!("{}", self) }
}
impl BitisPyCheck for OO_Documented_Sel {
  fn bitis_ok(&self, py: Python<'_>) -> bool {
    match self {
      Self::Flag(v) => v.bitis_ok(py),
      Self::Num(v) => v.bitis_ok(py),
    }
  }
}
impl OO_Documented_Sel {
  pub fn to_rust(&self) -> messages::OO_Documented_Sel {
    Python::attach(|_py| {
      match self {Self::Flag(v) => messages::OO_Documented_Sel::Flag(do_val_from(v)),
Self::Num(v) => messages::OO_Documented_Sel::Num(do_val_from(v)),

      }
    })
  }
  pub fn from_rust_obj(d: &messages::OO_Documented_Sel) -> PyResult<Self> {
    let r = Python::attach(|_py| {
      match d {messages::OO_Documented_Sel::Flag(v) => Self::Flag(v.val_into()),
messages::OO_Documented_Sel::Num(v) => Self::Num(v.val_into()),

      }
    });
    Ok(r)
  }
}
impl std::fmt::Display for OO_Documented_Sel {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      OO_Documented_Sel::Flag(v) => { write!(f, "OO_Documented_Sel(Flag({}))", v) },
      OO_Documented_Sel::Num(v) => { write!(f, "OO_Documented_Sel(Num({}))", v) },
    }
  }
}

impl ValFromInto<Py<OO_Documented_Sel>> for messages::OO_Documented_Sel {
  fn val_into(self: &Self) -> Py<OO_Documented_Sel> {
    Python::attach(|_py| { Py::new(_py, OO_Documented_Sel::from_rust_obj(self).unwrap()).unwrap() })
  }
  fn val_from(val: &Py<OO_Documented_Sel>) -> Self {
    Python::attach(|_py| { val.borrow(_py).to_rust() })
  }
}


// *****************************************************************
// *****************************************************************
// *** Messages

/// Message doc
#[pyclass]
#[derive(Debug)]
pub struct Documented {
  /// attribute doc
  #[pyo3(get, set)]
  pub value: u16,
  /// multi
  /// line attribute doc
  #[pyo3(get, set)]
  pub kind: Kind,
  /// oneof doc
  pub sel: Py<OO_Documented_Sel>,
}
#[pymethods]
impl Documented {
  #[new]  #[allow(non_snake_case)]
  pub fn __new__( value: u16, kind: Kind, sel: Py<OO_Documented_Sel>,) -> Self {
    Self{ value: value.into(), kind: kind.into(), sel: sel.into(), }
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
  pub fn deserialize(data: Bound<'_, PyBytes>) -> PyResult<Self> {
    let v = try_deserialize::<messages::Documented>(data.as_bytes()).map_err(|e| decode_err(e))?;
    Self::from_rust_obj(&v.0)
  }
  /// Like deserialize, but returns (value, total_bits, total_bytes) with the consumed size.
  #[staticmethod]
  pub fn try_deserialize(data: Bound<'_, PyBytes>) -> PyResult<(Self, u64, u64)> {
    let v = try_deserialize::<messages::Documented>(data.as_bytes()).map_err(|e| decode_err(e))?;
    Ok((Self::from_rust_obj(&v.0)?, v.1.total_bits, v.1.total_bytes))
  }
  /// Like deserialize, but raises DecodeError (kind TrailingData) if data is longer than the message.
  #[staticmethod]
  pub fn try_deserialize_exact(data: Bound<'_, PyBytes>) -> PyResult<Self> {
    let v = try_deserialize_exact::<messages::Documented>(data.as_bytes()).map_err(|e| decode_err(e))?;
    Self::from_rust_obj(&v)
  }
  /// Lower bound of the encoded size in bits (spec section 4).
  #[staticmethod]
  pub fn min_bits() -> u64 { <messages::Documented as BiserdiTrait>::min_bits() }
  /// Upper bound of the encoded size in bits, None if unbounded (spec section 4).
  #[staticmethod]
  pub fn max_bits() -> Option<u64> { <messages::Documented as BiserdiTrait>::max_bits() }
  pub fn __repr__(&self) -> String {
    format!("{}", self)
  }

  #[getter]
  pub fn sel_oo(&self) -> OO_Documented_SelEnum {
    match self.sel.get() {
      OO_Documented_Sel::Flag(_) => OO_Documented_SelEnum::Flag,
      OO_Documented_Sel::Num(_) => OO_Documented_SelEnum::Num,
  } }
  #[getter(sel_flag)]
  fn sel_get_flag(&self) -> Option<&bool> {
    match self.sel.get() {
      OO_Documented_Sel::Flag(v) => Some(v), _ => None
  } }
  #[setter(sel_flag)]
  fn sel_set_flag(&mut self, v:bool) -> PyResult<()> {
    Python::attach(|py| {
      self.sel = Py::new(py, OO_Documented_Sel::Flag(v))?;
      Ok(())
    })
  }
  #[getter(sel_num)]
  fn sel_get_num(&self) -> Option<&u16> {
    match self.sel.get() {
      OO_Documented_Sel::Num(v) => Some(v), _ => None
  } }
  #[setter(sel_num)]
  fn sel_set_num(&mut self, v:u16) -> PyResult<()> {
    Python::attach(|py| {
      self.sel = Py::new(py, OO_Documented_Sel::Num(v))?;
      Ok(())
    })
  }
}
impl BitisPyCheck for Documented {
  fn bitis_ok(&self, _py: Python<'_>) -> bool {
    true && self.value.bitis_ok(_py) && self.kind.bitis_ok(_py) && self.sel.bitis_ok(_py)
  }
}
impl Documented {
  pub fn to_rust(&self) -> messages::Documented {
    Python::attach(|_py| {
      messages::Documented{ value: do_val_from(&self.value), kind: do_val_from(&self.kind), sel: do_val_from(&self.sel),}
    })
  }
  pub fn from_rust_obj(_d: &messages::Documented) -> PyResult<Self> {
    let r = Python::attach(|_py| { Self{ value: _d.value.val_into(), kind: _d.kind.val_into(), sel: _d.sel.val_into(),} });
    Ok(r)
  }
}
impl std::fmt::Display for Documented {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    Python::attach(|_py| {
      let t = self.to_rust();
      write!(f, "{}", t)
    })
  }
}
impl ValFromInto<Py<Documented>> for messages::Documented {
  fn val_into(self: &Self) -> Py<Documented> {
    Python::attach(|_py| { Py::new(_py, Documented::from_rust_obj(self).unwrap()).unwrap() })
  }
  fn val_from(val: &Py<Documented>) -> Self {
    Python::attach(|_py| { val.borrow(_py).to_rust() })
  }
}


