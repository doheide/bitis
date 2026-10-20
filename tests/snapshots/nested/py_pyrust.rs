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

// *****************************************************************
// *****************************************************************
// Exceptions (docs/WIRE_FORMAT.md section 3); `kind` is the spec error name
pub mod bitis_py_errors {
  pyo3::create_exception!(nested, DecodeError, pyo3::exceptions::PyValueError,
    "Decoding failed. Attribute `kind`: UnexpectedEnd, InvalidValue, InvalidDiscriminant or TrailingData.");
  pyo3::create_exception!(nested, EncodeError, pyo3::exceptions::PyValueError,
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
pub enum E {
  A,
  B,
}
impl std::fmt::Display for E {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      E::A => { write!(f, "E::A") }, 
      E::B => { write!(f, "E::B") }, 
    }
} }

impl BitisPyCheck for E { fn bitis_ok(&self, _py: Python<'_>) -> bool { true } }
impl ValFromInto<E> for messages::E {
  fn val_into(&self) -> E {
    match self.clone() {
      messages::E::A => E::A, 
      messages::E::B => E::B, 
    }
  }
  fn val_from(v: &E) -> Self {
    match v {
      E::A => messages::E::A, 
      E::B => messages::E::B, 
    }
  }
}

// *****************************************************************
// *****************************************************************
// *** Enums for oneof

#[pyclass]
#[derive(Debug)]
#[allow(dead_code)]
pub enum OO_Top_Outer {
  Leaf(Py<Leaf>),
  Inner(Py<OO_Outer_Inner>),
}

#[pyclass(eq, eq_int, hash, frozen)]
#[derive(Debug, PartialEq, Hash)]
pub enum OO_Top_OuterEnum {
  Leaf,
  Inner,
}

#[pymethods]
impl OO_Top_Outer {
  #[staticmethod]
  fn new_leaf(py: Python<'_>, leaf: Py<Leaf>) -> PyResult<Py<Self>> {
    Py::new(py, Self::Leaf(leaf))
  }
  #[staticmethod]
  fn new_inner(py: Python<'_>, inner: Py<OO_Outer_Inner>) -> PyResult<Py<Self>> {
    Py::new(py, Self::Inner(inner))
  }
  fn __repr__(&self) -> String { format!("{}", self) }
}
impl BitisPyCheck for OO_Top_Outer {
  fn bitis_ok(&self, py: Python<'_>) -> bool {
    match self {
      Self::Leaf(v) => v.bitis_ok(py),
      Self::Inner(v) => v.bitis_ok(py),
    }
  }
}
impl OO_Top_Outer {
  pub fn to_rust(&self) -> messages::OO_Top_Outer {
    Python::attach(|_py| {
      match self {Self::Leaf(v) => messages::OO_Top_Outer::Leaf(do_val_from(v)),
Self::Inner(v) => messages::OO_Top_Outer::Inner(do_val_from(v)),

      }
    })
  }
  pub fn from_rust_obj(d: &messages::OO_Top_Outer) -> PyResult<Self> {
    let r = Python::attach(|_py| {
      match d {messages::OO_Top_Outer::Leaf(v) => Self::Leaf(v.val_into()),
messages::OO_Top_Outer::Inner(v) => Self::Inner(v.val_into()),

      }
    });
    Ok(r)
  }
}
impl std::fmt::Display for OO_Top_Outer {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      OO_Top_Outer::Leaf(v) => {
        Python::attach(|py| {
          write!(f, "OO_Top_Outer(Leaf({})", v.borrow(py).__repr__()) }
        )},
      OO_Top_Outer::Inner(v) => {
        Python::attach(|py| {
          write!(f, "OO_Top_Outer(Inner({})", v.borrow(py).__repr__()) }
        )},
    }
  }
}

impl ValFromInto<Py<OO_Top_Outer>> for messages::OO_Top_Outer {
  fn val_into(self: &Self) -> Py<OO_Top_Outer> {
    Python::attach(|_py| { Py::new(_py, OO_Top_Outer::from_rust_obj(self).unwrap()).unwrap() })
  }
  fn val_from(val: &Py<OO_Top_Outer>) -> Self {
    Python::attach(|_py| { val.borrow(_py).to_rust() })
  }
}

#[pyclass]
#[derive(Debug)]
#[allow(dead_code)]
pub enum OO_Outer_Inner {
  X(bool),
  E(E),
}

