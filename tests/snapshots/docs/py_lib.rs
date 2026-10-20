use pyo3::prelude::*;

mod pyrust;
mod messages;

use pyrust::*;


/// A Python module implemented in Rust.
#[pymodule]
fn docs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("BITIS_VERSION", messages::BITIS_VERSION)?;
    m.add("BITIS_SCHEMA_HASH", messages::BITIS_SCHEMA_HASH)?;
    m.add("DecodeError", m.py().get_type::<bitis_py_errors::DecodeError>())?;
    m.add("EncodeError", m.py().get_type::<bitis_py_errors::EncodeError>())?;


    m.add_class::<Kind>()?;

    m.add_class::<Documented>()?;

    m.add_class::<OO_Documented_Sel>()?;
    m.add_class::<OO_Documented_SelEnum>()?;

    Ok(())
}
