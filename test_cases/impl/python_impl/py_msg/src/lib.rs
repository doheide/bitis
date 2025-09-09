use pyo3::prelude::*;

mod pyrust;
mod messages;

use pyrust::*;


/// A Python module implemented in Rust.
#[pymodule]
fn py_msg(m: &Bound<'_, PyModule>) -> PyResult<()> {



    m.add_class::<MsgOOSimpleBase>()?;
    m.add_class::<MsgSimpleBaseOneInt>()?;
    m.add_class::<MsgOONestedBase>()?;

    m.add_class::<OO_MsgOoSimpleBase_Value>()?;
    m.add_class::<OO_MsgOoSimpleBase_ValueEnum>()?;
    m.add_class::<OO_MsgOoNestedBase_Value>()?;
    m.add_class::<OO_MsgOoNestedBase_ValueEnum>()?;

    Ok(())
}