#[pyclass(eq, eq_int, hash, frozen)]
#[derive(Debug, PartialEq, Hash)]
pub enum OO_Outer_InnerEnum {
  X,
  E,
}

#[pymethods]
impl OO_Outer_Inner {
  #[staticmethod]
  fn new_x(py: Python<'_>, x: bool) -> PyResult<Py<Self>> {
    Py::new(py, Self::X(x))
  }
  #[staticmethod]
  fn new_e(py: Python<'_>, e: E) -> PyResult<Py<Self>> {
    Py::new(py, Self::E(e))
  }
  fn __repr__(&self) -> String { format!("{}", self) }
}
impl BitisPyCheck for OO_Outer_Inner {
  fn bitis_ok(&self, py: Python<'_>) -> bool {
    match self {
      Self::X(v) => v.bitis_ok(py),
      Self::E(v) => v.bitis_ok(py),
    }
  }
}
impl OO_Outer_Inner {
  pub fn to_rust(&self) -> messages::OO_Outer_Inner {
    Python::attach(|_py| {
      match self {Self::X(v) => messages::OO_Outer_Inner::X(do_val_from(v)),
Self::E(v) => messages::OO_Outer_Inner::E(do_val_from(v)),

      }
    })
  }
  pub fn from_rust_obj(d: &messages::OO_Outer_Inner) -> PyResult<Self> {
    let r = Python::attach(|_py| {
      match d {messages::OO_Outer_Inner::X(v) => Self::X(v.val_into()),
messages::OO_Outer_Inner::E(v) => Self::E(v.val_into()),

      }
    });
    Ok(r)
  }
}
impl std::fmt::Display for OO_Outer_Inner {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      OO_Outer_Inner::X(v) => { write!(f, "OO_Outer_Inner(X({}))", v) },
      OO_Outer_Inner::E(v) => { write!(f, "OO_Outer_Inner(E({}))", v) },
    }
  }
}

impl ValFromInto<Py<OO_Outer_Inner>> for messages::OO_Outer_Inner {
  fn val_into(self: &Self) -> Py<OO_Outer_Inner> {
    Python::attach(|_py| { Py::new(_py, OO_Outer_Inner::from_rust_obj(self).unwrap()).unwrap() })
  }
  fn val_from(val: &Py<OO_Outer_Inner>) -> Self {
    Python::attach(|_py| { val.borrow(_py).to_rust() })
  }
}


// *****************************************************************
// *****************************************************************
// *** Messages

#[pyclass]
#[derive(Debug)]
pub struct Top {
  #[pyo3(get, set)]
  pub m: Py<Middle>,
  pub outer: Py<OO_Top_Outer>,
}
#[pymethods]
impl Top {
  #[new]  #[allow(non_snake_case)]
  pub fn __new__( m: Py<Middle>, outer: Py<OO_Top_Outer>,) -> Self {
    Self{ m: m.into(), outer: outer.into(), }
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
    let v = try_deserialize::<messages::Top>(data.as_bytes()).map_err(|e| decode_err(e))?;
    Self::from_rust_obj(&v.0)
  }
  /// Like deserialize, but returns (value, total_bits, total_bytes) with the consumed size.
  #[staticmethod]
  pub fn try_deserialize(py: Python<'_>, data: Bound<'_, PyBytes>) -> PyResult<(Self, u64, u64)> {
    let v = try_deserialize::<messages::Top>(data.as_bytes()).map_err(|e| decode_err(e))?;
    Ok((Self::from_rust_obj(&v.0)?, v.1.total_bits, v.1.total_bytes))
  }
  /// Like deserialize, but raises DecodeError (kind TrailingData) if data is longer than the message.
  #[staticmethod]
  pub fn try_deserialize_exact(py: Python<'_>, data: Bound<'_, PyBytes>) -> PyResult<Self> {
    let v = try_deserialize_exact::<messages::Top>(data.as_bytes()).map_err(|e| decode_err(e))?;
    Self::from_rust_obj(&v)
  }
  /// Lower bound of the encoded size in bits (spec section 4).
  #[staticmethod]
  pub fn min_bits() -> u64 { <messages::Top as BiserdiTrait>::min_bits() }
  /// Upper bound of the encoded size in bits, None if unbounded (spec section 4).
  #[staticmethod]
  pub fn max_bits() -> Option<u64> { <messages::Top as BiserdiTrait>::max_bits() }
  pub fn __repr__(&self) -> String {
    format!("{}", self)
  }

