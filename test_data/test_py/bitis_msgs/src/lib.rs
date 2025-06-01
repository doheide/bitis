use pyo3::prelude::*;

mod pyrust;
mod messages;

use pyrust::*;


/// A Python module implemented in Rust.
#[pymodule]
fn bitis_msgs(m: &Bound<'_, PyModule>) -> PyResult<()> {


    m.add_class::<Numbers>()?;

    m.add_class::<Inner>()?;
    m.add_class::<ParamTestWithInner>()?;

    m.add_class::<OO_ParamTestWithInner_Action>()?;

    Ok(())
}
