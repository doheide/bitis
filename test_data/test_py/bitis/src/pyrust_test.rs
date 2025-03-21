use bitis_lib::*;

use pyo3::prelude::*;
use pyo3::exceptions::PyException;
use pyo3::types::{PyBytes};

use super::messages;




// Enums
/// Test comment for Enum
#[pyclass]
#[derive(Debug, Clone)]
#[allow(nonstandard_style)]
pub enum Numbers {
  One,
  Two,
  Three,
  Four,
}
impl std::fmt::Display for Numbers {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      Numbers::One => { write!(f, "Numbers::One") }, 
      Numbers::Two => { write!(f, "Numbers::Two") }, 
      Numbers::Three => { write!(f, "Numbers::Three") }, 
      Numbers::Four => { write!(f, "Numbers::Four") }, 
    }
} }
#[allow(nonstandard_style)]
fn Numbers_rust_to_py(v: messages::Numbers) -> Numbers {
    match v {
      messages::Numbers::One => Numbers::One, 
      messages::Numbers::Two => Numbers::Two, 
      messages::Numbers::Three => Numbers::Three, 
      messages::Numbers::Four => Numbers::Four, 
    }
}
#[allow(nonstandard_style)]
fn Numbers_py_to_rust(v: Numbers) -> messages::Numbers {
    match v {
      Numbers::One => messages::Numbers::One, 
      Numbers::Two => messages::Numbers::Two, 
      Numbers::Three => messages::Numbers::Three, 
      Numbers::Four => messages::Numbers::Four, 
    }
}



// *** Enums for oneof
#[pyclass]
#[derive(Debug)]
#[allow(nonstandard_style)]
pub enum OO_ParamTestWithInner_Action {
  Inner(Py<Inner>),
  Val(u8),
}
#[pyclass]
#[derive(Debug, PartialEq)]
#[allow(nonstandard_style)]
pub enum OO_ParamTestWithInner_ActionEnum {
  Inner,
  Val,
}
#[pymethods]
impl OO_ParamTestWithInner_Action {
  #[staticmethod]
  fn new_inner(inner: Py<Inner>) -> PyResult<Py<Self>> {
    Ok(Python::with_gil(|_py| { Py::new(_py, Self::Inner(inner)) })?)
  }
  #[staticmethod]
  fn new_val(val: u8) -> PyResult<Py<Self>> {
    Ok(Python::with_gil(|_py| { Py::new(_py, Self::Val(val)) })?)
  }
  fn __repr__(&self) -> String { format!("{}", self) }
}
impl OO_ParamTestWithInner_Action {
  pub fn to_rust(&self) -> messages::OO_ParamTestWithInner_Action {
    Python::with_gil(|_py| {
      match self {
        Self::Inner(v) => messages::OO_ParamTestWithInner_Action::Inner(v.borrow(_py).to_rust()),
        Self::Val(v) => messages::OO_ParamTestWithInner_Action::Val((*v).into())
      }
    })
  }
  pub fn from_rust_obj(d: messages::OO_ParamTestWithInner_Action) -> PyResult<Self> {
    let r = Python::with_gil(|_py| {
      match d {
        messages::OO_ParamTestWithInner_Action::Inner(v) => Self::Inner(Py::new(_py, Inner::from_rust_obj(v).unwrap()).unwrap()),
        messages::OO_ParamTestWithInner_Action::Val(v) => Self::Val(v.val.into())
      }
    });
    Ok(r)
  }
}
impl std::fmt::Display for OO_ParamTestWithInner_Action {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      OO_ParamTestWithInner_Action::Inner(v) => {
        Python::with_gil(|py| {
          write!(f, "OO_ParamTestWithInner_Action(Inner({})", v.borrow(py).__repr__()) }
        )},
      OO_ParamTestWithInner_Action::Val(v) => { write!(f, "OO_ParamTestWithInner_Action(Val({}))", v) },
    }
  }
}



