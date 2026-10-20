use pyo3::prelude::*;

mod pyrust;
mod messages;

use pyrust::*;


/// A Python module implemented in Rust.
#[pymodule]
fn all_types(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("BITIS_VERSION", messages::BITIS_VERSION)?;
    m.add("BITIS_SCHEMA_HASH", messages::BITIS_SCHEMA_HASH)?;
    m.add("DecodeError", m.py().get_type::<bitis_py_errors::DecodeError>())?;
    m.add("EncodeError", m.py().get_type::<bitis_py_errors::EncodeError>())?;


    m.add_class::<Color>()?;

    m.add_class::<Inner>()?;
    m.add_class::<Empty>()?;
    m.add_class::<AllTypes>()?;

    m.add_class::<OO_AllTypes_Choice>()?;
    m.add_class::<OO_AllTypes_ChoiceEnum>()?;

    Ok(())
}
