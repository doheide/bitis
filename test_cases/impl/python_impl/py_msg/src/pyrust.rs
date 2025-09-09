#![allow(dead_code, non_snake_case, nonstandard_style)]

use bitis_lib::*;

use pyo3::prelude::*;
use pyo3::exceptions::PyException;
use pyo3::types::{PyBytes};

use super::messages;

// Base function
fn do_val_from<T: ValFromInto<U>, U>(v: &U) -> T {
    T::val_from(v)
}

// *****************************************************************
// *****************************************************************
// Enums


// *****************************************************************
// *****************************************************************
// *** Enums for oneof

#[pyclass]
#[derive(Debug)]
#[allow(dead_code)]
pub enum OO_MsgOoSimpleBase_Value {
  Int(u16),
  Number(f64),
  TrueFalse(bool),
}

#[pyclass(eq, eq_int)]
#[derive(Debug, PartialEq)]
pub enum OO_MsgOoSimpleBase_ValueEnum {
  Int,
  Number,
  TrueFalse,
}

#[pymethods]
impl OO_MsgOoSimpleBase_Value {
  #[staticmethod]
  fn new_int(int: u16) -> PyResult<Py<Self>> {
    Ok(Python::with_gil(|_py| { Py::new(_py, Self::Int(int)) })?)
  }
  #[staticmethod]
  fn new_number(number: f64) -> PyResult<Py<Self>> {
    Ok(Python::with_gil(|_py| { Py::new(_py, Self::Number(number)) })?)
  }
  #[staticmethod]
  fn new_true_false(true_false: bool) -> PyResult<Py<Self>> {
    Ok(Python::with_gil(|_py| { Py::new(_py, Self::TrueFalse(true_false)) })?)
  }
  fn __repr__(&self) -> String { format!("{}", self) }
}
impl OO_MsgOoSimpleBase_Value {
  pub fn to_rust(&self) -> messages::OO_MsgOoSimpleBase_Value {
    Python::with_gil(|_py| {
      match self {Self::Int(v) => messages::OO_MsgOoSimpleBase_Value::Int(do_val_from(v)),
Self::Number(v) => messages::OO_MsgOoSimpleBase_Value::Number(do_val_from(v)),
Self::TrueFalse(v) => messages::OO_MsgOoSimpleBase_Value::TrueFalse(do_val_from(v)),

      }
    })
  }
  pub fn from_rust_obj(d: &messages::OO_MsgOoSimpleBase_Value) -> PyResult<Self> {
    let r = Python::with_gil(|_py| {
      match d {messages::OO_MsgOoSimpleBase_Value::Int(v) => Self::Int(v.val_into()),
messages::OO_MsgOoSimpleBase_Value::Number(v) => Self::Number(v.val_into()),
messages::OO_MsgOoSimpleBase_Value::TrueFalse(v) => Self::TrueFalse(v.val_into()),

      }
    });
    Ok(r)
  }
}
impl std::fmt::Display for OO_MsgOoSimpleBase_Value {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      OO_MsgOoSimpleBase_Value::Int(v) => { write!(f, "OO_MsgOoSimpleBase_Value(Int({}))", v) },
      OO_MsgOoSimpleBase_Value::Number(v) => { write!(f, "OO_MsgOoSimpleBase_Value(Number({}))", v) },
      OO_MsgOoSimpleBase_Value::TrueFalse(v) => { write!(f, "OO_MsgOoSimpleBase_Value(TrueFalse({}))", v) },
    }
  }
}

impl ValFromInto<Py<OO_MsgOoSimpleBase_Value>> for messages::OO_MsgOoSimpleBase_Value {
  fn val_into(self: &Self) -> Py<OO_MsgOoSimpleBase_Value> {
    Python::with_gil(|_py| { Py::new(_py, OO_MsgOoSimpleBase_Value::from_rust_obj(self).unwrap()).unwrap() })
  }
  fn val_from(val: &Py<OO_MsgOoSimpleBase_Value>) -> Self {
    Python::with_gil(|_py| { val.borrow(_py).to_rust() })
  }
}

#[pyclass]
#[derive(Debug)]
#[allow(dead_code)]
pub enum OO_MsgOoNestedBase_Value {
  Inner(Py<MsgSimpleBaseOneInt>),
  Number(f64),
  TrueFalse(bool),
}

#[pyclass(eq, eq_int)]
#[derive(Debug, PartialEq)]
pub enum OO_MsgOoNestedBase_ValueEnum {
  Inner,
  Number,
  TrueFalse,
}

