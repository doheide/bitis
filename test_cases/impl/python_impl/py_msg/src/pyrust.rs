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
#[pyclass]
#[derive(Debug, Clone)]
#[allow(nonstandard_style)]
#[allow(dead_code)]
pub enum SensorSource {
  TemperaturSensor,
  MovementSensor,
}
impl std::fmt::Display for SensorSource {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      SensorSource::TemperaturSensor => { write!(f, "SensorSource::TemperaturSensor") }, 
      SensorSource::MovementSensor => { write!(f, "SensorSource::MovementSensor") }, 
    }
} }

impl ValFromInto<SensorSource> for messages::SensorSource {
  fn val_into(&self) -> SensorSource {
    match self.clone() {
      messages::SensorSource::TemperaturSensor => SensorSource::TemperaturSensor, 
      messages::SensorSource::MovementSensor => SensorSource::MovementSensor, 
    }
  }
  fn val_from(v: &SensorSource) -> Self {
    match v {
      SensorSource::TemperaturSensor => messages::SensorSource::TemperaturSensor, 
      SensorSource::MovementSensor => messages::SensorSource::MovementSensor, 
    }
  }
}#[pyclass]
#[derive(Debug, Clone)]
#[allow(nonstandard_style)]
#[allow(dead_code)]
pub enum ExampleEnum {
  E1,
  E2,
  E3,
  E4,
  E5,
  E6,
  E7,
  E8,
  E9,
}
impl std::fmt::Display for ExampleEnum {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      ExampleEnum::E1 => { write!(f, "ExampleEnum::E1") }, 
      ExampleEnum::E2 => { write!(f, "ExampleEnum::E2") }, 
      ExampleEnum::E3 => { write!(f, "ExampleEnum::E3") }, 
      ExampleEnum::E4 => { write!(f, "ExampleEnum::E4") }, 
      ExampleEnum::E5 => { write!(f, "ExampleEnum::E5") }, 
      ExampleEnum::E6 => { write!(f, "ExampleEnum::E6") }, 
      ExampleEnum::E7 => { write!(f, "ExampleEnum::E7") }, 
      ExampleEnum::E8 => { write!(f, "ExampleEnum::E8") }, 
      ExampleEnum::E9 => { write!(f, "ExampleEnum::E9") }, 
    }
} }

impl ValFromInto<ExampleEnum> for messages::ExampleEnum {
  fn val_into(&self) -> ExampleEnum {
    match self.clone() {
      messages::ExampleEnum::E1 => ExampleEnum::E1, 
      messages::ExampleEnum::E2 => ExampleEnum::E2, 
      messages::ExampleEnum::E3 => ExampleEnum::E3, 
      messages::ExampleEnum::E4 => ExampleEnum::E4, 
      messages::ExampleEnum::E5 => ExampleEnum::E5, 
      messages::ExampleEnum::E6 => ExampleEnum::E6, 
      messages::ExampleEnum::E7 => ExampleEnum::E7, 
      messages::ExampleEnum::E8 => ExampleEnum::E8, 
      messages::ExampleEnum::E9 => ExampleEnum::E9, 
    }
  }
  fn val_from(v: &ExampleEnum) -> Self {
    match v {
      ExampleEnum::E1 => messages::ExampleEnum::E1, 
      ExampleEnum::E2 => messages::ExampleEnum::E2, 
      ExampleEnum::E3 => messages::ExampleEnum::E3, 
      ExampleEnum::E4 => messages::ExampleEnum::E4, 
      ExampleEnum::E5 => messages::ExampleEnum::E5, 
      ExampleEnum::E6 => messages::ExampleEnum::E6, 
      ExampleEnum::E7 => messages::ExampleEnum::E7, 
      ExampleEnum::E8 => messages::ExampleEnum::E8, 
      ExampleEnum::E9 => messages::ExampleEnum::E9, 
    }
  }
}

// *****************************************************************
// *****************************************************************
// *** Enums for oneof


