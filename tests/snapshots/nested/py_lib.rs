use pyo3::prelude::*;

mod pyrust;
mod messages;

use pyrust::*;


/// A Python module implemented in Rust.
#[pymodule]
fn nested(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("BITIS_VERSION", messages::BITIS_VERSION)?;
    m.add("BITIS_SCHEMA_HASH", messages::BITIS_SCHEMA_HASH)?;
    m.add("DecodeError", m.py().get_type::<bitis_py_errors::DecodeError>())?;
    m.add("EncodeError", m.py().get_type::<bitis_py_errors::EncodeError>())?;


    m.add_class::<E>()?;

    m.add_class::<Top>()?;
    m.add_class::<Middle>()?;
    m.add_class::<Leaf>()?;

    m.add_class::<OO_Top_Outer>()?;
    m.add_class::<OO_Top_OuterEnum>()?;
    m.add_class::<OO_Outer_Inner>()?;
    m.add_class::<OO_Outer_InnerEnum>()?;

    Ok(())
}
