use pyo3::prelude::*;

mod pyrust;
mod messages;

use pyrust::*;


/// A Python module implemented in Rust.
#[pymodule]
fn py_msg(m: &Bound<'_, PyModule>) -> PyResult<()> {


    m.add_class::<SensorSource>()?;
    m.add_class::<ExampleEnum>()?;

    m.add_class::<Inner>()?;
    m.add_class::<MsgFixedBaseArray>()?;


    Ok(())
}
