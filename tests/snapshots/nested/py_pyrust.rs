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

#[pyclass]
#[derive(Debug, Clone)]
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

#[pyclass(eq, eq_int)]
#[derive(Debug, PartialEq)]
pub enum OO_Top_OuterEnum {
  Leaf,
  Inner,
}

#[pymethods]
impl OO_Top_Outer {
  #[staticmethod]
  fn new_leaf(leaf: Py<Leaf>) -> PyResult<Py<Self>> {
    Ok(Python::with_gil(|_py| { Py::new(_py, Self::Leaf(leaf)) })?)
  }
  #[staticmethod]
  fn new_inner(inner: Py<OO_Outer_Inner>) -> PyResult<Py<Self>> {
    Ok(Python::with_gil(|_py| { Py::new(_py, Self::Inner(inner)) })?)
  }
  fn __repr__(&self) -> String { format!("{}", self) }
}
impl OO_Top_Outer {
  pub fn to_rust(&self) -> messages::OO_Top_Outer {
    Python::with_gil(|_py| {
      match self {Self::Leaf(v) => messages::OO_Top_Outer::Leaf(do_val_from(v)),
Self::Inner(v) => messages::OO_Top_Outer::Inner(do_val_from(v)),

      }
    })
  }
  pub fn from_rust_obj(d: &messages::OO_Top_Outer) -> PyResult<Self> {
    let r = Python::with_gil(|_py| {
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
        Python::with_gil(|py| {
          write!(f, "OO_Top_Outer(Leaf({})", v.borrow(py).__repr__()) }
        )},
      OO_Top_Outer::Inner(v) => {
        Python::with_gil(|py| {
          write!(f, "OO_Top_Outer(Inner({})", v.borrow(py).__repr__()) }
        )},
    }
  }
}

impl ValFromInto<Py<OO_Top_Outer>> for messages::OO_Top_Outer {
  fn val_into(self: &Self) -> Py<OO_Top_Outer> {
    Python::with_gil(|_py| { Py::new(_py, OO_Top_Outer::from_rust_obj(self).unwrap()).unwrap() })
  }
  fn val_from(val: &Py<OO_Top_Outer>) -> Self {
    Python::with_gil(|_py| { val.borrow(_py).to_rust() })
  }
}

#[pyclass]
#[derive(Debug)]
#[allow(dead_code)]
pub enum OO_Outer_Inner {
  X(bool),
  E(E),
}

#[pyclass(eq, eq_int)]
#[derive(Debug, PartialEq)]
pub enum OO_Outer_InnerEnum {
  X,
  E,
}

