/*!General constant enums used in the trading platform. */
use pyo3::prelude::*;
use strum::{Display, EnumString};

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Display)]
pub enum Direction {
    LONG,
    SHORT,
    NET,
}

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Display)]
pub enum Offset_ {
    #[strum(serialize = "")]
    NONE,
    #[strum(serialize = "开")]
    OPEN,
    #[strum(serialize = "平")]
    CLOSE,
    #[strum(serialize = "平今")]
    CLOSETODAY,
    #[strum(serialize = "平昨")]
    CLOSEYESTERDAY,
}

#[pymethods]
impl Offset_ {
    pub fn __str__(&self) -> String {
        self.to_string()
    }
}

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Display, Hash)]
pub enum Status {
    #[strum(serialize = "提交中")]
    SUBMITTING,
    #[strum(serialize = "未成交")]
    NOTTRADED,
    #[strum(serialize = "部分成交")]
    PARTTRADED,
    #[strum(serialize = "全部成交")]
    ALLTRADED,
    #[strum(serialize = "已撤销")]
    CANCELLED,
    #[strum(serialize = "拒单")]
    REJECTED,
}

#[pymethods]
impl Status {
    pub fn __str__(&self) -> String {
        self.to_string()
    }
}

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Display)]
pub enum Product {
    EQUITY,
    FUTURES,
    OPTION,
    INDEX,
    FOREX,
    SPOT,
    ETF,
    BOND,
    WARRANT,
    SPREAD,
    FUND,
    CFD,
    SWAP,
}

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Display)]
pub enum OrderType {
    #[strum(serialize = "限价")]
    LIMIT,
    #[strum(serialize = "市价")]
    MARKET,
    #[strum(serialize = "STOP")]
    STOP,
    #[strum(serialize = "FAK")]
    FAK,
    #[strum(serialize = "FOK")]
    FOK,
    #[strum(serialize = "询价")]
    RFQ,
}

#[pymethods]
impl OrderType {
    pub fn __str__(&self) -> String {
        self.to_string()
    }
}

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Display)]
pub enum OptionType {
    CALL,
    PUT,
}

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Display)]
pub enum Currency {
    USD,
    HKD,
    CNY,
    CAD,
}

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Display, Hash)]
pub enum Interval {
    #[strum(serialize = "1m")]
    MINUTE,
    #[strum(serialize = "1h")]
    HOUR,
    #[strum(serialize = "d")]
    DAILY,
    #[strum(serialize = "w")]
    WEEKLY,
    #[strum(serialize = "tick")]
    TICK,
}

#[pymethods]
impl Interval {
    fn __str__(&self) -> String {
        self.to_string()
    }
}

#[pymodule]
pub fn constant(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Direction>()?;
    m.add_class::<Offset_>()?;
    m.add_class::<Status>()?;
    m.add_class::<Product>()?;
    m.add_class::<OrderType>()?;
    m.add_class::<OptionType>()?;
    m.add_class::<Currency>()?;
    m.add_class::<Interval>()?;
    Ok(())
}
