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

#[pyclass(eq, eq_int)]
#[derive(Debug, PartialEq)]
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
  fn new_flag(flag: bool) -> PyResult<Py<Self>> {
    Ok(Python::with_gil(|_py| { Py::new(_py, Self::Flag(flag)) })?)
  }
  #[staticmethod]
  fn new_number(number: u16) -> PyResult<Py<Self>> {
    Ok(Python::with_gil(|_py| { Py::new(_py, Self::Number(number)) })?)
  }
  #[staticmethod]
  fn new_inner_val(inner_val: Py<Inner>) -> PyResult<Py<Self>> {
    Ok(Python::with_gil(|_py| { Py::new(_py, Self::InnerVal(inner_val)) })?)
  }
  #[staticmethod]
  fn new_color_val(color_val: Color) -> PyResult<Py<Self>> {
    Ok(Python::with_gil(|_py| { Py::new(_py, Self::ColorVal(color_val)) })?)
  }
  #[staticmethod]
  fn new_text(text: String) -> PyResult<Py<Self>> {
    Ok(Python::with_gil(|_py| { Py::new(_py, Self::Text(text)) })?)
  }
  #[staticmethod]
  fn new_nothing(nothing: Py<Empty>) -> PyResult<Py<Self>> {
    Ok(Python::with_gil(|_py| { Py::new(_py, Self::Nothing(nothing)) })?)
  }
  fn __repr__(&self) -> String { format!("{}", self) }
}
impl OO_AllTypes_Choice {
  pub fn to_rust(&self) -> messages::OO_AllTypes_Choice {
    Python::with_gil(|_py| {
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
    let r = Python::with_gil(|_py| {
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
        Python::with_gil(|py| {
          write!(f, "OO_AllTypes_Choice(InnerVal({})", v.borrow(py).__repr__()) }
        )},
      OO_AllTypes_Choice::ColorVal(v) => { write!(f, "OO_AllTypes_Choice(ColorVal({}))", v) },
      OO_AllTypes_Choice::Text(v) => { write!(f, "OO_AllTypes_Choice(Text({}))", v) },
      OO_AllTypes_Choice::Nothing(v) => {
        Python::with_gil(|py| {
          write!(f, "OO_AllTypes_Choice(Nothing({})", v.borrow(py).__repr__()) }
        )},
    }
  }
}

impl ValFromInto<Py<OO_AllTypes_Choice>> for messages::OO_AllTypes_Choice {
  fn val_into(self: &Self) -> Py<OO_AllTypes_Choice> {
    Python::with_gil(|_py| { Py::new(_py, OO_AllTypes_Choice::from_rust_obj(self).unwrap()).unwrap() })
  }
  fn val_from(val: &Py<OO_AllTypes_Choice>) -> Self {
    Python::with_gil(|_py| { val.borrow(_py).to_rust() })
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
      messages::Inner{ a: do_val_from(&self.a),}
    })
  }
  pub fn from_rust_obj(_d: &messages::Inner) -> PyResult<Self> {
    let r = Python::with_gil(|_py| { Self{ a: _d.a.val_into(),} });
    Ok(r)
  }
}
impl std::fmt::Display for Inner {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    Python::with_gil(|_py| {
      let t = self.to_rust();
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

  pub fn serialize(&self, py: Python) -> (PyObject, u64, u64) {
    let msg = self.to_rust();
    let r = serialize(&msg);
    (PyBytes::new(py, &r.0).into(), r.1.total_bits, r.1.total_bytes)
  }
  #[staticmethod]
  pub fn deserialize(_py: Python, data: Bound<'_, PyBytes>) -> PyResult<Self> {
    let dv: Vec<u8> = data.extract()?;
    let v = match deserialize::<messages::Empty>(&dv) {
      Some(v) => v, None => return Err(PyErr::new::<PyException, _>("Error when deserializing Empty"))
    };
    Self::from_rust_obj(&v.0)
  }
  pub fn __repr__(&self) -> String {
    format!("{}", self)
  }

}
impl Empty {
  pub fn to_rust(&self) -> messages::Empty {
    Python::with_gil(|_py| {
      messages::Empty{}
    })
  }
  pub fn from_rust_obj(_d: &messages::Empty) -> PyResult<Self> {
    let r = Python::with_gil(|_py| { Self{} });
    Ok(r)
  }
}
impl std::fmt::Display for Empty {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    Python::with_gil(|_py| {
      let t = self.to_rust();
      write!(f, "{}", t)
    })
  }
}
impl ValFromInto<Py<Empty>> for messages::Empty {
  fn val_into(self: &Self) -> Py<Empty> {
    Python::with_gil(|_py| { Py::new(_py, Empty::from_rust_obj(self).unwrap()).unwrap() })
  }
  fn val_from(val: &Py<Empty>) -> Self {
    Python::with_gil(|_py| { val.borrow(_py).to_rust() })
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

  pub fn serialize(&self, py: Python) -> (PyObject, u64, u64) {
    let msg = self.to_rust();
    let r = serialize(&msg);
    (PyBytes::new(py, &r.0).into(), r.1.total_bits, r.1.total_bytes)
  }
  #[staticmethod]
  pub fn deserialize(_py: Python, data: Bound<'_, PyBytes>) -> PyResult<Self> {
    let dv: Vec<u8> = data.extract()?;
    let v = match deserialize::<messages::AllTypes>(&dv) {
      Some(v) => v, None => return Err(PyErr::new::<PyException, _>("Error when deserializing AllTypes"))
    };
    Self::from_rust_obj(&v.0)
  }
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
    Python::with_gil(|py| {
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
    Python::with_gil(|py| {
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
    Python::with_gil(|py| {
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
    Python::with_gil(|py| {
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
    Python::with_gil(|py| {
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
    Python::with_gil(|py| {
      self.choice = Py::new(py, OO_AllTypes_Choice::Nothing(v))?;
      Ok(())
    })
  }
}
impl AllTypes {
  pub fn to_rust(&self) -> messages::AllTypes {
    Python::with_gil(|_py| {
      messages::AllTypes{ b: do_val_from(&self.b), u1: do_val_from(&self.u1), u17: do_val_from(&self.u17), u64v: do_val_from(&self.u64v), i5: do_val_from(&self.i5), i33: do_val_from(&self.i33), ud: do_val_from(&self.ud), id: do_val_from(&self.id), f: do_val_from(&self.f), d: do_val_from(&self.d), fp: do_val_from(&self.fp), lat: do_val_from(&self.lat), s: do_val_from(&self.s), data: do_val_from(&self.data), color: do_val_from(&self.color), inner: do_val_from(&self.inner), empty: do_val_from(&self.empty), opt_u: do_val_from(&self.opt_u), opt_inner: do_val_from(&self.opt_inner), opt_empty: do_val_from(&self.opt_empty), rf_b: do_val_from(&self.rf_b), rf_inner: do_val_from(&self.rf_inner), rd_u: do_val_from(&self.rd_u), rd_color: do_val_from(&self.rd_color), rd_inner: do_val_from(&self.rd_inner), choice: do_val_from(&self.choice),}
    })
  }
  pub fn from_rust_obj(_d: &messages::AllTypes) -> PyResult<Self> {
    let r = Python::with_gil(|_py| { Self{ b: _d.b.val_into(), u1: _d.u1.val_into(), u17: _d.u17.val_into(), u64v: _d.u64v.val_into(), i5: _d.i5.val_into(), i33: _d.i33.val_into(), ud: _d.ud.val_into(), id: _d.id.val_into(), f: _d.f.val_into(), d: _d.d.val_into(), fp: _d.fp.val_into(), lat: _d.lat.val_into(), s: _d.s.val_into(), data: _d.data.val_into(), color: _d.color.val_into(), inner: _d.inner.val_into(), empty: _d.empty.val_into(), opt_u: _d.opt_u.val_into(), opt_inner: _d.opt_inner.val_into(), opt_empty: _d.opt_empty.val_into(), rf_b: _d.rf_b.val_into(), rf_inner: _d.rf_inner.val_into(), rd_u: _d.rd_u.val_into(), rd_color: _d.rd_color.val_into(), rd_inner: _d.rd_inner.val_into(), choice: _d.choice.val_into(),} });
    Ok(r)
  }
}
impl std::fmt::Display for AllTypes {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    Python::with_gil(|_py| {
      let t = self.to_rust();
      write!(f, "{}", t)
    })
  }
}
impl ValFromInto<Py<AllTypes>> for messages::AllTypes {
  fn val_into(self: &Self) -> Py<AllTypes> {
    Python::with_gil(|_py| { Py::new(_py, AllTypes::from_rust_obj(self).unwrap()).unwrap() })
  }
  fn val_from(val: &Py<AllTypes>) -> Self {
    Python::with_gil(|_py| { val.borrow(_py).to_rust() })
  }
}


