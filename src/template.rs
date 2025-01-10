// use std::str::FromStr;

// use crate::backtesting::BacktestingEngine;
// use crate::trader::constant::Interval;
// use pyo3::{prelude::*, types::PyDict};

// #[pyclass(subclass, dict)]
// pub struct CtaTemplate {
//     pub cta_engine: Py<BacktestingEngine>,
//     pub strategy_name: String,
//     pub vt_symbol: String,
//     pub inited: bool,
//     pub trading: bool,
//     pub pos: i64,
// }

// #[pymethods]
// impl CtaTemplate {
//     #[classattr]
//     fn author() -> String {
//         "".to_string()
//     }

//     #[new]
//     pub fn __new__(
//         _py: Python<'_>,
//         cta_engine: PyRef<'_, BacktestingEngine>,
//         strategy_name: String,
//         vt_symbol: String,
//         _setting: Bound<'_, PyDict>,
//     ) -> Self {
//         CtaTemplate {
//             cta_engine: cta_engine.into(),
//             strategy_name,
//             vt_symbol,
//             inited: false,
//             trading: false,
//             pos: 0,
//         }
//     }

//     pub fn __init__(
//         slf: Bound<'_, Self>,
//         _py: Python<'_>,
//         _cta_engine: PyObject,
//         _strategy_name: String,
//         _vt_symbol: String,
//         setting: Bound<'_, PyDict>,
//     ) {
//         let mut variables: Vec<String> = slf.getattr("variables").unwrap().extract().unwrap();
//         variables.extend(vec![
//             "inited".to_string(),
//             "trading".to_string(),
//             "pos".to_string(),
//         ]);
//         slf.setattr("variables", variables).unwrap();
//         slf.call_method1("update_setting", (setting,)).unwrap();
//     }

//     pub fn update_setting(slf: Bound<'_, Self>, py: Python<'_>, setting: Bound<'_, PyDict>) {
//         let parameters: Vec<String> = slf.getattr("parameters").unwrap().extract().unwrap();
//         let slf = slf.into_pyobject(py).unwrap();
//         for name in &parameters {
//             if setting.contains(name).unwrap() {
//                 slf.setattr(name, setting.get_item(name).unwrap()).unwrap();
//             }
//         }
//     }

//     pub fn write_log(&self, py: Python<'_>, msg: &str) {
//         self.cta_engine
//             .getattr(py, "write_log")
//             .unwrap()
//             .call1(py, (msg,))
//             .unwrap();
//     }

//     #[pyo3(signature = (days,interval="1m".to_string(),callback=None,use_database=false))]
//     pub fn load_bar(
//         slf: PyRef<'_, Self>,
//         py: Python<'_>,
//         days: i64,
//         interval: String,
//         mut callback: Option<PyObject>,
//         use_database: bool,
//     ) {
//         let vt_symbol = slf.vt_symbol.clone();
//         let cta_engine = slf.cta_engine.clone_ref(py).bind(py);
//         let py_slf = slf.into_pyobject(py).unwrap();
//         if callback.is_none() {
//             callback = Some(py_slf.getattr("on_bar").unwrap().unbind());
//         }
//         let bars = cta_engine
//             .call_method1(
//                 py,
//                 "load_bar",
//                 (
//                     vt_symbol,
//                     days,
//                     Interval::from_str(&interval).unwrap(),
//                     callback,
//                     use_database,
//                 ),
//             )
//             .unwrap();
//     }
// }
