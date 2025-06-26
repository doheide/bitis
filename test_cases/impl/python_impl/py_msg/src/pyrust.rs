use bitis_lib::*;

use pyo3::prelude::*;
use pyo3::exceptions::PyException;
use pyo3::types::{PyBytes};

use super::messages;

// Base
fn do_val_from<T: ValFromInto<U>, U>(v: &U) -> T {
    T::val_from(v)
}

// *****************************************************************
// *****************************************************************
// Enums


// *****************************************************************
// *****************************************************************
// *** Enums for oneof


// *****************************************************************
// *****************************************************************
// *** Messages
#[pyclass]
#[derive(Debug)]
#[allow(dead_code)]
pub struct MsgSimpleBaseOneInt {
// lala
  #[pyo3(get, set)]
  pub param_1: u16,
}
#[pymethods]
impl MsgSimpleBaseOneInt {
  #[new]
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
  pub fn from_rust_obj(d: &messages::MsgSimpleBaseOneInt) -> PyResult<Self> {
    let r = Python::with_gil(|_py| { Self{ param_1: d.param_1.val_into(),} });
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
#[allow(dead_code)]
pub struct MsgSimpleBaseThreeInt {
// lala
  #[pyo3(get, set)]
  pub param_1: u16,
  #[pyo3(get, set)]
  pub param_2: u16,
  #[pyo3(get, set)]
  pub param_3: u16,
  #[pyo3(get, set)]
  pub param_4: u16,
}
#[pymethods]
impl MsgSimpleBaseThreeInt {
  #[new]
  pub fn __new__( param_1: u16, param_2: u16, param_3: u16, param_4: u16,) -> Self {
    Self{ param_1: param_1.into(), param_2: param_2.into(), param_3: param_3.into(), param_4: param_4.into(), }
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
    let v = match deserialize::<messages::MsgSimpleBaseThreeInt>(&dv) {
      Some(v) => v, None => return Err(PyErr::new::<PyException, _>("Error when deserializing MsgSimpleBaseThreeInt"))
    };
    Self::from_rust_obj(&v.0)
  }
  pub fn __repr__(&self) -> String {
    format!("{}", self)
  }

}
impl MsgSimpleBaseThreeInt {
  pub fn to_rust(&self) -> messages::MsgSimpleBaseThreeInt {
    Python::with_gil(|_py| {
      messages::MsgSimpleBaseThreeInt{ param_1: do_val_from(&self.param_1), param_2: do_val_from(&self.param_2), param_3: do_val_from(&self.param_3), param_4: do_val_from(&self.param_4),}
    })
  }
  pub fn from_rust_obj(d: &messages::MsgSimpleBaseThreeInt) -> PyResult<Self> {
    let r = Python::with_gil(|_py| { Self{ param_1: d.param_1.val_into(), param_2: d.param_2.val_into(), param_3: d.param_3.val_into(), param_4: d.param_4.val_into(),} });
    Ok(r)
  }
}
impl std::fmt::Display for MsgSimpleBaseThreeInt {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    Python::with_gil(|_py| {
      let t = self.to_rust();
      write!(f, "{}", t)
    })
  }
}
impl ValFromInto<Py<MsgSimpleBaseThreeInt>> for messages::MsgSimpleBaseThreeInt {
  fn val_into(self: &Self) -> Py<MsgSimpleBaseThreeInt> {
    Python::with_gil(|_py| { Py::new(_py, MsgSimpleBaseThreeInt::from_rust_obj(self).unwrap()).unwrap() })
  }
  fn val_from(val: &Py<MsgSimpleBaseThreeInt>) -> Self {
    Python::with_gil(|_py| { val.borrow(_py).to_rust() })
  }
}

#[pyclass]
#[derive(Debug)]
#[allow(dead_code)]
pub struct MsgSimpleTestBase {
// lala
  #[pyo3(get, set)]
  pub param_1: u16,
  #[pyo3(get, set)]
  pub param_2: bool,
  #[pyo3(get, set)]
  pub param_3: i16,
  #[pyo3(get, set)]
  pub name: String,
}
#[pymethods]
impl MsgSimpleTestBase {
  #[new]
  pub fn __new__( param_1: u16, param_2: bool, param_3: i16, name: String,) -> Self {
    Self{ param_1: param_1.into(), param_2: param_2.into(), param_3: param_3.into(), name: name.into(), }
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
    let v = match deserialize::<messages::MsgSimpleTestBase>(&dv) {
      Some(v) => v, None => return Err(PyErr::new::<PyException, _>("Error when deserializing MsgSimpleTestBase"))
    };
    Self::from_rust_obj(&v.0)
  }
  pub fn __repr__(&self) -> String {
    format!("{}", self)
  }

}
impl MsgSimpleTestBase {
  pub fn to_rust(&self) -> messages::MsgSimpleTestBase {
    Python::with_gil(|_py| {
      messages::MsgSimpleTestBase{ param_1: do_val_from(&self.param_1), param_2: do_val_from(&self.param_2), param_3: do_val_from(&self.param_3), name: do_val_from(&self.name),}
    })
  }
  pub fn from_rust_obj(d: &messages::MsgSimpleTestBase) -> PyResult<Self> {
    let r = Python::with_gil(|_py| { Self{ param_1: d.param_1.val_into(), param_2: d.param_2.val_into(), param_3: d.param_3.val_into(), name: d.name.val_into(),} });
    Ok(r)
  }
}
impl std::fmt::Display for MsgSimpleTestBase {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    Python::with_gil(|_py| {
      let t = self.to_rust();
      write!(f, "{}", t)
    })
  }
}
impl ValFromInto<Py<MsgSimpleTestBase>> for messages::MsgSimpleTestBase {
  fn val_into(self: &Self) -> Py<MsgSimpleTestBase> {
    Python::with_gil(|_py| { Py::new(_py, MsgSimpleTestBase::from_rust_obj(self).unwrap()).unwrap() })
  }
  fn val_from(val: &Py<MsgSimpleTestBase>) -> Self {
    Python::with_gil(|_py| { val.borrow(_py).to_rust() })
  }
}

