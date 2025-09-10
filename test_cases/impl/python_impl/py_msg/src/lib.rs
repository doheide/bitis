use pyo3::prelude::*;

mod pyrust;
mod messages;

use pyrust::*;


/// A Python module implemented in Rust.
#[pymodule]
fn py_msg(m: &Bound<'_, PyModule>) -> PyResult<()> {



    m.add_class::<MsgSimpleBaseOneInt>()?;
    m.add_class::<MsgSimpleBaseThreeInt>()?;
    m.add_class::<MsgSimpleTestBase>()?;
    m.add_class::<MsgSimpleTestFP>()?;
    m.add_class::<MsgSimpleOpt>()?;


    Ok(())
}