#[pymethods]
impl OO_MsgOoNestedBase_Value {
  #[staticmethod]
  fn new_inner(inner: Py<MsgSimpleBaseOneInt>) -> PyResult<Py<Self>> {
    Ok(Python::with_gil(|_py| { Py::new(_py, Self::Inner(inner)) })?)
  }
  #[staticmethod]
  fn new_number(number: f64) -> PyResult<Py<Self>> {
    Ok(Python::with_gil(|_py| { Py::new(_py, Self::Number(number)) })?)
  }
  #[staticmethod]
  fn new_true_false(true_false: bool) -> PyResult<Py<Self>> {
    Ok(Python::with_gil(|_py| { Py::new(_py, Self::TrueFalse(true_false)) })?)
  }
  fn __repr__(&self) -> String { format!("{}", self) }
}
impl OO_MsgOoNestedBase_Value {
  pub fn to_rust(&self) -> messages::OO_MsgOoNestedBase_Value {
    Python::with_gil(|_py| {
      match self {Self::Inner(v) => messages::OO_MsgOoNestedBase_Value::Inner(do_val_from(v)),
Self::Number(v) => messages::OO_MsgOoNestedBase_Value::Number(do_val_from(v)),
Self::TrueFalse(v) => messages::OO_MsgOoNestedBase_Value::TrueFalse(do_val_from(v)),

      }
    })
  }
  pub fn from_rust_obj(d: &messages::OO_MsgOoNestedBase_Value) -> PyResult<Self> {
    let r = Python::with_gil(|_py| {
      match d {messages::OO_MsgOoNestedBase_Value::Inner(v) => Self::Inner(v.val_into()),
messages::OO_MsgOoNestedBase_Value::Number(v) => Self::Number(v.val_into()),
messages::OO_MsgOoNestedBase_Value::TrueFalse(v) => Self::TrueFalse(v.val_into()),

      }
    });
    Ok(r)
  }
}
impl std::fmt::Display for OO_MsgOoNestedBase_Value {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      OO_MsgOoNestedBase_Value::Inner(v) => {
        Python::with_gil(|py| {
          write!(f, "OO_MsgOoNestedBase_Value(Inner({})", v.borrow(py).__repr__()) }
        )},
      OO_MsgOoNestedBase_Value::Number(v) => { write!(f, "OO_MsgOoNestedBase_Value(Number({}))", v) },
      OO_MsgOoNestedBase_Value::TrueFalse(v) => { write!(f, "OO_MsgOoNestedBase_Value(TrueFalse({}))", v) },
    }
  }
}

impl ValFromInto<Py<OO_MsgOoNestedBase_Value>> for messages::OO_MsgOoNestedBase_Value {
  fn val_into(self: &Self) -> Py<OO_MsgOoNestedBase_Value> {
    Python::with_gil(|_py| { Py::new(_py, OO_MsgOoNestedBase_Value::from_rust_obj(self).unwrap()).unwrap() })
  }
  fn val_from(val: &Py<OO_MsgOoNestedBase_Value>) -> Self {
    Python::with_gil(|_py| { val.borrow(_py).to_rust() })
  }
}


// *****************************************************************
// *****************************************************************
// *** Messages

#[pyclass]
#[derive(Debug)]
pub struct MsgOOSimpleBase {
// lala
  #[pyo3(get, set)]
  pub id: u16,
  pub value: Py<OO_MsgOoSimpleBase_Value>,
}
#[pymethods]
impl MsgOOSimpleBase {
  #[new]  #[allow(non_snake_case)]
  pub fn __new__( id: u16, value: Py<OO_MsgOoSimpleBase_Value>,) -> Self {
    Self{ id: id.into(), value: value.into(), }
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
    let v = match deserialize::<messages::MsgOOSimpleBase>(&dv) {
      Some(v) => v, None => return Err(PyErr::new::<PyException, _>("Error when deserializing MsgOOSimpleBase"))
    };
    Self::from_rust_obj(&v.0)
  }
  pub fn __repr__(&self) -> String {
    format!("{}", self)
  }

