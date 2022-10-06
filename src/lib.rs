mod evaluator;
use pyo3::prelude::*;

use crate::evaluator::evaluator::evaluate;


/// A Python module implemented in Rust.
#[pymodule]
fn evalrspy(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(evaluate, m)?)?;
    Ok(())
}