  #[getter]
  pub fn outer_oo(&self) -> OO_Top_OuterEnum {
    match self.outer.get() {
      OO_Top_Outer::Leaf(_) => OO_Top_OuterEnum::Leaf,
      OO_Top_Outer::Inner(_) => OO_Top_OuterEnum::Inner,
  } }
  #[getter(outer_leaf)]
  fn outer_get_leaf(&self) -> Option<&Py<Leaf>> {
    match self.outer.get() {
      OO_Top_Outer::Leaf(v) => Some(v), _ => None
  } }
  #[setter(outer_leaf)]
  fn outer_set_leaf(&mut self, v:Py<Leaf>) -> PyResult<()> {
    Python::attach(|py| {
      self.outer = Py::new(py, OO_Top_Outer::Leaf(v))?;
      Ok(())
    })
  }
  #[getter(outer_inner)]
  fn outer_get_inner(&self) -> Option<&Py<OO_Outer_Inner>> {
    match self.outer.get() {
      OO_Top_Outer::Inner(v) => Some(v), _ => None
  } }
  #[setter(outer_inner)]
  fn outer_set_inner(&mut self, v:Py<OO_Outer_Inner>) -> PyResult<()> {
    Python::attach(|py| {
      self.outer = Py::new(py, OO_Top_Outer::Inner(v))?;
      Ok(())
    })
  }
}
impl BitisPyCheck for Top {
  fn bitis_ok(&self, _py: Python<'_>) -> bool {
    true && self.m.bitis_ok(_py) && self.outer.bitis_ok(_py)
  }
}
impl Top {
  pub fn to_rust(&self) -> messages::Top {
    Python::attach(|_py| {
      messages::Top{ m: do_val_from(&self.m), outer: do_val_from(&self.outer),}
    })
  }
  pub fn from_rust_obj(_d: &messages::Top) -> PyResult<Self> {
    let r = Python::attach(|_py| { Self{ m: _d.m.val_into(), outer: _d.outer.val_into(),} });
    Ok(r)
  }
}
impl std::fmt::Display for Top {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    Python::attach(|_py| {
      let t = self.to_rust();
      write!(f, "{}", t)
    })
  }
}
impl ValFromInto<Py<Top>> for messages::Top {
  fn val_into(self: &Self) -> Py<Top> {
    Python::attach(|_py| { Py::new(_py, Top::from_rust_obj(self).unwrap()).unwrap() })
  }
  fn val_from(val: &Py<Top>) -> Self {
    Python::attach(|_py| { val.borrow(_py).to_rust() })
  }
}


#[pyclass]
#[derive(Debug)]
pub struct Middle {
  #[pyo3(get, set)]
  pub leaves: Vec<Py<Leaf>>,
  #[pyo3(get, set)]
  pub e: Option<E>,
}
#[pymethods]
impl Middle {
  #[new]  #[allow(non_snake_case)]
  pub fn __new__( leaves: Vec<Py<Leaf>>, e: Option<E>,) -> Self {
    Self{ leaves: leaves.into(), e: e.into(), }
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
    let v = try_deserialize::<messages::Middle>(data.as_bytes()).map_err(|e| decode_err(e))?;
    Self::from_rust_obj(&v.0)
  }
  /// Like deserialize, but returns (value, total_bits, total_bytes) with the consumed size.
  #[staticmethod]
  pub fn try_deserialize(py: Python<'_>, data: Bound<'_, PyBytes>) -> PyResult<(Self, u64, u64)> {
    let v = try_deserialize::<messages::Middle>(data.as_bytes()).map_err(|e| decode_err(e))?;
    Ok((Self::from_rust_obj(&v.0)?, v.1.total_bits, v.1.total_bytes))
  }
  /// Like deserialize, but raises DecodeError (kind TrailingData) if data is longer than the message.
  #[staticmethod]
  pub fn try_deserialize_exact(py: Python<'_>, data: Bound<'_, PyBytes>) -> PyResult<Self> {
    let v = try_deserialize_exact::<messages::Middle>(data.as_bytes()).map_err(|e| decode_err(e))?;
    Self::from_rust_obj(&v)
  }
  /// Lower bound of the encoded size in bits (spec section 4).
  #[staticmethod]
  pub fn min_bits() -> u64 { <messages::Middle as BiserdiTrait>::min_bits() }
  /// Upper bound of the encoded size in bits, None if unbounded (spec section 4).
  #[staticmethod]
  pub fn max_bits() -> Option<u64> { <messages::Middle as BiserdiTrait>::max_bits() }
  pub fn __repr__(&self) -> String {
    format!("{}", self)
  }

}
impl BitisPyCheck for Middle {
  fn bitis_ok(&self, _py: Python<'_>) -> bool {
    true && self.leaves.bitis_ok(_py) && self.e.bitis_ok(_py)
  }
}
impl Middle {
  pub fn to_rust(&self) -> messages::Middle {
    Python::attach(|_py| {
      messages::Middle{ leaves: do_val_from(&self.leaves), e: do_val_from(&self.e),}
    })
  }
  pub fn from_rust_obj(_d: &messages::Middle) -> PyResult<Self> {
    let r = Python::attach(|_py| { Self{ leaves: _d.leaves.val_into(), e: _d.e.val_into(),} });
    Ok(r)
  }
}
impl std::fmt::Display for Middle {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    Python::attach(|_py| {
      let t = self.to_rust();
      write!(f, "{}", t)
    })
  }
}
impl ValFromInto<Py<Middle>> for messages::Middle {
  fn val_into(self: &Self) -> Py<Middle> {
    Python::attach(|_py| { Py::new(_py, Middle::from_rust_obj(self).unwrap()).unwrap() })
  }
  fn val_from(val: &Py<Middle>) -> Self {
    Python::attach(|_py| { val.borrow(_py).to_rust() })
  }
}