// *** Messages
/// Test comment for Inner
#[pyclass]
#[derive(Debug)]
pub struct Inner {
  #[pyo3(get, set)]
  pub val: u8,
  #[pyo3(get, set)]
  pub num: Numbers,
}
#[pymethods]
impl Inner {
  #[new]
  pub fn __new__( val: u8, num: Numbers,) -> Self {
    Self{ val: val.into(), num: num.into(), }
  }
  pub fn serialize(&self, py: Python) -> PyObject {
    let msg = self.to_rust();
    PyBytes::new(py, &serialize(&msg)).into()
  }
  #[staticmethod]
  pub fn deserialize(_py: Python, data: Bound<'_, PyBytes>) -> PyResult<Self> {
    println!("rust: {:?}", data);
    let dv: Vec<u8> = data.extract()?;
    let v = match deserialize::<messages::Inner>(&dv) {
      Some(v) => v, None => return Err(PyErr::new::<PyException, _>("Error when deserializing Inner"))
    };
    println!("{:?}", v);
    Self::from_rust_obj(v.0)
  }
  pub fn __repr__(&self) -> String {
    format!("{}", self)
  }

}
impl Inner {
  pub fn to_rust(&self) -> messages::Inner {
    Python::with_gil(|_py| {
      messages::Inner{ val: self.val.clone().into(), num: Numbers_py_to_rust(self.num.clone().into()),}
    })
  }
  pub fn from_rust_obj(d: messages::Inner) -> PyResult<Self> {
    let r = Python::with_gil(|_py| { Self{ val: d.val.val, num: Numbers_rust_to_py(d.num),} });
    Ok(r)
  }
}
impl std::fmt::Display for Inner {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    Python::with_gil(|_py| {
      write!(f, "Inner(val: ({}), num: ({}), )", self.val, self.num,)
    })
  }
}

#[pyclass]
#[derive(Debug)]
pub struct ParamTestWithInner {
  #[pyo3(get, set)]
  pub param_1: u8,
  #[pyo3(get, set)]
  pub param_2: bool,
  pub action: Py<OO_ParamTestWithInner_Action>,
}
#[pymethods]
impl ParamTestWithInner {
  #[new]
  pub fn __new__( param_1: u8, param_2: bool, action: Py<OO_ParamTestWithInner_Action>,) -> Self {
    Self{ param_1: param_1.into(), param_2: param_2.into(), action: action.into(), }
  }
  pub fn serialize(&self, py: Python) -> PyObject {
    let msg = self.to_rust();
    PyBytes::new(py, &serialize(&msg)).into()
  }
  #[staticmethod]
  pub fn deserialize(_py: Python, data: Bound<'_, PyBytes>) -> PyResult<Self> {
    println!("rust: {:?}", data);
    let dv: Vec<u8> = data.extract()?;
    let v = match deserialize::<messages::ParamTestWithInner>(&dv) {
      Some(v) => v, None => return Err(PyErr::new::<PyException, _>("Error when deserializing ParamTestWithInner"))
    };
    println!("{:?}", v);
    Self::from_rust_obj(v.0)
  }
  pub fn __repr__(&self) -> String {
    format!("{}", self)
  }

  #[getter]
  pub fn action_oo(&self) -> OO_ParamTestWithInner_ActionEnum {
    match self.action.get() {
      OO_ParamTestWithInner_Action::Inner(_) => OO_ParamTestWithInner_ActionEnum::Inner,
      OO_ParamTestWithInner_Action::Val(_) => OO_ParamTestWithInner_ActionEnum::Val,
  } }
  #[getter(action_inner)]
  fn action_get_inner(&self) -> Option<&Py<Inner>> {
    match self.action.get() {
      OO_ParamTestWithInner_Action::Inner(v) => Some(v), _ => None
  } }
  #[setter(action_inner)]
  fn action_set_inner(&mut self, v:Py<Inner>) -> PyResult<()> {
    Python::with_gil(|py| {
      self.action = Py::new(py, OO_ParamTestWithInner_Action::Inner(v))?;
      Ok(())
    })
  }
  #[getter(action_val)]
  fn action_get_val(&self) -> Option<&u8> {
    match self.action.get() {
      OO_ParamTestWithInner_Action::Val(v) => Some(v), _ => None
  } }
  #[setter(action_val)]
  fn action_set_val(&mut self, v:u8) -> PyResult<()> {
    Python::with_gil(|py| {
      self.action = Py::new(py, OO_ParamTestWithInner_Action::Val(v))?;
      Ok(())
    })
  }
}
impl ParamTestWithInner {
  pub fn to_rust(&self) -> messages::ParamTestWithInner {
    Python::with_gil(|_py| {
      messages::ParamTestWithInner{ param_1: self.param_1.clone().into(), param_2: self.param_2.clone().into(), action: self.action.borrow(_py).to_rust(),}
    })
  }
  pub fn from_rust_obj(d: messages::ParamTestWithInner) -> PyResult<Self> {
    let r = Python::with_gil(|_py| { Self{ param_1: d.param_1.val, param_2: d.param_2, action: Py::new(_py, OO_ParamTestWithInner_Action::from_rust_obj(d.action).unwrap()).unwrap(),} });
    Ok(r)
  }
}
impl std::fmt::Display for ParamTestWithInner {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    Python::with_gil(|_py| {
      write!(f, "ParamTestWithInner(param_1: ({}), param_2: ({}), action: ({}), )", self.param_1, self.param_2, self.action.borrow(_py).__repr__(),)
    })
  }
}


