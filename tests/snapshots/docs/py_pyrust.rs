#![allow(dead_code, non_snake_case, nonstandard_style)]

use bitis_lib::*;

use pyo3::prelude::*;
use pyo3::exceptions::{PyException, PyValueError};
use pyo3::types::{PyBytes};

use super::messages;

// Base function
fn do_val_from<T: ValFromInto<U>, U>(v: &U) -> T {
    T::val_from(v)
}
// Attributes with a maximum length [M]: the value is accepted if its Rust value encodes without error
fn check_bounds<T: BiserdiTrait + ValFromInto<U>, U>(v: &U, name: &str) -> PyResult<()> {
    match try_serialize(&T::val_from(v)) {
        Ok(_) => Ok(()),
        Err(e) => Err(PyErr::new::<PyValueError, _>(format!("{name}: {e} (longer than its maximum length or out of range)"))),
    }
}

// *****************************************************************
// *****************************************************************
// Enums

/// Enum doc with special chars: <html> & "quotes" \ 'single'
/// second line
#[pyclass]
#[derive(Debug, Clone)]
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

#[pyclass(eq, eq_int)]
#[derive(Debug, PartialEq)]
pub enum OO_Documented_SelEnum {
  Flag,
  Num,
}

#[pymethods]
impl OO_Documented_Sel {
  #[staticmethod]
  fn new_flag(flag: bool) -> PyResult<Py<Self>> {
    Ok(Python::with_gil(|_py| { Py::new(_py, Self::Flag(flag)) })?)
  }
  #[staticmethod]
  fn new_num(num: u16) -> PyResult<Py<Self>> {
    Ok(Python::with_gil(|_py| { Py::new(_py, Self::Num(num)) })?)
  }
  fn __repr__(&self) -> String { format!("{}", self) }
}
impl OO_Documented_Sel {
  pub fn to_rust(&self) -> messages::OO_Documented_Sel {
    Python::with_gil(|_py| {
      match self {Self::Flag(v) => messages::OO_Documented_Sel::Flag(do_val_from(v)),
Self::Num(v) => messages::OO_Documented_Sel::Num(do_val_from(v)),

      }
    })
  }
  pub fn from_rust_obj(d: &messages::OO_Documented_Sel) -> PyResult<Self> {
    let r = Python::with_gil(|_py| {
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
    Python::with_gil(|_py| { Py::new(_py, OO_Documented_Sel::from_rust_obj(self).unwrap()).unwrap() })
  }
  fn val_from(val: &Py<OO_Documented_Sel>) -> Self {
    Python::with_gil(|_py| { val.borrow(_py).to_rust() })
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

  pub fn serialize(&self, py: Python) -> (PyObject, u64, u64) {
    let msg = self.to_rust();
    let r = serialize(&msg);
    (PyBytes::new(py, &r.0).into(), r.1.total_bits, r.1.total_bytes)
  }
  #[staticmethod]
  pub fn deserialize(_py: Python, data: Bound<'_, PyBytes>) -> PyResult<Self> {
    let dv: Vec<u8> = data.extract()?;
    let v = match deserialize::<messages::Documented>(&dv) {
      Some(v) => v, None => return Err(PyErr::new::<PyException, _>("Error when deserializing Documented"))
    };
    Self::from_rust_obj(&v.0)
  }
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
    Python::with_gil(|py| {
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
    Python::with_gil(|py| {
      self.sel = Py::new(py, OO_Documented_Sel::Num(v))?;
      Ok(())
    })
  }
}
impl Documented {
  pub fn to_rust(&self) -> messages::Documented {
    Python::with_gil(|_py| {
      messages::Documented{ value: do_val_from(&self.value), kind: do_val_from(&self.kind), sel: do_val_from(&self.sel),}
    })
  }
  pub fn from_rust_obj(_d: &messages::Documented) -> PyResult<Self> {
    let r = Python::with_gil(|_py| { Self{ value: _d.value.val_into(), kind: _d.kind.val_into(), sel: _d.sel.val_into(),} });
    Ok(r)
  }
}
impl std::fmt::Display for Documented {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    Python::with_gil(|_py| {
      let t = self.to_rust();
      write!(f, "{}", t)
    })
  }
}
impl ValFromInto<Py<Documented>> for messages::Documented {
  fn val_into(self: &Self) -> Py<Documented> {
    Python::with_gil(|_py| { Py::new(_py, Documented::from_rust_obj(self).unwrap()).unwrap() })
  }
  fn val_from(val: &Py<Documented>) -> Self {
    Python::with_gil(|_py| { val.borrow(_py).to_rust() })
  }
}