// *****************************************************************
// *****************************************************************
// *** Messages
#[pyclass]
#[derive(Debug)]
#[allow(dead_code)]
pub struct Inner {
// lala
  #[pyo3(get, set)]
  pub val2: i16,
}
#[pymethods]
impl Inner {
  #[new]
  pub fn __new__( val2: i16,) -> Self {
    Self{ val2: val2.into(), }
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
    let v = match deserialize::<messages::Inner>(&dv) {
      Some(v) => v, None => return Err(PyErr::new::<PyException, _>("Error when deserializing Inner"))
    };
    Self::from_rust_obj(&v.0)
  }
  pub fn __repr__(&self) -> String {
    format!("{}", self)
  }

}
impl Inner {
  pub fn to_rust(&self) -> messages::Inner {
    Python::with_gil(|_py| {
      messages::Inner{ val2: do_val_from(&self.val2),}
    })
  }
  pub fn from_rust_obj(d: &messages::Inner) -> PyResult<Self> {
    let r = Python::with_gil(|_py| { Self{ val2: d.val2.val_into(),} });
    Ok(r)
  }
}
impl std::fmt::Display for Inner {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    Python::with_gil(|_py| {
      let t = self.to_rust();
//      write!(f, "Inner(val2: ({}), )",
// t.val2,)
      write!(f, "{}", t)
    })
  }
}
impl ValFromInto<Py<Inner>> for messages::Inner {
  fn val_into(self: &Self) -> Py<Inner> {
    Python::with_gil(|_py| { Py::new(_py, Inner::from_rust_obj(self).unwrap()).unwrap() })
  }
  fn val_from(val: &Py<Inner>) -> Self {
    Python::with_gil(|_py| { val.borrow(_py).to_rust() })
  }
}

#[pyclass]
#[derive(Debug)]
#[allow(dead_code)]
pub struct MsgFixedBaseArray {
// lala
  #[pyo3(get, set)]
  pub param_1: SensorSource,
  #[pyo3(get, set)]
  pub val1: [u16;3],
  #[pyo3(get, set)]
  pub val2: [i16;3],
  #[pyo3(get, set)]
  pub val3: [bool;3],
  #[pyo3(get, set)]
  pub val4: [i16;3],
  #[pyo3(get, set)]
  pub val5: [f64;3],
  #[pyo3(get, set)]
  pub val6: [f64;3],
  #[pyo3(get, set)]
  pub val7: [SensorSource;3],
  #[pyo3(get, set)]
  pub val8: [Py<Inner>;3],
}
#[pymethods]
impl MsgFixedBaseArray {
  #[new]
  pub fn __new__( param_1: SensorSource, val1: [u16;3], val2: [i16;3], val3: [bool;3], val4: [i16;3], val5: [f64;3], val6: [f64;3], val7: [SensorSource;3], val8: [Py<Inner>;3],) -> Self {
    Self{ param_1: param_1.into(), val1: val1.into(), val2: val2.into(), val3: val3.into(), val4: val4.into(), val5: val5.into(), val6: val6.into(), val7: val7.into(), val8: val8.into(), }
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
    let v = match deserialize::<messages::MsgFixedBaseArray>(&dv) {
      Some(v) => v, None => return Err(PyErr::new::<PyException, _>("Error when deserializing MsgFixedBaseArray"))
    };
    Self::from_rust_obj(&v.0)
  }
  pub fn __repr__(&self) -> String {
    format!("{}", self)
  }

}
impl MsgFixedBaseArray {
  pub fn to_rust(&self) -> messages::MsgFixedBaseArray {
    Python::with_gil(|_py| {
      messages::MsgFixedBaseArray{ param_1: do_val_from(&self.param_1), val1: do_val_from(&self.val1), val2: do_val_from(&self.val2), val3: do_val_from(&self.val3), val4: do_val_from(&self.val4), val5: do_val_from(&self.val5), val6: do_val_from(&self.val6), val7: do_val_from(&self.val7), val8: do_val_from(&self.val8),}
    })
  }
  pub fn from_rust_obj(d: &messages::MsgFixedBaseArray) -> PyResult<Self> {
    let r = Python::with_gil(|_py| { Self{ param_1: d.param_1.val_into(), val1: d.val1.val_into(), val2: d.val2.val_into(), val3: d.val3.val_into(), val4: d.val4.val_into(), val5: d.val5.val_into(), val6: d.val6.val_into(), val7: d.val7.val_into(), val8: d.val8.val_into(),} });
    Ok(r)
  }
}
impl std::fmt::Display for MsgFixedBaseArray {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    Python::with_gil(|_py| {
      let t = self.to_rust();
//      write!(f, "MsgFixedBaseArray(param_1: ({}), val1: ({}), val2: ({}), val3: ({}), val4: ({}), val5: ({}), val6: ({}), val7: ({}), val8: ({}), )",
// t.param_1, t.val1, t.val2, t.val3, t.val4, t.val5, t.val6, t.val7, t.val8,)
      write!(f, "{}", t)
    })
  }
}
impl ValFromInto<Py<MsgFixedBaseArray>> for messages::MsgFixedBaseArray {
  fn val_into(self: &Self) -> Py<MsgFixedBaseArray> {
    Python::with_gil(|_py| { Py::new(_py, MsgFixedBaseArray::from_rust_obj(self).unwrap()).unwrap() })
  }
  fn val_from(val: &Py<MsgFixedBaseArray>) -> Self {
    Python::with_gil(|_py| { val.borrow(_py).to_rust() })
  }
}