  #[getter]
  pub fn value_oo(&self) -> OO_MsgOoSimpleBase_ValueEnum {
    match self.value.get() {
      OO_MsgOoSimpleBase_Value::Int(_) => OO_MsgOoSimpleBase_ValueEnum::Int,
      OO_MsgOoSimpleBase_Value::Number(_) => OO_MsgOoSimpleBase_ValueEnum::Number,
      OO_MsgOoSimpleBase_Value::TrueFalse(_) => OO_MsgOoSimpleBase_ValueEnum::TrueFalse,
  } }
  #[getter(value_int)]
  fn value_get_int(&self) -> Option<&u16> {
    match self.value.get() {
      OO_MsgOoSimpleBase_Value::Int(v) => Some(v), _ => None
  } }
  #[setter(value_int)]
  fn value_set_int(&mut self, v:u16) -> PyResult<()> {
    Python::with_gil(|py| {
      self.value = Py::new(py, OO_MsgOoSimpleBase_Value::Int(v))?;
      Ok(())
    })
  }
  #[getter(value_number)]
  fn value_get_number(&self) -> Option<&f64> {
    match self.value.get() {
      OO_MsgOoSimpleBase_Value::Number(v) => Some(v), _ => None
  } }
  #[setter(value_number)]
  fn value_set_number(&mut self, v:f64) -> PyResult<()> {
    Python::with_gil(|py| {
      self.value = Py::new(py, OO_MsgOoSimpleBase_Value::Number(v))?;
      Ok(())
    })
  }
  #[getter(value_true_false)]
  fn value_get_true_false(&self) -> Option<&bool> {
    match self.value.get() {
      OO_MsgOoSimpleBase_Value::TrueFalse(v) => Some(v), _ => None
  } }
  #[setter(value_true_false)]
  fn value_set_true_false(&mut self, v:bool) -> PyResult<()> {
    Python::with_gil(|py| {
      self.value = Py::new(py, OO_MsgOoSimpleBase_Value::TrueFalse(v))?;
      Ok(())
    })
  }
}
impl MsgOOSimpleBase {
  pub fn to_rust(&self) -> messages::MsgOOSimpleBase {
    Python::with_gil(|_py| {
      messages::MsgOOSimpleBase{ id: do_val_from(&self.id), value: do_val_from(&self.value),}
    })
  }
  pub fn from_rust_obj(_d: &messages::MsgOOSimpleBase) -> PyResult<Self> {
    let r = Python::with_gil(|_py| { Self{ id: _d.id.val_into(), value: _d.value.val_into(),} });
    Ok(r)
  }
}
impl std::fmt::Display for MsgOOSimpleBase {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    Python::with_gil(|_py| {
      let t = self.to_rust();
      write!(f, "{}", t)
    })
  }
}
impl ValFromInto<Py<MsgOOSimpleBase>> for messages::MsgOOSimpleBase {
  fn val_into(self: &Self) -> Py<MsgOOSimpleBase> {
    Python::with_gil(|_py| { Py::new(_py, MsgOOSimpleBase::from_rust_obj(self).unwrap()).unwrap() })
  }
  fn val_from(val: &Py<MsgOOSimpleBase>) -> Self {
    Python::with_gil(|_py| { val.borrow(_py).to_rust() })
  }
}


#[pyclass]
#[derive(Debug)]
pub struct MsgSimpleBaseOneInt {
// lala
  #[pyo3(get, set)]
  pub param_1: u16,
}
#[pymethods]
impl MsgSimpleBaseOneInt {
  #[new]  #[allow(non_snake_case)]
  pub fn __new__( param_1: u16,) -> Self {
    Self{ param_1: param_1.into(), }
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
    let v = match deserialize::<messages::MsgSimpleBaseOneInt>(&dv) {
      Some(v) => v, None => return Err(PyErr::new::<PyException, _>("Error when deserializing MsgSimpleBaseOneInt"))
    };
    Self::from_rust_obj(&v.0)
  }
  pub fn __repr__(&self) -> String {
    format!("{}", self)
  }

}
impl MsgSimpleBaseOneInt {
  pub fn to_rust(&self) -> messages::MsgSimpleBaseOneInt {
    Python::with_gil(|_py| {
      messages::MsgSimpleBaseOneInt{ param_1: do_val_from(&self.param_1),}
    })
  }
  pub fn from_rust_obj(_d: &messages::MsgSimpleBaseOneInt) -> PyResult<Self> {
    let r = Python::with_gil(|_py| { Self{ param_1: _d.param_1.val_into(),} });
    Ok(r)
  }
}
impl std::fmt::Display for MsgSimpleBaseOneInt {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    Python::with_gil(|_py| {
      let t = self.to_rust();
      write!(f, "{}", t)
    })
  }
}
impl ValFromInto<Py<MsgSimpleBaseOneInt>> for messages::MsgSimpleBaseOneInt {
  fn val_into(self: &Self) -> Py<MsgSimpleBaseOneInt> {
    Python::with_gil(|_py| { Py::new(_py, MsgSimpleBaseOneInt::from_rust_obj(self).unwrap()).unwrap() })
  }
  fn val_from(val: &Py<MsgSimpleBaseOneInt>) -> Self {
    Python::with_gil(|_py| { val.borrow(_py).to_rust() })
  }
}


