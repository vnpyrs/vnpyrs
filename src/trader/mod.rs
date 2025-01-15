pub mod constant;
pub mod database;
pub mod database_impl;
pub mod object;
pub mod setting;
pub mod utility;

use pyo3::{prelude::*, types::PyDict, wrap_pymodule};

#[pymodule]
pub fn trader(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_wrapped(wrap_pymodule!(constant::constant))?;
    m.add_wrapped(wrap_pymodule!(object::object))?;
    m.add_wrapped(wrap_pymodule!(utility::utility))?;

    let sys = PyModule::import(py, "sys")?;
    let sys_modules: Bound<'_, PyDict> = sys.getattr("modules")?.downcast_into()?;
    sys_modules.set_item("vnpyrs.trader.constant", m.getattr("constant")?)?;
    sys_modules.set_item("vnpyrs.trader.object", m.getattr("object")?)?;
    sys_modules.set_item("vnpyrs.trader.utility", m.getattr("utility")?)?;

    Ok(())
}
