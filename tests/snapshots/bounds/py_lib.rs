use pyo3::prelude::*;

mod pyrust;
mod messages;

use pyrust::*;


/// A Python module implemented in Rust.
#[pymodule]
fn bounds(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("BITIS_VERSION", messages::BITIS_VERSION)?;
    m.add("BITIS_SCHEMA_HASH", messages::BITIS_SCHEMA_HASH)?;


    m.add_class::<Kind>()?;

    m.add_class::<Point>()?;
    m.add_class::<Bounded>()?;

    m.add_class::<OO_Bounded_Payload>()?;
    m.add_class::<OO_Bounded_PayloadEnum>()?;

    Ok(())
}