#[pyclass]
#[derive(Debug)]
pub struct Leaf {
  #[pyo3(get, set)]
  pub v: u16,
}
#[pymethods]
impl Leaf {
  #[new]  #[allow(non_snake_case)]
  pub fn __new__( v: u16,) -> Self {
    Self{ v: v.into(), }
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
    let v = try_deserialize::<messages::Leaf>(data.as_bytes()).map_err(|e| decode_err(e))?;
    Self::from_rust_obj(&v.0)
  }
  /// Like deserialize, but returns (value, total_bits, total_bytes) with the consumed size.
  #[staticmethod]
  pub fn try_deserialize(py: Python<'_>, data: Bound<'_, PyBytes>) -> PyResult<(Self, u64, u64)> {
    let v = try_deserialize::<messages::Leaf>(data.as_bytes()).map_err(|e| decode_err(e))?;
    Ok((Self::from_rust_obj(&v.0)?, v.1.total_bits, v.1.total_bytes))
  }
  /// Like deserialize, but raises DecodeError (kind TrailingData) if data is longer than the message.
  #[staticmethod]
  pub fn try_deserialize_exact(py: Python<'_>, data: Bound<'_, PyBytes>) -> PyResult<Self> {
    let v = try_deserialize_exact::<messages::Leaf>(data.as_bytes()).map_err(|e| decode_err(e))?;
    Self::from_rust_obj(&v)
  }
  /// Lower bound of the encoded size in bits (spec section 4).
  #[staticmethod]
  pub fn min_bits() -> u64 { <messages::Leaf as BiserdiTrait>::min_bits() }
  /// Upper bound of the encoded size in bits, None if unbounded (spec section 4).
  #[staticmethod]
  pub fn max_bits() -> Option<u64> { <messages::Leaf as BiserdiTrait>::max_bits() }
  pub fn __repr__(&self) -> String {
    format!("{}", self)
  }

}
impl BitisPyCheck for Leaf {
  fn bitis_ok(&self, _py: Python<'_>) -> bool {
    true && self.v.bitis_ok(_py)
  }
}
impl Leaf {
  pub fn to_rust(&self) -> messages::Leaf {
    Python::attach(|_py| {
      messages::Leaf{ v: do_val_from(&self.v),}
    })
  }
  pub fn from_rust_obj(_d: &messages::Leaf) -> PyResult<Self> {
    let r = Python::attach(|_py| { Self{ v: _d.v.val_into(),} });
    Ok(r)
  }
}
impl std::fmt::Display for Leaf {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    Python::attach(|_py| {
      let t = self.to_rust();
      write!(f, "{}", t)
    })
  }
}
impl ValFromInto<Py<Leaf>> for messages::Leaf {
  fn val_into(self: &Self) -> Py<Leaf> {
    Python::attach(|_py| { Py::new(_py, Leaf::from_rust_obj(self).unwrap()).unwrap() })
  }
  fn val_from(val: &Py<Leaf>) -> Self {
    Python::attach(|_py| { val.borrow(_py).to_rust() })
  }
}


