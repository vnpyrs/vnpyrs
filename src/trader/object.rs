/*！Basic data structure used for general trading function in the trading platform.*/

use pyo3::prelude::*;
use std::{collections::HashSet, sync::LazyLock};

use super::{
    constant::{Direction, Interval, Offset_, OrderType, Status},
    database::DB_TZ,
};
use chrono::{DateTime, NaiveDateTime};
use chrono_tz::Tz;

pub static ACTIVE_STATUSES: LazyLock<HashSet<Status>> = LazyLock::new(|| {
    vec![Status::SUBMITTING, Status::NOTTRADED, Status::PARTTRADED]
        .into_iter()
        .collect()
});

#[pyclass(get_all)]
#[derive(Debug, Clone)]

pub struct TickData {
    pub gateway_name: &'static str,

    pub symbol: String,
    pub exchange: String,
    pub datetime: DateTime<Tz>,

    pub name: String,
    pub volume: f64,
    pub turnover: f64,
    pub open_interest: f64,
    pub last_price: f64,
    pub last_volume: f64,
    pub limit_up: f64,
    pub limit_down: f64,

    pub open_price: f64,
    pub high_price: f64,
    pub low_price: f64,
    pub pre_close: f64,

    pub bid_price_1: f64,
    pub bid_price_2: f64,
    pub bid_price_3: f64,
    pub bid_price_4: f64,
    pub bid_price_5: f64,

    pub ask_price_1: f64,
    pub ask_price_2: f64,
    pub ask_price_3: f64,
    pub ask_price_4: f64,
    pub ask_price_5: f64,

    pub bid_volume_1: f64,
    pub bid_volume_2: f64,
    pub bid_volume_3: f64,
    pub bid_volume_4: f64,
    pub bid_volume_5: f64,

    pub ask_volume_1: f64,
    pub ask_volume_2: f64,
    pub ask_volume_3: f64,
    pub ask_volume_4: f64,
    pub ask_volume_5: f64,

    pub localtime: NaiveDateTime,
}

#[pymethods]
impl TickData {
    #[getter]
    pub fn vt_symbol(&self) -> String {
        format!("{}.{}", self.symbol, self.exchange.to_string())
    }
}

#[pyclass(get_all)]
#[derive(Debug, Clone)]
pub struct BarData {
    pub gateway_name: &'static str,

    pub symbol: String,
    pub exchange: String,
    pub datetime: DateTime<Tz>,

    pub interval: Interval,
    pub volume: f64,
    pub turnover: f64,
    pub open_interest: f64,
    pub open_price: f64,
    pub high_price: f64,
    pub low_price: f64,
    pub close_price: f64,
}

impl Default for BarData {
    fn default() -> Self {
        Self {
            gateway_name: Default::default(),
            symbol: Default::default(),
            exchange: Default::default(),
            datetime: NaiveDateTime::default()
                .and_local_timezone(DB_TZ.clone())
                .unwrap(),
            interval: Interval::MINUTE,
            volume: Default::default(),
            turnover: Default::default(),
            open_interest: Default::default(),
            open_price: Default::default(),
            high_price: Default::default(),
            low_price: Default::default(),
            close_price: Default::default(),
        }
    }
}

#[pymethods]
impl BarData {
    #[getter]
    pub fn vt_symbol(&self) -> String {
        format!("{}.{}", self.symbol, self.exchange.to_string())
    }
}

#[derive(Debug, Clone)]
pub enum MixData {
    TickData(TickData),
    BarData(BarData),
}

#[pyclass(get_all)]
#[derive(Debug, Clone)]
pub struct OrderData {
    pub gateway_name: &'static str,

    pub symbol: String,
    pub exchange: String,
    pub orderid: String,

    pub r#type: OrderType,
    pub direction: Direction,
    pub offset: Offset_,
    pub price: f64,
    pub volume: f64,
    pub traded: f64,
    pub status: Status,
    pub datetime: DateTime<Tz>,
    pub reference: String,
}

#[pymethods]
impl OrderData {
    #[getter]
    pub fn vt_symbol(&self) -> String {
        format!("{}.{}", self.symbol, self.exchange)
    }

    #[getter]
    pub fn vt_orderid(&self) -> String {
        format!("{}.{}", self.gateway_name, self.orderid)
    }

    pub fn is_active(&self) -> bool {
        ACTIVE_STATUSES.contains(&self.status)
    }
}

#[pyclass(get_all)]
#[derive(Debug, Clone)]
pub struct TradeData {
    pub gateway_name: &'static str,

    pub symbol: String,
    pub exchange: String,
    pub orderid: String,
    pub tradeid: String,
    pub direction: Direction,

    pub offset: Offset_,
    pub price: f64,
    pub volume: f64,
    pub datetime: DateTime<Tz>,
}

#[pymethods]
impl TradeData {
    #[getter]
    pub fn vt_symbol(&self) -> String {
        format!("{}.{}", self.symbol, self.exchange.to_string())
    }

    #[getter]
    pub fn vt_orderid(&self) -> String {
        format!("{}.{}", self.gateway_name, self.orderid)
    }

    #[getter]
    pub fn vt_tradeid(&self) -> String {
        format!("{}.{}", self.gateway_name, self.tradeid)
    }
}

#[pymodule]
pub fn object(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<BarData>()?;
    m.add_class::<TickData>()?;
    m.add_class::<OrderData>()?;
    m.add_class::<TradeData>()?;
    Ok(())
}
