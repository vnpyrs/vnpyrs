use chrono::{DateTime, NaiveTime, Timelike};
use chrono_tz::Tz;
use pyo3::prelude::*;
use rust_decimal::prelude::*;
use std::{env, path::PathBuf, sync::LazyLock};

use super::{
    constant::Interval,
    object::{BarData, TickData},
};

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
    let value: Decimal = Decimal::from_str(&value.to_string()).unwrap();
    let target: Decimal = Decimal::from_str(&target.to_string()).unwrap();
    ((value / target).round() * target)
        .to_string()
        .parse()
        .unwrap()
}

/// For:
/// 1. generating 1 minute bar data from tick data
/// 2. generating x minute bar/x hour bar data from 1 minute data
/// Notice:
/// 1. for x minute bar, x must be able to divide 60: 2, 3, 5, 6, 10, 15, 20, 30
/// 2. for x hour bar, x can be any number
#[pyclass]
pub struct BarGenerator {
    bar: Option<BarData>,
    on_bar: PyObject,

    interval: Interval,
    interval_count: u32,

    hour_bar: Option<BarData>,
    daily_bar: Option<BarData>,

    window: u32,
    window_bar: Option<BarData>,
    on_window_bar: Option<PyObject>,

    last_tick: Option<TickData>,

    daily_end: Option<NaiveTime>,
}

#[pymethods]
impl BarGenerator {
    #[new]
    #[pyo3(signature=(on_bar,window=0,on_window_bar=None,interval=Interval::MINUTE,daily_end=None))]
    pub fn __new__(
        on_bar: PyObject,
        window: u32,
        on_window_bar: Option<PyObject>,
        interval: Interval,
        daily_end: Option<NaiveTime>,
    ) -> Self {
        let ret = BarGenerator {
            bar: None,
            on_bar: on_bar,

            interval: interval,
            interval_count: 0,

            hour_bar: None,
            daily_bar: None,

            window: window,
            window_bar: None,
            on_window_bar: on_window_bar,

            last_tick: None,

            daily_end: daily_end,
        };
        if interval == Interval::DAILY && daily_end.is_none() {
            panic!("合成日K线必须传入每日收盘时间");
        }
        ret
    }

    /// Update new tick data into generator.
    pub fn update_tick(&mut self, py: Python<'_>, tick: TickData) -> PyResult<()> {
        let mut new_minute: bool = false;

        // Filter tick data with 0 last price
        if tick.last_price == 0.0 {
            return Ok(());
        }

        if self.bar.is_none() {
            new_minute = true;
        } else if (self.bar.as_mut().unwrap().datetime.minute() != tick.datetime.minute())
            || (self.bar.as_mut().unwrap().datetime.hour() != tick.datetime.hour())
        {
            self.bar.as_mut().unwrap().datetime = self
                .bar
                .as_ref()
                .unwrap()
                .datetime
                .with_second(0)
                .unwrap()
                .with_nanosecond(0)
                .unwrap();
            self.on_bar.call1(py, (self.bar.clone(),))?;

            new_minute = true;
        }
        if new_minute {
            let mut bar = BarData::default();
            bar.symbol = tick.symbol.clone();
            bar.exchange = tick.exchange.clone();
            bar.interval = Interval::MINUTE;
            bar.datetime = tick.datetime;
            bar.gateway_name = tick.gateway_name;
            bar.open_price = tick.last_price;
            bar.high_price = tick.last_price;
            bar.low_price = tick.last_price;
            bar.close_price = tick.last_price;
            bar.open_interest = tick.open_interest;
            self.bar = Some(bar);
        } else {
            self.bar.as_mut().unwrap().high_price =
                f64::max(self.bar.as_mut().unwrap().high_price, tick.last_price);
            if tick.high_price > self.last_tick.as_mut().unwrap().high_price {
                self.bar.as_mut().unwrap().high_price =
                    f64::max(self.bar.as_mut().unwrap().high_price, tick.high_price);
            }
            self.bar.as_mut().unwrap().low_price =
                f64::min(self.bar.as_mut().unwrap().low_price, tick.last_price);
            if tick.low_price < self.last_tick.as_mut().unwrap().low_price {
                self.bar.as_mut().unwrap().low_price =
                    f64::min(self.bar.as_mut().unwrap().low_price, tick.low_price);
            }

            self.bar.as_mut().unwrap().close_price = tick.last_price;
            self.bar.as_mut().unwrap().open_interest = tick.open_interest;
            self.bar.as_mut().unwrap().datetime = tick.datetime;
        }
        if self.last_tick.is_some() {
            let volume_change: f64 = tick.volume - self.last_tick.as_mut().unwrap().volume;
            self.bar.as_mut().unwrap().volume += f64::max(volume_change, 0.0);

            let turnover_change: f64 = tick.turnover - self.last_tick.as_mut().unwrap().turnover;
            self.bar.as_mut().unwrap().turnover += f64::max(turnover_change, 0.0);
        }
        self.last_tick.replace(tick);
        Ok(())
    }

