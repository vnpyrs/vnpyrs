use std::{collections::HashMap, sync::LazyLock};

use chrono::{DateTime, Duration};
use chrono_tz::Tz;
use pyo3::prelude::*;
use strum::{Display, EnumString};

use crate::trader::constant::{Direction, Interval, Offset_};

pub const STOPORDER_PREFIX: &'static str = "STOP";

#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StopOrderStatus {
    WAITING,
    CANCELLED,
    TRIGGERED,
}

#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EngineType {
    LIVE,
    BACKTESTING,
}

#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq, EnumString, Display)]
pub enum BacktestingMode {
    #[strum(serialize = "BAR")]
    BAR = 1,
    #[strum(serialize = "TICK")]
    TICK = 2,
}

#[pymethods]
impl BacktestingMode {
    fn __str__(&self) -> String {
        self.to_string()
    }
}

#[pyclass(get_all)]
#[derive(Clone)]
pub struct StopOrder {
    pub vt_symbol: String,
    pub direction: Direction,
    pub offset: Offset_,
    pub price: f64,
    pub volume: f64,
    pub stop_orderid: String,
    pub strategy_name: String,
    pub datetime: DateTime<Tz>,
    pub lock: bool,
    pub net: bool,
    pub vt_orderids: Vec<String>,
    pub status: StopOrderStatus,
}

pub static INTERVAL_DELTA_MAP: LazyLock<HashMap<Interval, Duration>> = LazyLock::new(|| {
    vec![
        (Interval::TICK, Duration::milliseconds(1)),
        (Interval::MINUTE, Duration::minutes(1)),
        (Interval::HOUR, Duration::hours(1)),
        (Interval::DAILY, Duration::days(1)),
    ]
    .into_iter()
    .collect()
});
