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
    m.add_function(wrap_pyfunction!(bitis_py_crc32, m)?)?;
    m.add_class::<BitisPyCrc32>()?;


    m.add_class::<Color>()?;

    m.add_class::<Inner>()?;
    m.add_class::<Empty>()?;
    m.add_class::<AllTypes>()?;

    m.add_class::<OO_AllTypes_Choice>()?;
    m.add_class::<OO_AllTypes_ChoiceEnum>()?;

    Ok(())
}

// *****************************************************************
// CRC-32 helper (CRC-32/ISO-HDLC, same values as zlib.crc32), see bitis_lib::crc32

/// Bytes of `data`: bytes directly, other sequences of ints (bytearray, memoryview, list)
/// via extraction into a Vec<u8>.
fn bitis_py_bytes_of<R>(data: &Bound<'_, PyAny>, f: impl FnOnce(&[u8]) -> R) -> PyResult<R> {
    match data.extract::<&[u8]>() {
        Ok(b) => Ok(f(b)),
        Err(_) => Ok(f(&data.extract::<Vec<u8>>()?)),
    }
}

/// CRC-32/ISO-HDLC of data (same value as zlib.crc32(data)).
#[pyfunction]
#[pyo3(name = "crc32")]
fn bitis_py_crc32(data: &Bound<'_, PyAny>) -> PyResult<u32> {
    bitis_py_bytes_of(data, bitis_lib::crc32)
}

/// Incremental CRC-32/ISO-HDLC: update() any number of times, finish() / value gives the CRC.
#[pyclass(name = "Crc32")]
#[derive(Default)]
struct BitisPyCrc32 { inner: bitis_lib::Crc32 }

#[pymethods]
impl BitisPyCrc32 {
    #[new]
    fn new() -> Self { Self::default() }
    fn update(&mut self, data: &Bound<'_, PyAny>) -> PyResult<()> {
        bitis_py_bytes_of(data, |d| self.inner.update(d))
    }
    fn finish(&self) -> u32 { self.inner.finish() }
    #[getter]
    fn value(&self) -> u32 { self.inner.finish() }
    fn reset(&mut self) { self.inner.reset() }
}