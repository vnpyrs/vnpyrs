use pyo3::prelude::*;
use rust_decimal::prelude::*;
use std::{env, path::PathBuf, sync::LazyLock};

pub const MYPYCODE: &str = r#"
"#;

pub fn extract_vt_symbol(vt_symbol: &str) -> (String, String) {
    let vec_str: Vec<&str> = vt_symbol.rsplitn(2, ".").collect();
    let (symbol, exchange) = (vec_str[1], vec_str[0]);
    return (symbol.to_string(), exchange.to_string());
}

fn _get_trader_dir(temp_name: &str) -> (PathBuf, PathBuf) {
    let cwd = env::current_dir().unwrap();
    let mut temp_path = cwd.join(temp_name);

    if temp_path.exists() {
        return (cwd, temp_path);
    }

    let home_path: PathBuf = Python::with_gil(|py| {
        let pathlib = PyModule::import(py, "pathlib").unwrap();
        pathlib
            .getattr("Path")
            .unwrap()
            .call_method0("home")
            .unwrap()
            .extract()
            .unwrap()
    });
    temp_path = home_path.join(temp_name);

    if !temp_path.exists() {
        std::fs::create_dir_all(&temp_path).unwrap();
    }

    (home_path, temp_path)
}

static COMBIN_DIRS: LazyLock<(PathBuf, PathBuf)> = LazyLock::new(|| _get_trader_dir(".vntrader"));
pub fn get_file_path(filename: &str) -> PathBuf {
    COMBIN_DIRS.1.join(filename)
}

pub fn round_to(value: f64, target: f64) -> f64 {
    let value: Decimal = Decimal::from_f64(value).unwrap();
    let target: Decimal = Decimal::from_f64(target).unwrap();
    ((value / target).round() * target)
        .to_string()
        .parse()
        .unwrap()
}
