use std::io::{BufWriter, Seek, Write};

use pyo3::{prelude::*, types::PyList};

const VERSION: u64 = 0;

#[pyclass]
pub struct CandleChartDialog {
    updated: bool,
}

#[pymethods]
impl CandleChartDialog {
    #[new]
    pub fn __new__() -> Self {
        CandleChartDialog { updated: false }
    }

    pub fn update_history(&mut self, _py: Python<'_>, history: Bound<'_, PyList>) -> PyResult<()> {
        let home_path = get_home_path();
        std::fs::create_dir_all(format!("{}/vnpyrs", home_path))?;
        let mut writer = BufWriter::new(std::fs::File::create(format!(
            "{}/vnpyrs/history.dat",
            home_path
        ))?);
        writer.write_all(&VERSION.to_le_bytes())?;
        writer.write_all(&0f64.to_le_bytes())?;
        let mut count: u64 = 0;
        for bar in history.iter() {
            let timestamp: f64 = bar
                .getattr("datetime")?
                .call_method0("timestamp")?
                .extract()?;
            let open_price: f64 = bar.getattr("open_price")?.extract()?;
            let high_price: f64 = bar.getattr("high_price")?.extract()?;
            let low_price: f64 = bar.getattr("low_price")?.extract()?;
            let close_price: f64 = bar.getattr("close_price")?.extract()?;
            let volume: f64 = bar.getattr("volume")?.extract()?;
            writer.write_all(&(timestamp as u64).to_le_bytes())?;
            writer.write_all(&open_price.to_le_bytes())?;
            writer.write_all(&high_price.to_le_bytes())?;
            writer.write_all(&low_price.to_le_bytes())?;
            writer.write_all(&close_price.to_le_bytes())?;
            writer.write_all(&volume.to_le_bytes())?;
            count += 1;
        }
        writer.seek(std::io::SeekFrom::Start(8))?;
        writer.write_all(&count.to_le_bytes())?;
        println!("共{}根k线", count);
        Ok(())
    }

    pub fn update_trades(&mut self, _py: Python<'_>, trades: Bound<'_, PyList>) -> PyResult<()> {
        let home_path = get_home_path();
        std::fs::create_dir_all(format!("{}/vnpyrs", home_path))?;
        let mut writer = BufWriter::new(std::fs::File::create(format!(
            "{}/vnpyrs/trades.dat",
            home_path
        ))?);
        writer.write_all(&VERSION.to_le_bytes())?;
        writer.write_all(&0f64.to_le_bytes())?;
        let mut count: u64 = 0;
        for trade in trades.iter() {
            let timestamp: f64 = trade
                .getattr("datetime")?
                .call_method0("timestamp")?
                .extract()?;
            let direction: String = trade
                .getattr("direction")?
                .call_method0("__str__")?
                .extract()?;
            let direction: u8 = match direction.as_str() {
                "Direction.LONG" => 1,
                "Direction.SHORT" => 2,
                _ => 0,
            };
            let price: f64 = trade.getattr("price")?.extract()?;
            let volume: f64 = trade.getattr("volume")?.extract()?;
            writer.write_all(&(timestamp as u64).to_le_bytes())?;
            writer.write_all(&direction.to_le_bytes())?;
            writer.write_all(&price.to_le_bytes())?;
            writer.write_all(&volume.to_le_bytes())?;
            count += 1;
        }
        writer.seek(std::io::SeekFrom::Start(8))?;
        writer.write_all(&count.to_le_bytes())?;
        println!("共{}个交易", count);
        Ok(())
    }

    pub fn clear_data(&mut self, _py: Python<'_>) {}

    pub fn is_updated(&self) -> bool {
        self.updated
    }

    pub fn exec_(&mut self) {}
}

fn get_home_path() -> String {
    let win = std::env::var("USERPROFILE");
    let unix = std::env::var("HOME");
    if win.is_ok() {
        win.unwrap()
    } else if unix.is_ok() {
        unix.unwrap()
    } else {
        ".".to_string()
    }
}