#[pyclass]
#[derive(Debug)]
#[allow(dead_code)]
pub struct MsgSimpleTestFP {
// lala
  #[pyo3(get, set)]
  pub param_1: bool,
  #[pyo3(get, set)]
  pub fp: f64,
}
#[pymethods]
impl MsgSimpleTestFP {
  #[new]
  pub fn __new__( param_1: bool, fp: f64,) -> Self {
    Self{ param_1: param_1.into(), fp: fp.into(), }
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
    let v = match deserialize::<messages::MsgSimpleTestFP>(&dv) {
      Some(v) => v, None => return Err(PyErr::new::<PyException, _>("Error when deserializing MsgSimpleTestFP"))
    };
    Self::from_rust_obj(&v.0)
  }
  pub fn __repr__(&self) -> String {
    format!("{}", self)
  }

}
impl MsgSimpleTestFP {
  pub fn to_rust(&self) -> messages::MsgSimpleTestFP {
    Python::with_gil(|_py| {
      messages::MsgSimpleTestFP{ param_1: do_val_from(&self.param_1), fp: do_val_from(&self.fp),}
    })
  }
  pub fn from_rust_obj(d: &messages::MsgSimpleTestFP) -> PyResult<Self> {
    let r = Python::with_gil(|_py| { Self{ param_1: d.param_1.val_into(), fp: d.fp.val_into(),} });
    Ok(r)
  }
}
impl std::fmt::Display for MsgSimpleTestFP {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    Python::with_gil(|_py| {
      let t = self.to_rust();
      write!(f, "{}", t)
    })
  }
}
impl ValFromInto<Py<MsgSimpleTestFP>> for messages::MsgSimpleTestFP {
  fn val_into(self: &Self) -> Py<MsgSimpleTestFP> {
    Python::with_gil(|_py| { Py::new(_py, MsgSimpleTestFP::from_rust_obj(self).unwrap()).unwrap() })
  }
  fn val_from(val: &Py<MsgSimpleTestFP>) -> Self {
    Python::with_gil(|_py| { val.borrow(_py).to_rust() })
  }
}

#[pyclass]
#[derive(Debug)]
#[allow(dead_code)]
pub struct MsgSimpleOpt {
// lala
  #[pyo3(get, set)]
  pub param_1: u16,
  #[pyo3(get, set)]
  pub param_2: Option<bool>,
  #[pyo3(get, set)]
  pub param_3: Option<u16>,
  #[pyo3(get, set)]
  pub param_4: Option<f64>,
}
#[pymethods]
impl MsgSimpleOpt {
  #[new]
  pub fn __new__( param_1: u16, param_2: Option<bool>, param_3: Option<u16>, param_4: Option<f64>,) -> Self {
    Self{ param_1: param_1.into(), param_2: param_2.into(), param_3: param_3.into(), param_4: param_4.into(), }
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
    let v = match deserialize::<messages::MsgSimpleOpt>(&dv) {
      Some(v) => v, None => return Err(PyErr::new::<PyException, _>("Error when deserializing MsgSimpleOpt"))
    };
    Self::from_rust_obj(&v.0)
  }
  pub fn __repr__(&self) -> String {
    format!("{}", self)
  }

}
impl MsgSimpleOpt {
  pub fn to_rust(&self) -> messages::MsgSimpleOpt {
    Python::with_gil(|_py| {
      messages::MsgSimpleOpt{ param_1: do_val_from(&self.param_1), param_2: do_val_from(&self.param_2), param_3: do_val_from(&self.param_3), param_4: do_val_from(&self.param_4),}
    })
  }
  pub fn from_rust_obj(d: &messages::MsgSimpleOpt) -> PyResult<Self> {
    let r = Python::with_gil(|_py| { Self{ param_1: d.param_1.val_into(), param_2: d.param_2.val_into(), param_3: d.param_3.val_into(), param_4: d.param_4.val_into(),} });
    Ok(r)
  }
}
impl std::fmt::Display for MsgSimpleOpt {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    Python::with_gil(|_py| {
      let t = self.to_rust();
      write!(f, "{}", t)
    })
  }
}
impl ValFromInto<Py<MsgSimpleOpt>> for messages::MsgSimpleOpt {
  fn val_into(self: &Self) -> Py<MsgSimpleOpt> {
    Python::with_gil(|_py| { Py::new(_py, MsgSimpleOpt::from_rust_obj(self).unwrap()).unwrap() })
  }
  fn val_from(val: &Py<MsgSimpleOpt>) -> Self {
    Python::with_gil(|_py| { val.borrow(_py).to_rust() })
  }
}