    /// Update 1 minute bar into generator
    pub fn update_bar(&mut self, py: Python<'_>, bar: BarData) -> PyResult<()> {
        if self.interval == Interval::MINUTE {
            self.update_bar_minute_window(py, bar)?;
        } else if self.interval == Interval::HOUR {
            self.update_bar_hour_window(py, bar)?;
        } else {
            self.update_bar_daily_window(py, bar)?;
        }
        Ok(())
    }

    pub fn update_bar_minute_window(&mut self, py: Python<'_>, bar: BarData) -> PyResult<()> {
        // If not inited, create window bar object
        if self.window_bar.is_none() {
            let dt: DateTime<Tz> = bar
                .datetime
                .with_second(0)
                .unwrap()
                .with_nanosecond(0)
                .unwrap();
            let mut new_bar = BarData::default();
            new_bar.symbol = bar.symbol;
            new_bar.exchange = bar.exchange;
            new_bar.datetime = dt;
            new_bar.gateway_name = bar.gateway_name;
            new_bar.open_price = bar.open_price;
            new_bar.high_price = bar.high_price;
            new_bar.low_price = bar.low_price;
            self.window_bar = Some(new_bar);
        }
        // Otherwise, update high/low price into window bar
        else {
            self.window_bar.as_mut().unwrap().high_price =
                f64::max(self.window_bar.as_ref().unwrap().high_price, bar.high_price);
            self.window_bar.as_mut().unwrap().low_price =
                f64::min(self.window_bar.as_ref().unwrap().low_price, bar.low_price);
        }
        // Update close price/volume/turnover into window bar
        self.window_bar.as_mut().unwrap().close_price = bar.close_price;
        self.window_bar.as_mut().unwrap().volume += bar.volume;
        self.window_bar.as_mut().unwrap().turnover += bar.turnover;
        self.window_bar.as_mut().unwrap().open_interest = bar.open_interest;

        // Check if window bar completed
        if (bar.datetime.minute() + 1) % self.window == 0 {
            self.on_window_bar
                .as_ref()
                .unwrap()
                .call1(py, (self.window_bar.clone(),))?;
            self.window_bar = None;
        }
        Ok(())
    }

    pub fn update_bar_hour_window(&mut self, py: Python<'_>, bar: BarData) -> PyResult<()> {
        // If not inited, create window bar object
        if self.hour_bar.is_none() {
            let dt: DateTime<Tz> = bar
                .datetime
                .with_minute(0)
                .unwrap()
                .with_second(0)
                .unwrap()
                .with_nanosecond(0)
                .unwrap();
            let mut new_bar = BarData::default();
            new_bar.symbol = bar.symbol;
            new_bar.exchange = bar.exchange;
            new_bar.datetime = dt;
            new_bar.gateway_name = bar.gateway_name;
            new_bar.open_price = bar.open_price;
            new_bar.high_price = bar.high_price;
            new_bar.low_price = bar.low_price;
            new_bar.close_price = bar.close_price;
            new_bar.volume = bar.volume;
            new_bar.turnover = bar.turnover;
            new_bar.open_interest = bar.open_interest;
            self.hour_bar = Some(new_bar);
            return Ok(());
        }

        let mut finished_bar: Option<BarData> = None;

        // If minute is 59, update minute bar into window bar and push
        if bar.datetime.minute() == 59 {
            self.hour_bar.as_mut().unwrap().high_price =
                f64::max(self.hour_bar.as_ref().unwrap().high_price, bar.high_price);
            self.hour_bar.as_mut().unwrap().low_price =
                f64::min(self.hour_bar.as_ref().unwrap().low_price, bar.low_price);

            self.hour_bar.as_mut().unwrap().close_price = bar.close_price;
            self.hour_bar.as_mut().unwrap().volume += bar.volume;
            self.hour_bar.as_mut().unwrap().turnover += bar.turnover;
            self.hour_bar.as_mut().unwrap().open_interest = bar.open_interest;

            finished_bar = self.hour_bar.clone();
            self.hour_bar = None;
        }
        // If minute bar of new hour, then push existing window bar
        else if bar.datetime.hour() != self.hour_bar.as_ref().unwrap().datetime.hour() {
            finished_bar = self.hour_bar.clone();

            let dt: DateTime<Tz> = bar
                .datetime
                .with_minute(0)
                .unwrap()
                .with_second(0)
                .unwrap()
                .with_nanosecond(0)
                .unwrap();
            let mut new_bar = BarData::default();
            new_bar.symbol = bar.symbol;
            new_bar.exchange = bar.exchange;
            new_bar.datetime = dt;
            new_bar.gateway_name = bar.gateway_name;
            new_bar.open_price = bar.open_price;
            new_bar.high_price = bar.high_price;
            new_bar.low_price = bar.low_price;
            new_bar.close_price = bar.close_price;
            new_bar.volume = bar.volume;
            new_bar.turnover = bar.turnover;
            new_bar.open_interest = bar.open_interest;
            self.hour_bar = Some(new_bar)
        }
        // Otherwise only update minute bar
        else {
            self.hour_bar.as_mut().unwrap().high_price =
                f64::max(self.hour_bar.as_ref().unwrap().high_price, bar.high_price);
            self.hour_bar.as_mut().unwrap().low_price =
                f64::min(self.hour_bar.as_ref().unwrap().low_price, bar.low_price);

            self.hour_bar.as_mut().unwrap().close_price = bar.close_price;
            self.hour_bar.as_mut().unwrap().volume += bar.volume;
            self.hour_bar.as_mut().unwrap().turnover += bar.turnover;
            self.hour_bar.as_mut().unwrap().open_interest = bar.open_interest;
        }

        // Push finished window bar
        if finished_bar.is_some() {
            self.on_hour_bar(py, finished_bar.as_ref().unwrap().clone())?;
        }
        Ok(())
    }