#[pymethods]
impl OO_Outer_Inner {
  #[staticmethod]
  fn new_x(x: bool) -> PyResult<Py<Self>> {
    Ok(Python::with_gil(|_py| { Py::new(_py, Self::X(x)) })?)
  }
  #[staticmethod]
  fn new_e(e: E) -> PyResult<Py<Self>> {
    Ok(Python::with_gil(|_py| { Py::new(_py, Self::E(e)) })?)
  }
  fn __repr__(&self) -> String { format!("{}", self) }
}
impl OO_Outer_Inner {
  pub fn to_rust(&self) -> messages::OO_Outer_Inner {
    Python::with_gil(|_py| {
      match self {Self::X(v) => messages::OO_Outer_Inner::X(do_val_from(v)),
Self::E(v) => messages::OO_Outer_Inner::E(do_val_from(v)),

      }
    })
  }
  pub fn from_rust_obj(d: &messages::OO_Outer_Inner) -> PyResult<Self> {
    let r = Python::with_gil(|_py| {
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
    Python::with_gil(|_py| { Py::new(_py, OO_Outer_Inner::from_rust_obj(self).unwrap()).unwrap() })
  }
  fn val_from(val: &Py<OO_Outer_Inner>) -> Self {
    Python::with_gil(|_py| { val.borrow(_py).to_rust() })
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

  pub fn serialize(&self, py: Python) -> (PyObject, u64, u64) {
    let msg = self.to_rust();
    let r = serialize(&msg);
    (PyBytes::new(py, &r.0).into(), r.1.total_bits, r.1.total_bytes)
  }
  #[staticmethod]
  pub fn deserialize(_py: Python, data: Bound<'_, PyBytes>) -> PyResult<Self> {
    let dv: Vec<u8> = data.extract()?;
    let v = match deserialize::<messages::Top>(&dv) {
      Some(v) => v, None => return Err(PyErr::new::<PyException, _>("Error when deserializing Top"))
    };
    Self::from_rust_obj(&v.0)
  }
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
    Python::with_gil(|py| {
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
    Python::with_gil(|py| {
      self.outer = Py::new(py, OO_Top_Outer::Inner(v))?;
      Ok(())
    })
  }
}
impl Top {
  pub fn to_rust(&self) -> messages::Top {
    Python::with_gil(|_py| {
      messages::Top{ m: do_val_from(&self.m), outer: do_val_from(&self.outer),}
    })
  }
  pub fn from_rust_obj(_d: &messages::Top) -> PyResult<Self> {
    let r = Python::with_gil(|_py| { Self{ m: _d.m.val_into(), outer: _d.outer.val_into(),} });
    Ok(r)
  }
}
impl std::fmt::Display for Top {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    Python::with_gil(|_py| {
      let t = self.to_rust();
      write!(f, "{}", t)
    })
  }
}
impl ValFromInto<Py<Top>> for messages::Top {
  fn val_into(self: &Self) -> Py<Top> {
    Python::with_gil(|_py| { Py::new(_py, Top::from_rust_obj(self).unwrap()).unwrap() })
  }
  fn val_from(val: &Py<Top>) -> Self {
    Python::with_gil(|_py| { val.borrow(_py).to_rust() })
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

  pub fn serialize(&self, py: Python) -> (PyObject, u64, u64) {
    let msg = self.to_rust();
    let r = serialize(&msg);
    (PyBytes::new(py, &r.0).into(), r.1.total_bits, r.1.total_bytes)
  }
  #[staticmethod]
  pub fn deserialize(_py: Python, data: Bound<'_, PyBytes>) -> PyResult<Self> {
    let dv: Vec<u8> = data.extract()?;
    let v = match deserialize::<messages::Middle>(&dv) {
      Some(v) => v, None => return Err(PyErr::new::<PyException, _>("Error when deserializing Middle"))
    };
    Self::from_rust_obj(&v.0)
  }
  pub fn __repr__(&self) -> String {
    format!("{}", self)
  }

}
impl Middle {
  pub fn to_rust(&self) -> messages::Middle {
    Python::with_gil(|_py| {
      messages::Middle{ leaves: do_val_from(&self.leaves), e: do_val_from(&self.e),}
    })
  }
  pub fn from_rust_obj(_d: &messages::Middle) -> PyResult<Self> {
    let r = Python::with_gil(|_py| { Self{ leaves: _d.leaves.val_into(), e: _d.e.val_into(),} });
    Ok(r)
  }
}
impl std::fmt::Display for Middle {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    Python::with_gil(|_py| {
      let t = self.to_rust();
      write!(f, "{}", t)
    })
  }
}
impl ValFromInto<Py<Middle>> for messages::Middle {
  fn val_into(self: &Self) -> Py<Middle> {
    Python::with_gil(|_py| { Py::new(_py, Middle::from_rust_obj(self).unwrap()).unwrap() })
  }
  fn val_from(val: &Py<Middle>) -> Self {
    Python::with_gil(|_py| { val.borrow(_py).to_rust() })
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

  pub fn serialize(&self, py: Python) -> (PyObject, u64, u64) {
    let msg = self.to_rust();
    let r = serialize(&msg);
    (PyBytes::new(py, &r.0).into(), r.1.total_bits, r.1.total_bytes)
  }
  #[staticmethod]
  pub fn deserialize(_py: Python, data: Bound<'_, PyBytes>) -> PyResult<Self> {
    let dv: Vec<u8> = data.extract()?;
    let v = match deserialize::<messages::Leaf>(&dv) {
      Some(v) => v, None => return Err(PyErr::new::<PyException, _>("Error when deserializing Leaf"))
    };
    Self::from_rust_obj(&v.0)
  }
  pub fn __repr__(&self) -> String {
    format!("{}", self)
  }

}
impl Leaf {
  pub fn to_rust(&self) -> messages::Leaf {
    Python::with_gil(|_py| {
      messages::Leaf{ v: do_val_from(&self.v),}
    })
  }
  pub fn from_rust_obj(_d: &messages::Leaf) -> PyResult<Self> {
    let r = Python::with_gil(|_py| { Self{ v: _d.v.val_into(),} });
    Ok(r)
  }
}
impl std::fmt::Display for Leaf {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    Python::with_gil(|_py| {
      let t = self.to_rust();
      write!(f, "{}", t)
    })
  }
}
impl ValFromInto<Py<Leaf>> for messages::Leaf {
  fn val_into(self: &Self) -> Py<Leaf> {
    Python::with_gil(|_py| { Py::new(_py, Leaf::from_rust_obj(self).unwrap()).unwrap() })
  }
  fn val_from(val: &Py<Leaf>) -> Self {
    Python::with_gil(|_py| { val.borrow(_py).to_rust() })
  }
}


