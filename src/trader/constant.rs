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
    NONE,
    OPEN,
    CLOSE,
    CLOSETODAY,
    CLOSEYESTERDAY,
}

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Display, Hash)]
pub enum Status {
    SUBMITTING,
    NOTTRADED,
    PARTTRADED,
    ALLTRADED,
    CANCELLED,
    REJECTED,
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
    LIMIT,
    MARKET,
    STOP,
    FAK,
    FOK,
    RFQ,
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