    pub fn on_hour_bar(&mut self, py: Python<'_>, bar: BarData) -> PyResult<()> {
        if self.window == 1 {
            self.on_window_bar
                .as_ref()
                .unwrap()
                .call1(py, (bar.clone(),))?;
        } else {
            if self.window_bar.is_none() {
                let mut new_bar = BarData::default();
                new_bar.symbol = bar.symbol;
                new_bar.exchange = bar.exchange;
                new_bar.datetime = bar.datetime;
                new_bar.gateway_name = bar.gateway_name;
                new_bar.open_price = bar.open_price;
                new_bar.high_price = bar.high_price;
                new_bar.low_price = bar.low_price;
                self.window_bar = Some(new_bar);
            } else {
                self.window_bar.as_mut().unwrap().high_price =
                    f64::max(self.window_bar.as_ref().unwrap().high_price, bar.high_price);
                self.window_bar.as_mut().unwrap().low_price =
                    f64::min(self.window_bar.as_ref().unwrap().low_price, bar.low_price);
            }

            self.window_bar.as_mut().unwrap().close_price = bar.close_price;
            self.window_bar.as_mut().unwrap().volume += bar.volume;
            self.window_bar.as_mut().unwrap().turnover += bar.turnover;
            self.window_bar.as_mut().unwrap().open_interest = bar.open_interest;

            self.interval_count += 1;
            if self.interval_count % self.window == 0 {
                self.interval_count = 0;
                self.on_window_bar
                    .as_ref()
                    .unwrap()
                    .call1(py, (self.window_bar.clone(),))?;
                self.window_bar = None;
            }
        }
        Ok(())
    }

    pub fn update_bar_daily_window(&mut self, py: Python<'_>, bar: BarData) -> PyResult<()> {
        // If not inited, create daily bar object
        if self.daily_bar.is_none() {
            let mut new_bar = BarData::default();
            new_bar.symbol = bar.symbol;
            new_bar.exchange = bar.exchange;
            new_bar.datetime = bar.datetime;
            new_bar.gateway_name = bar.gateway_name;
            new_bar.open_price = bar.open_price;
            new_bar.high_price = bar.high_price;
            new_bar.low_price = bar.low_price;
            self.daily_bar = Some(new_bar);
        }
        // Otherwise, update high/low price into daily bar
        else {
            self.daily_bar.as_mut().unwrap().high_price =
                f64::max(self.daily_bar.as_ref().unwrap().high_price, bar.high_price);
            self.daily_bar.as_mut().unwrap().low_price =
                f64::min(self.daily_bar.as_ref().unwrap().low_price, bar.low_price);
        }

        // Update close price/volume/turnover into daily bar
        self.daily_bar.as_mut().unwrap().close_price = bar.close_price;
        self.daily_bar.as_mut().unwrap().volume += bar.volume;
        self.daily_bar.as_mut().unwrap().turnover += bar.turnover;
        self.daily_bar.as_mut().unwrap().open_interest = bar.open_interest;

        // Check if daily bar completed
        if &bar.datetime.time() == self.daily_end.as_ref().unwrap() {
            self.daily_bar.as_mut().unwrap().datetime = bar
                .datetime
                .with_hour(0)
                .unwrap()
                .with_minute(0)
                .unwrap()
                .with_second(0)
                .unwrap()
                .with_nanosecond(0)
                .unwrap();
            self.on_window_bar
                .as_ref()
                .unwrap()
                .call1(py, (self.daily_bar.clone(),))?;

            self.daily_bar = None;
        }
        Ok(())
    }

    ///Generate the bar data and call callback immediately.
    pub fn generate(&mut self, py: Python<'_>) -> PyResult<Option<BarData>> {
        let mut bar = self.bar.clone();

        if self.bar.is_some() {
            bar.as_mut().unwrap().datetime = bar
                .as_ref()
                .unwrap()
                .datetime
                .with_second(0)
                .unwrap()
                .with_nanosecond(0)
                .unwrap();
            self.on_bar.call1(py, (bar.clone(),))?;
        }

        self.bar = None;
        Ok(bar)
    }
}

#[pymodule]
pub fn utility(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<BarGenerator>()?;
    Ok(())
}
