pub mod backtesting;
pub mod template;
pub mod trader;

use backtesting::base::{EngineType, StopOrder};
use pyo3::{prelude::*, types::PyDict, wrap_pymodule};
use trader::object::{BarData, OrderData, TickData, TradeData};

/// A Python module implemented in Rust.
#[pymodule]
fn vnpyrs(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_wrapped(wrap_pymodule!(backtesting::backtesting))?;
    m.add_wrapped(wrap_pymodule!(trader::trader))?;
    m.add_class::<StopOrder>()?;
    m.add_class::<EngineType>()?;
    m.add_class::<TickData>()?;
    m.add_class::<BarData>()?;
    m.add_class::<TradeData>()?;
    m.add_class::<OrderData>()?;
    m.add_class::<OrderData>()?;

    // Inserting to sys.modules allows importing submodules nicely from Python
    // e.g. from maturin_starter.submodule import SubmoduleClass

    let sys = PyModule::import(py, "sys")?;
    let sys_modules: Bound<'_, PyDict> = sys.getattr("modules")?.downcast_into()?;
    sys_modules.set_item("vnpyrs.backtesting", m.getattr("backtesting")?)?;
    sys_modules.set_item("vnpyrs.trader", m.getattr("trader")?)?;

    // Python::with_gil(|py| {
    //     let mypycode = PyModule::from_code(
    //         py,
    //         &CString::new(MYPYCODE).unwrap(),
    //         c_str!("mypycode.py"),
    //         c_str!("mypycode"),
    //     )
    //     .unwrap();

    //     let vnpyrs_mod = PyModule::import(py, "vnpyrs").unwrap();
    //     let bar_generator_class = mypyclass.getattr("BarGenerator").unwrap();
    //     vnpyrs_mod
    //         .setattr("BarGenerator", bar_generator_class)
    //         .unwrap();
    //     let array_manager_class = mypyclass.getattr("ArrayManager").unwrap();
    //     vnpyrs_mod
    //         .setattr("ArrayManager", array_manager_class)
    //         .unwrap();
    // });

    Ok(())
}
