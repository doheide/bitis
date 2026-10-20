use pyo3::prelude::*;

mod pyrust;
mod messages;

use pyrust::*;


/// A Python module implemented in Rust.
#[pymodule]
fn bounds(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("BITIS_VERSION", messages::BITIS_VERSION)?;
    m.add("BITIS_SCHEMA_HASH", messages::BITIS_SCHEMA_HASH)?;
    m.add("DecodeError", m.py().get_type::<bitis_py_errors::DecodeError>())?;
    m.add("EncodeError", m.py().get_type::<bitis_py_errors::EncodeError>())?;
    m.add_function(wrap_pyfunction!(bitis_py_crc32, m)?)?;
    m.add_class::<BitisPyCrc32>()?;
    m.add_function(wrap_pyfunction!(bitis_py_crc8_autosar, m)?)?;
    m.add_class::<BitisPyCrc8Autosar>()?;
    m.add_function(wrap_pyfunction!(bitis_py_crc16_ibm3740, m)?)?;
    m.add_class::<BitisPyCrc16Ibm3740>()?;
    m.add_function(wrap_pyfunction!(bitis_py_transmission_crc_len, m)?)?;
    m.add_function(wrap_pyfunction!(bitis_py_transmission_crc, m)?)?;
    m.add_function(wrap_pyfunction!(bitis_py_append_transmission_crc, m)?)?;
    m.add_function(wrap_pyfunction!(bitis_py_verify_transmission_crc, m)?)?;
    m.add_function(wrap_pyfunction!(bitis_py_strip_transmission_crc, m)?)?;


    m.add_class::<Kind>()?;

    m.add_class::<Point>()?;
    m.add_class::<Bounded>()?;

    m.add_class::<OO_Bounded_Payload>()?;
    m.add_class::<OO_Bounded_PayloadEnum>()?;

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

// *****************************************************************
// CRC-8/AUTOSAR, CRC-16/IBM-3740 and the transmission CRC rule, see bitis_lib::crc8_autosar,
// bitis_lib::crc16_ibm3740, bitis_lib::transmission_crc_len

/// CRC-8/AUTOSAR of data (poly 0x2F, init 0xFF, xorout 0xFF, not reflected).
#[pyfunction]
#[pyo3(name = "crc8_autosar")]
fn bitis_py_crc8_autosar(data: &Bound<'_, PyAny>) -> PyResult<u8> {
    bitis_py_bytes_of(data, bitis_lib::crc8_autosar)
}

/// Incremental CRC-8/AUTOSAR: update() any number of times, finish() / value gives the CRC.
#[pyclass(name = "Crc8Autosar")]
#[derive(Default)]
struct BitisPyCrc8Autosar { inner: bitis_lib::Crc8Autosar }

#[pymethods]
impl BitisPyCrc8Autosar {
    #[new]
    fn new() -> Self { Self::default() }
    fn update(&mut self, data: &Bound<'_, PyAny>) -> PyResult<()> {
        bitis_py_bytes_of(data, |d| self.inner.update(d))
    }
    fn finish(&self) -> u8 { self.inner.finish() }
    #[getter]
    fn value(&self) -> u8 { self.inner.finish() }
    fn reset(&mut self) { self.inner.reset() }
}

/// CRC-16/IBM-3740 (CCITT-FALSE) of data (poly 0x1021, init 0xFFFF, xorout 0, not reflected).
#[pyfunction]
#[pyo3(name = "crc16_ibm3740")]
fn bitis_py_crc16_ibm3740(data: &Bound<'_, PyAny>) -> PyResult<u16> {
    bitis_py_bytes_of(data, bitis_lib::crc16_ibm3740)
}

/// Incremental CRC-16/IBM-3740: update() any number of times, finish() / value gives the CRC.
#[pyclass(name = "Crc16Ibm3740")]
#[derive(Default)]
struct BitisPyCrc16Ibm3740 { inner: bitis_lib::Crc16Ibm3740 }

#[pymethods]
impl BitisPyCrc16Ibm3740 {
    #[new]
    fn new() -> Self { Self::default() }
    fn update(&mut self, data: &Bound<'_, PyAny>) -> PyResult<()> {
        bitis_py_bytes_of(data, |d| self.inner.update(d))
    }
    fn finish(&self) -> u16 { self.inner.finish() }
    #[getter]
    fn value(&self) -> u16 { self.inner.finish() }
    fn reset(&mut self) { self.inner.reset() }
}

/// Trailer length for a frame of n bytes without the CRC: 1 (CRC-8) for n <= 14, else 2 (CRC-16).
#[pyfunction]
#[pyo3(name = "transmission_crc_len")]
fn bitis_py_transmission_crc_len(n: usize) -> usize {
    bitis_lib::transmission_crc_len(n)
}

/// Transmission CRC of frame (header + payload): CRC-8/AUTOSAR for len <= 14, else CRC-16/IBM-3740.
#[pyfunction]
#[pyo3(name = "transmission_crc")]
fn bitis_py_transmission_crc(frame: &Bound<'_, PyAny>) -> PyResult<u16> {
    bitis_py_bytes_of(frame, bitis_lib::transmission_crc)
}

/// frame (header + payload) followed by its transmission CRC trailer (CRC-16 little-endian).
#[pyfunction]
#[pyo3(name = "append_transmission_crc")]
fn bitis_py_append_transmission_crc<'py>(frame: &Bound<'py, PyAny>) -> PyResult<Bound<'py, pyo3::types::PyBytes>> {
    let out = bitis_py_bytes_of(frame, bitis_lib::with_transmission_crc)?;
    Ok(pyo3::types::PyBytes::new(frame.py(), &out))
}

/// True if data (frame with trailer) has a valid length (not 0 or 16) and a matching CRC.
#[pyfunction]
#[pyo3(name = "verify_transmission_crc")]
fn bitis_py_verify_transmission_crc(data: &Bound<'_, PyAny>) -> PyResult<bool> {
    bitis_py_bytes_of(data, bitis_lib::verify_transmission_crc)
}

/// The frame without its trailer; ValueError for an invalid length (0 or 16) or a CRC mismatch.
#[pyfunction]
#[pyo3(name = "strip_transmission_crc")]
fn bitis_py_strip_transmission_crc<'py>(data: &Bound<'py, PyAny>) -> PyResult<Bound<'py, pyo3::types::PyBytes>> {
    let py = data.py();
    bitis_py_bytes_of(data, |d| match bitis_lib::strip_transmission_crc(d) {
        Some(frame) => Ok(pyo3::types::PyBytes::new(py, frame)),
        None if d.is_empty() || d.len() == 16 => Err(pyo3::exceptions::PyValueError::new_err(
            format!("invalid transmission frame length {}", d.len()))),
        None => Err(pyo3::exceptions::PyValueError::new_err("transmission CRC mismatch")),
    })?
}