#[pyclass]
#[derive(Debug)]
pub struct MsgOONestedBase {
// lala
  #[pyo3(get, set)]
  pub id: u16,
  pub value: Py<OO_MsgOoNestedBase_Value>,
}
#[pymethods]
impl MsgOONestedBase {
  #[new]  #[allow(non_snake_case)]
  pub fn __new__( id: u16, value: Py<OO_MsgOoNestedBase_Value>,) -> Self {
    Self{ id: id.into(), value: value.into(), }
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
    let v = match deserialize::<messages::MsgOONestedBase>(&dv) {
      Some(v) => v, None => return Err(PyErr::new::<PyException, _>("Error when deserializing MsgOONestedBase"))
    };
    Self::from_rust_obj(&v.0)
  }
  pub fn __repr__(&self) -> String {
    format!("{}", self)
  }

  #[getter]
  pub fn value_oo(&self) -> OO_MsgOoNestedBase_ValueEnum {
    match self.value.get() {
      OO_MsgOoNestedBase_Value::Inner(_) => OO_MsgOoNestedBase_ValueEnum::Inner,
      OO_MsgOoNestedBase_Value::Number(_) => OO_MsgOoNestedBase_ValueEnum::Number,
      OO_MsgOoNestedBase_Value::TrueFalse(_) => OO_MsgOoNestedBase_ValueEnum::TrueFalse,
  } }
  #[getter(value_inner)]
  fn value_get_inner(&self) -> Option<&Py<MsgSimpleBaseOneInt>> {
    match self.value.get() {
      OO_MsgOoNestedBase_Value::Inner(v) => Some(v), _ => None
  } }
  #[setter(value_inner)]
  fn value_set_inner(&mut self, v:Py<MsgSimpleBaseOneInt>) -> PyResult<()> {
    Python::with_gil(|py| {
      self.value = Py::new(py, OO_MsgOoNestedBase_Value::Inner(v))?;
      Ok(())
    })
  }
  #[getter(value_number)]
  fn value_get_number(&self) -> Option<&f64> {
    match self.value.get() {
      OO_MsgOoNestedBase_Value::Number(v) => Some(v), _ => None
  } }
  #[setter(value_number)]
  fn value_set_number(&mut self, v:f64) -> PyResult<()> {
    Python::with_gil(|py| {
      self.value = Py::new(py, OO_MsgOoNestedBase_Value::Number(v))?;
      Ok(())
    })
  }
  #[getter(value_true_false)]
  fn value_get_true_false(&self) -> Option<&bool> {
    match self.value.get() {
      OO_MsgOoNestedBase_Value::TrueFalse(v) => Some(v), _ => None
  } }
  #[setter(value_true_false)]
  fn value_set_true_false(&mut self, v:bool) -> PyResult<()> {
    Python::with_gil(|py| {
      self.value = Py::new(py, OO_MsgOoNestedBase_Value::TrueFalse(v))?;
      Ok(())
    })
  }
}
impl MsgOONestedBase {
  pub fn to_rust(&self) -> messages::MsgOONestedBase {
    Python::with_gil(|_py| {
      messages::MsgOONestedBase{ id: do_val_from(&self.id), value: do_val_from(&self.value),}
    })
  }
  pub fn from_rust_obj(_d: &messages::MsgOONestedBase) -> PyResult<Self> {
    let r = Python::with_gil(|_py| { Self{ id: _d.id.val_into(), value: _d.value.val_into(),} });
    Ok(r)
  }
}
impl std::fmt::Display for MsgOONestedBase {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    Python::with_gil(|_py| {
      let t = self.to_rust();
      write!(f, "{}", t)
    })
  }
}
impl ValFromInto<Py<MsgOONestedBase>> for messages::MsgOONestedBase {
  fn val_into(self: &Self) -> Py<MsgOONestedBase> {
    Python::with_gil(|_py| { Py::new(_py, MsgOONestedBase::from_rust_obj(self).unwrap()).unwrap() })
  }
  fn val_from(val: &Py<MsgOONestedBase>) -> Self {
    Python::with_gil(|_py| { val.borrow(_py).to_rust() })
  }
}


