pub mod base;

use std::{
    collections::{BTreeMap, LinkedList},
    ops::Deref,
    str::FromStr,
    sync::{Arc, Mutex},
};

pub use base::BacktestingMode;

use base::{EngineType, StopOrder, StopOrderStatus, INTERVAL_DELTA_MAP, STOPORDER_PREFIX};
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, TimeDelta, TimeZone, Timelike};
use chrono_tz::Tz;
use pyo3::{
    prelude::*,
    types::{PyDict, PyList},
};

use crate::trader::{
    constant::{Direction, Interval, Offset_, OrderType, Status},
    database::get_database,
    object::{BarData, MixData, OrderData, TickData, TradeData},
    utility::{extract_vt_symbol, round_to},
};

static GLOBAL_HISTORY_DATA: Mutex<LinkedList<MixData>> = Mutex::new(LinkedList::new());

#[pyclass]
pub struct BacktestingEngine {
    #[pyo3(get, set)]
    vt_symbol: String,
    symbol: String,
    exchange: String,
    #[pyo3(get, set)]
    start: NaiveDateTime,
    #[pyo3(get, set)]
    end: NaiveDateTime,
    #[pyo3(get, set)]
    rate: f64,
    #[pyo3(get, set)]
    slippage: f64,
    #[pyo3(get, set)]
    size: f64,
    #[pyo3(get, set)]
    pricetick: f64,
    #[pyo3(get, set)]
    capital: f64,
    #[pyo3(get, set)]
    risk_free: f64,
    #[pyo3(get, set)]
    annual_days: i64,
    #[pyo3(get, set)]
    half_life: i64,
    #[pyo3(get, set)]
    mode: BacktestingMode,

    #[pyo3(get, set)]
    strategy_class: Option<PyObject>,
    #[pyo3(get, set)]
    strategy: Option<PyObject>,
    tick: Mutex<Option<TickData>>,
    bar: Mutex<Option<BarData>>,
    datetime: Mutex<Option<DateTime<Tz>>>,

    #[pyo3(get, set)]
    interval: Option<Interval>,
    _days: i64,
    history_data: LinkedList<MixData>,

    stop_order_count: Mutex<i64>,
    stop_orders: Mutex<BTreeMap<String, Arc<Mutex<StopOrder>>>>,
    active_stop_orders: Mutex<BTreeMap<String, Arc<Mutex<StopOrder>>>>,

    limit_order_count: Mutex<i64>,
    limit_orders: Mutex<BTreeMap<String, Arc<Mutex<OrderData>>>>,
    active_limit_orders: Mutex<BTreeMap<String, Arc<Mutex<OrderData>>>>,

    trade_count: Mutex<i64>,
    trades: Mutex<BTreeMap<String, Arc<Mutex<TradeData>>>>,

    logs: Mutex<Vec<String>>,

    daily_results: Mutex<BTreeMap<NaiveDate, DailyResult>>,
    #[pyo3(get, set)]
    daily_df: Option<PyObject>,

    #[pyo3(get, set)]
    rs_use_global_data: bool,

    rs_pyfunc_output: Option<PyObject>,
}

#[pymethods]
impl BacktestingEngine {
    #[classattr]
    fn engine_type() -> EngineType {
        EngineType::BACKTESTING
    }
    #[classattr]
    fn gateway_name() -> &'static str {
        "BACKTESTING"
    }

    #[new]
    #[allow(unexpected_cfgs)]
    pub fn __new__() -> Self {
        #[cfg(Py_LIMITED_API)]
        {
            println!("您正在使用通用版，兼容Python3.7以上所有版本，但是会降低运行性能。建议拉取源代码自行编译，或下载针对特定Python版本的whl包并通过pip安装");
        }
        BacktestingEngine {
            vt_symbol: "".to_string(),
            symbol: "".to_string(),
            exchange: "".to_string(),
            start: NaiveDateTime::default(),
            end: NaiveDateTime::default(),
            rate: 0.0,
            slippage: 0.0,
            size: 1.0,
            pricetick: 0.0,
            capital: 1_000_000.0,
            risk_free: 0.0,
            annual_days: 240,
            half_life: 120,
            mode: BacktestingMode::BAR,

            strategy_class: None,
            strategy: None,
            tick: Mutex::new(None),
            bar: Mutex::new(None),
            datetime: Mutex::new(None),

            interval: None,
            _days: 0,
            history_data: LinkedList::new(),

            stop_order_count: Mutex::new(0),
            stop_orders: Mutex::new(BTreeMap::new()),
            active_stop_orders: Mutex::new(BTreeMap::new()),

            limit_order_count: Mutex::new(0),
            limit_orders: Mutex::new(BTreeMap::new()),
            active_limit_orders: Mutex::new(BTreeMap::new()),

            trade_count: Mutex::new(0),
            trades: Mutex::new(BTreeMap::new()),

            logs: Mutex::new(Vec::new()),

            daily_results: Mutex::new(BTreeMap::new()),
            daily_df: None,

            rs_use_global_data: false,
            rs_pyfunc_output: None,
        }
    }

    pub fn clear_data(&mut self) {
        self.strategy.take();
        self.tick.lock().unwrap().take();
        self.bar.lock().unwrap().take();
        self.datetime.lock().unwrap().take();

        *self.stop_order_count.lock().unwrap() = 0;
        self.stop_orders.lock().unwrap().clear();
        self.active_stop_orders.lock().unwrap().clear();

        *self.limit_order_count.lock().unwrap() = 0;
        self.limit_orders.lock().unwrap().clear();
        self.active_limit_orders.lock().unwrap().clear();

        *self.trade_count.lock().unwrap() = 0;
        self.trades.lock().unwrap().clear();

        self.logs.lock().unwrap().clear();
        self.daily_results.lock().unwrap().clear();
    }

    #[pyo3(signature = (vt_symbol,interval,start,rate,slippage,size,pricetick,capital,end=NaiveDateTime::default(),mode="BAR",risk_free=0.0,annual_days=240,half_life=120))]
    pub fn set_parameters(
        &mut self,
        vt_symbol: &str,
        interval: &str,
        start: NaiveDateTime,
        rate: f64,
        slippage: f64,
        size: f64,
        pricetick: f64,
        capital: f64,
        mut end: NaiveDateTime,
        mode: &str,
        risk_free: f64,
        annual_days: i64,
        half_life: i64,
    ) {
        self.mode = BacktestingMode::from_str(mode).expect("mode字段只能是BAR,TICK中的一个");
        self.vt_symbol = vt_symbol.to_string();
        self.interval =
            Some(Interval::from_str(interval).expect("interval字段只能是1m,1h,d,w,tick中的一个"));
        self.rate = rate;
        self.slippage = slippage;
        self.size = size;
        self.pricetick = pricetick;
        self.start = start;

        let v: Vec<&str> = vt_symbol.split(".").collect();
        self.symbol = v[0].to_string();
        self.exchange = v[1].to_string();

        self.capital = capital;

        if end == NaiveDateTime::default() {
            end = Local::now().naive_local();
        }
        self.end = Local
            .from_local_datetime(&end)
            .unwrap()
            .with_hour(23)
            .unwrap()
            .with_minute(59)
            .unwrap()
            .with_second(59)
            .unwrap()
            .naive_local();

        self.risk_free = risk_free;
        self.annual_days = annual_days;
        self.half_life = half_life;
    }

    pub fn add_strategy(
        slf: PyRef<'_, Self>,
        py: Python<'_>,
        strategy_class: PyObject,
        setting: Bound<'_, PyDict>,
    ) -> PyResult<()> {
        let vt_symbol = slf.vt_symbol.clone();
        let slf = slf.into_pyobject(py)?;
        slf.setattr("strategy_class", Some(strategy_class.clone_ref(py)))?;
        slf.setattr(
            "strategy",
            strategy_class.call1(
                py,
                (
                    slf.clone(),
                    strategy_class.getattr(py, "__name__")?,
                    vt_symbol,
                    setting,
                ),
            )?,
        )?;
        Ok(())
    }

    pub fn load_data(&mut self, py: Python<'_>) -> PyResult<()> {
        if self.rs_use_global_data {
            if !GLOBAL_HISTORY_DATA.lock().unwrap().is_empty() {
                return Ok(());
            }
        }
        self.output(py, "开始加载历史数据");
        if self.end == NaiveDateTime::default() {
            self.end = Local::now().naive_local();
        }
        if self.start >= self.end {
            self.output(py, "起始日期必须小于结束日期");
            return Ok(());
        }
        if self.rs_use_global_data {
        } else {
            self.history_data.clear(); // Clear previously loaded history data
        }

        // Load 30 days of data each time and allow for progress update
        let total_days = (self.end - self.start).num_days();
        let progress_days = i64::max(total_days / 10, 1);
        let progress_delta = TimeDelta::days(progress_days);
        let interval_delta = INTERVAL_DELTA_MAP
            .get(&self.interval.unwrap())
            .unwrap()
            .clone();

        let mut start = self.start;
        let mut end = self.start + progress_delta;
        let mut progress: f64 = 0.0;

        while start < self.end {
            py.check_signals()?;
            let progress_bar = "#".repeat((progress * 10.0 + 1.0) as usize);
            self.output(
                py,
                &format!(
                    "加载进度：{progress_bar} [{progress:.0}%]",
                    progress_bar = progress_bar,
                    progress = progress * 100.0
                ),
            );

            end = NaiveDateTime::min(end, self.end); // Make sure end time stays within set range

            if self.mode == BacktestingMode::BAR {
                let mut data: LinkedList<MixData> = load_bar_data(
                    &self.symbol,
                    &self.exchange,
                    self.interval.unwrap(),
                    start,
                    end,
                );
                if self.rs_use_global_data {
                    GLOBAL_HISTORY_DATA.lock().unwrap().append(&mut data);
                } else {
                    self.history_data.append(&mut data);
                }
            } else {
                let mut data: LinkedList<MixData> =
                    load_tick_data(&self.symbol, &self.exchange, start, end);
                if self.rs_use_global_data {
                    GLOBAL_HISTORY_DATA.lock().unwrap().append(&mut data);
                } else {
                    self.history_data.append(&mut data);
                }
            }

            progress += progress_days as f64 / total_days as f64;
            progress = f64::min(progress, 1.0);

            start = end + interval_delta;
            end += progress_delta
        }

        let len = if self.rs_use_global_data {
            GLOBAL_HISTORY_DATA.lock().unwrap().len()
        } else {
            self.history_data.len()
        };
        self.output(py, format!("历史数据加载完成，数据量：{}", len).as_str());
        Ok(())
    }

    pub fn run_backtesting(&self, py: Python<'_>) -> PyResult<()> {
        self.strategy
            .as_ref()
            .unwrap()
            .call_method0(py, "on_init")?;
        self.strategy
            .as_ref()
            .unwrap()
            .setattr(py, "inited", true)
            .unwrap();
        self.output(py, "策略初始化完成");

        self.strategy
            .as_ref()
            .unwrap()
            .call_method0(py, "on_start")
            .unwrap();
        self.strategy
            .as_ref()
            .unwrap()
            .setattr(py, "trading", true)
            .unwrap();
        self.output(py, "开始回放历史数据");

        let total_size: usize = if self.rs_use_global_data {
            GLOBAL_HISTORY_DATA.lock().unwrap().len()
        } else {
            self.history_data.len()
        };
        let batch_size: usize = (total_size / 10).max(1);

        let global_history_data_mutex;
        let data_iter;
        if self.rs_use_global_data {
            global_history_data_mutex = GLOBAL_HISTORY_DATA.lock().unwrap();
            data_iter = global_history_data_mutex.iter().enumerate();
        } else {
            data_iter = self.history_data.iter().enumerate();
        }
        for (i, item) in data_iter {
            py.check_signals()?;
            match item {
                MixData::BarData(bar_data) => {
                    self.new_bar(py, bar_data)?;
                }
                MixData::TickData(tick_data) => {
                    self.new_tick(py, tick_data)?;
                }
            }
            if i % batch_size == 0 {
                let ix = i / batch_size;
                let progress = (ix as f64 / 10.0).min(1.0);
                let progress_bar = "=".repeat(ix + 1);
                self.output(
                    py,
                    &format!("回放进度：{} [{:.0}%]", progress_bar, progress * 100.0),
                );
            }
        }
        self.strategy
            .as_ref()
            .unwrap()
            .call_method0(py, "on_stop")
            .unwrap();
        self.output(py, "历史数据回放结束");
        Ok(())
    }

    fn calculate_result(&mut self, py: Python<'_>) -> PyResult<PyObject> {
        self.output(py, "开始计算逐日盯市盈亏");

        if self.trades.lock().unwrap().len() == 0 {
            self.output(py, "回测成交记录为空");
        }

        // Add trade data into daily reuslt.
        for trade in self.trades.lock().unwrap().values() {
            let d = trade.lock().unwrap().datetime.naive_local().date();
            let mut daily_result_map = self.daily_results.lock().unwrap();
            let daily_result = daily_result_map.get_mut(&d).unwrap();
            daily_result.add_trade(trade.lock().unwrap().clone())
        }

        // Calculate daily result by iteration.
        let mut pre_close = 0.0;
        let mut start_pos = 0.0;

        for daily_result in self.daily_results.lock().unwrap().values_mut() {
            daily_result.calculate_pnl(pre_close, start_pos, self.size, self.rate, self.slippage);

            pre_close = daily_result.close_price;
            start_pos = daily_result.end_pos;
        }

        // Generate dataframe
        let mut date: Vec<NaiveDate> = Vec::new();
        let mut close_price = Vec::new();
        let mut pre_close = Vec::new();
        let mut trade_count = Vec::new();
        let mut start_pos = Vec::new();
        let mut end_pos = Vec::new();
        let mut turnover = Vec::new();
        let mut commission = Vec::new();
        let mut slippage = Vec::new();
        let mut trading_pnl = Vec::new();
        let mut holding_pnl = Vec::new();
        let mut total_pnl = Vec::new();
        let mut net_pnl = Vec::new();
        for daily_result in self.daily_results.lock().unwrap().values() {
            date.push(daily_result.date);
            close_price.push(daily_result.close_price);
            pre_close.push(daily_result.pre_close);
            trade_count.push(daily_result.trade_count);
            start_pos.push(daily_result.start_pos);
            end_pos.push(daily_result.end_pos);
            turnover.push(daily_result.turnover);
            commission.push(daily_result.commission);
            slippage.push(daily_result.slippage);
            trading_pnl.push(daily_result.trading_pnl);
            holding_pnl.push(daily_result.holding_pnl);
            total_pnl.push(daily_result.total_pnl);
            net_pnl.push(daily_result.net_pnl);
        }
        let results = PyDict::new(py);
        results.set_item("date", date)?;
        results.set_item("close_price", close_price)?;
        results.set_item("pre_close", pre_close)?;
        results.set_item("trade_count", trade_count)?;
        results.set_item("start_pos", start_pos)?;
        results.set_item("end_pos", end_pos)?;
        results.set_item("turnover", turnover)?;
        results.set_item("commission", commission)?;
        results.set_item("slippage", slippage)?;
        results.set_item("trading_pnl", trading_pnl)?;
        results.set_item("holding_pnl", holding_pnl)?;
        results.set_item("total_pnl", total_pnl)?;
        results.set_item("net_pnl", net_pnl)?;

        let pd = PyModule::import(py, "pandas")?;
        let dataframe = pd.getattr("DataFrame")?;
        let daily_df = dataframe
            .call_method1("from_dict", (results,))?
            .call_method1("set_index", ("date",))?;
        self.daily_df = Some(daily_df.unbind());

        self.output(py, "逐日盯市盈亏计算完成");
        Ok(self.daily_df.as_ref().unwrap().clone_ref(py))
    }

    fn calculate_statistics(&mut self) {}

    fn show_chart(&mut self) {}

    fn run_bf_optimization(&self) {}

    fn run_ga_optimization(&self) {}

    fn update_daily_close(&self, price: f64) {
        let d = self.datetime.lock().unwrap().unwrap().naive_local().date();

        self.daily_results
            .lock()
            .unwrap()
            .entry(d)
            .and_modify(|e| e.close_price = price)
            .or_insert(DailyResult::new(d, price));
    }

    fn new_bar(&self, py: Python<'_>, bar: &BarData) -> PyResult<()> {
        self.bar.lock().unwrap().replace(bar.clone());
        self.datetime.lock().unwrap().replace(bar.datetime);

        self.cross_limit_order(py)?;
        self.cross_stop_order(py)?;
        self.strategy
            .as_ref()
            .unwrap()
            .call_method1(py, "on_bar", (bar.clone(),))?;

        self.update_daily_close(bar.close_price);
        Ok(())
    }

    fn new_tick(&self, py: Python<'_>, tick: &TickData) -> PyResult<()> {
        self.tick.lock().unwrap().replace(tick.clone());
        self.datetime.lock().unwrap().replace(tick.datetime.clone());

        self.cross_limit_order(py)?;
        self.cross_stop_order(py)?;
        self.strategy
            .as_ref()
            .unwrap()
            .call_method1(py, "on_tick", (tick.clone(),))?;

        self.update_daily_close(tick.last_price);
        Ok(())
    }

    ///Cross limit order with last bar/tick data.
    fn cross_limit_order(&self, py: Python<'_>) -> PyResult<()> {
        let long_cross_price;
        let short_cross_price;
        let long_best_price;
        let short_best_price;
        if self.mode == BacktestingMode::BAR {
            long_cross_price = self.bar.lock().unwrap().as_ref().unwrap().low_price;
            short_cross_price = self.bar.lock().unwrap().as_ref().unwrap().high_price;
            long_best_price = self.bar.lock().unwrap().as_ref().unwrap().open_price;
            short_best_price = self.bar.lock().unwrap().as_ref().unwrap().open_price;
        } else {
            long_cross_price = self.tick.lock().unwrap().as_ref().unwrap().ask_price_1;
            short_cross_price = self.tick.lock().unwrap().as_ref().unwrap().bid_price_1;
            long_best_price = long_cross_price;
            short_best_price = short_cross_price;
        }

        let value_list: Vec<Arc<Mutex<OrderData>>> = self
            .active_limit_orders
            .lock()
            .unwrap()
            .values()
            .map(|v| v.clone())
            .collect();
        for order in value_list {
            // Push order update with status "not traded" (pending).
            let mut order = order.lock().unwrap();
            if order.status == Status::SUBMITTING {
                order.status = Status::NOTTRADED;
                self.strategy
                    .as_ref()
                    .unwrap()
                    .call_method1(py, "on_order", (order.clone(),))?;
            }

            // Check whether limit orders can be filled.
            let long_cross: bool = order.direction == Direction::LONG
                && order.price >= long_cross_price
                && long_cross_price > 0.0;

            let short_cross: bool = order.direction == Direction::SHORT
                && order.price <= short_cross_price
                && short_cross_price > 0.0;

            if !long_cross && !short_cross {
                continue;
            }

            // Push order udpate with status "all traded" (filled).
            order.traded = order.volume;
            order.status = Status::ALLTRADED;
            self.strategy
                .as_ref()
                .unwrap()
                .call_method1(py, "on_order", (order.clone(),))?;

            self.active_limit_orders
                .lock()
                .unwrap()
                .remove(&order.vt_orderid());

            // Push trade update
            *self.trade_count.lock().unwrap() += 1;

            let trade_price;
            let pos_change;
            if long_cross {
                trade_price = order.price.min(long_best_price);
                pos_change = order.volume;
            } else {
                trade_price = order.price.max(short_best_price);
                pos_change = -order.volume;
            }
            let trade = Arc::new(Mutex::new(TradeData {
                symbol: order.symbol.clone(),
                exchange: order.exchange.clone(),
                orderid: order.orderid.clone(),
                tradeid: format!("{:10}", self.trade_count.lock().unwrap()),
                direction: order.direction,
                offset: order.offset,
                price: trade_price,
                volume: order.volume,
                datetime: self.datetime.lock().unwrap().deref().unwrap(),
                gateway_name: BacktestingEngine::gateway_name(),
            }));

            let pos: f64 = self
                .strategy
                .as_ref()
                .unwrap()
                .getattr(py, "pos")?
                .extract(py)?;
            self.strategy
                .as_ref()
                .unwrap()
                .setattr(py, "pos", pos + pos_change)?;
            self.strategy.as_ref().unwrap().call_method1(
                py,
                "on_trade",
                (trade.lock().unwrap().clone(),),
            )?;

            self.trades
                .lock()
                .unwrap()
                .insert(trade.lock().unwrap().vt_tradeid(), trade.clone());
        }
        Ok(())
    }

    ///Cross stop order with last bar/tick data.
    fn cross_stop_order(&self, py: Python<'_>) -> PyResult<()> {
        let long_cross_price;
        let short_cross_price;
        let long_best_price;
        let short_best_price;
        if self.mode == BacktestingMode::BAR {
            long_cross_price = self.bar.lock().unwrap().as_ref().unwrap().high_price;
            short_cross_price = self.bar.lock().unwrap().as_ref().unwrap().low_price;
            long_best_price = self.bar.lock().unwrap().as_ref().unwrap().open_price;
            short_best_price = self.bar.lock().unwrap().as_ref().unwrap().open_price;
        } else {
            long_cross_price = self.tick.lock().unwrap().as_ref().unwrap().last_price;
            short_cross_price = self.tick.lock().unwrap().as_ref().unwrap().last_price;
            long_best_price = long_cross_price;
            short_best_price = short_cross_price;
        }

        let value_list: Vec<Arc<Mutex<StopOrder>>> = self
            .active_stop_orders
            .lock()
            .unwrap()
            .values()
            .map(|v| v.clone())
            .collect();
        for stop_order in value_list {
            let mut stop_order = stop_order.lock().unwrap();
            // Check whether stop order can be triggered.
            let long_cross: bool =
                stop_order.direction == Direction::LONG && stop_order.price <= long_cross_price;

            let short_cross: bool =
                stop_order.direction == Direction::SHORT && stop_order.price >= short_cross_price;

            if !long_cross && !short_cross {
                continue;
            }

            // Create order data.
            *self.limit_order_count.lock().unwrap() += 1;

            let order = Arc::new(Mutex::new(OrderData {
                symbol: self.symbol.clone(),
                exchange: self.exchange.clone(),
                orderid: format!("{:10}", self.limit_order_count.lock().unwrap()),
                direction: stop_order.direction,
                offset: stop_order.offset,
                price: stop_order.price,
                volume: stop_order.volume,
                traded: stop_order.volume,
                status: Status::ALLTRADED,
                gateway_name: BacktestingEngine::gateway_name(),
                datetime: self.datetime.lock().unwrap().deref().unwrap(),
                r#type: OrderType::LIMIT,
                reference: "".to_string(),
            }));

            self.limit_orders
                .lock()
                .unwrap()
                .insert(order.lock().unwrap().vt_orderid(), order.clone());

            // Create trade data.
            let trade_price;
            let pos_change;
            if long_cross {
                trade_price = f64::max(stop_order.price, long_best_price);
                pos_change = order.lock().unwrap().volume;
            } else {
                trade_price = f64::min(stop_order.price, short_best_price);
                pos_change = -order.lock().unwrap().volume;
            }

            *self.trade_count.lock().unwrap() += 1;

            let order_cloned = order.lock().unwrap().clone();
            let trade = Arc::new(Mutex::new(TradeData {
                symbol: order_cloned.symbol.clone(),
                exchange: order_cloned.exchange.clone(),
                orderid: order_cloned.orderid.clone(),
                tradeid: format!("{:10}", self.trade_count.lock().unwrap()),
                direction: order_cloned.direction,
                offset: order_cloned.offset,
                price: trade_price,
                volume: order_cloned.volume,
                datetime: self.datetime.lock().unwrap().deref().unwrap(),
                gateway_name: BacktestingEngine::gateway_name(),
            }));

            self.trades
                .lock()
                .unwrap()
                .insert(trade.lock().unwrap().vt_tradeid(), trade.clone());

            // Update stop order.
            stop_order
                .vt_orderids
                .push(order.lock().unwrap().vt_orderid());
            stop_order.status = StopOrderStatus::TRIGGERED;

            self.active_stop_orders
                .lock()
                .unwrap()
                .remove(&stop_order.stop_orderid);

            // Push update to strategy.
            self.strategy.as_ref().unwrap().call_method1(
                py,
                "on_stop_order",
                (stop_order.clone(),),
            )?;
            self.strategy.as_ref().unwrap().call_method1(
                py,
                "on_order",
                (order.lock().unwrap().clone(),),
            )?;

            let pos: f64 = self
                .strategy
                .as_ref()
                .unwrap()
                .getattr(py, "pos")?
                .extract(py)?;
            self.strategy
                .as_ref()
                .unwrap()
                .setattr(py, "pos", pos + pos_change)?;
            self.strategy.as_ref().unwrap().call_method1(
                py,
                "on_trade",
                (trade.lock().unwrap().clone(),),
            )?;
        }
        Ok(())
    }

    pub fn load_bar(
        &self,
        vt_symbol: &str,
        days: i64,
        interval: Interval,
        _callback: PyObject,
        _use_database: bool,
    ) -> Vec<BarData> {
        let init_end = self.start - INTERVAL_DELTA_MAP[&interval];
        let init_start = self.start - TimeDelta::days(days);

        let (symbol, exchange) = extract_vt_symbol(vt_symbol);

        let bars_mixed = load_bar_data(&symbol, &exchange, interval, init_start, init_end);
        let mut bars: Vec<BarData> = Vec::new();
        for mix_data in bars_mixed {
            if let MixData::BarData(bar_data) = mix_data {
                bars.push(bar_data);
            }
        }

        return bars;
    }

    pub fn load_tick(&self, vt_symbol: &str, days: i64, _callback: PyObject) -> Vec<TickData> {
        let init_end = self.start - TimeDelta::seconds(1);
        let init_start = self.start - TimeDelta::days(days);

        let (symbol, exchange) = extract_vt_symbol(vt_symbol);

        let ticks_mixed = load_tick_data(&symbol, &exchange, init_start, init_end);
        let mut ticks: Vec<TickData> = Vec::new();
        for mix_data in ticks_mixed {
            if let MixData::TickData(tick_data) = mix_data {
                ticks.push(tick_data);
            }
        }

        return ticks;
    }

    pub fn send_order(
        &self,
        py: Python<'_>,
        _strategy: PyObject,
        direction: Direction,
        offset: Offset_,
        price: f64,
        volume: f64,
        stop: bool,
        _lock: bool,
        _net: bool,
    ) -> PyResult<Vec<String>> {
        let price: f64 = round_to(price, self.pricetick);
        let vt_orderid;
        if stop {
            vt_orderid = self.send_stop_order(py, direction, offset, price, volume)?;
        } else {
            vt_orderid = self.send_limit_order(py, direction, offset, price, volume)?;
        }
        Ok(vec![vt_orderid])
    }

    pub fn send_stop_order(
        &self,
        py: Python<'_>,
        direction: Direction,
        offset: Offset_,
        price: f64,
        volume: f64,
    ) -> PyResult<String> {
        *self.stop_order_count.lock().unwrap() += 1;

        let stop_order = Arc::new(Mutex::new(StopOrder {
            vt_symbol: self.vt_symbol.clone(),
            direction: direction,
            offset: offset,
            price: price,
            volume: volume,
            datetime: self.datetime.lock().unwrap().deref().unwrap(),
            stop_orderid: format!(
                "{}.{:10}",
                STOPORDER_PREFIX,
                self.stop_order_count.lock().unwrap()
            ),
            strategy_name: self
                .strategy
                .as_ref()
                .unwrap()
                .getattr(py, "strategy_name")?
                .extract(py)?,
            lock: false,
            net: false,
            vt_orderids: Vec::new(),
            status: StopOrderStatus::WAITING,
        }));

        self.active_stop_orders.lock().unwrap().insert(
            stop_order.lock().unwrap().stop_orderid.clone(),
            stop_order.clone(),
        );
        self.stop_orders.lock().unwrap().insert(
            stop_order.lock().unwrap().stop_orderid.clone(),
            stop_order.clone(),
        );

        let ok = stop_order.lock().unwrap().stop_orderid.clone();
        Ok(ok)
    }

    pub fn send_limit_order(
        &self,
        _py: Python<'_>,
        direction: Direction,
        offset: Offset_,
        price: f64,
        volume: f64,
    ) -> PyResult<String> {
        *self.limit_order_count.lock().unwrap() += 1;

        let order = Arc::new(Mutex::new(OrderData {
            symbol: self.symbol.clone(),
            exchange: self.exchange.clone(),
            orderid: format!("{:10}", self.limit_order_count.lock().unwrap()),
            direction: direction,
            offset: offset,
            price: price,
            volume: volume,
            status: Status::SUBMITTING,
            gateway_name: BacktestingEngine::gateway_name(),
            datetime: self.datetime.lock().unwrap().deref().unwrap(),
            r#type: OrderType::LIMIT,
            reference: "".to_string(),
            traded: 0.0,
        }));

        self.active_limit_orders
            .lock()
            .unwrap()
            .insert(order.lock().unwrap().vt_orderid(), order.clone());
        self.limit_orders
            .lock()
            .unwrap()
            .insert(order.lock().unwrap().vt_orderid(), order.clone());

        let ok = order.lock().unwrap().vt_orderid();
        Ok(ok)
    }

    pub fn cancel_order(
        &self,
        py: Python<'_>,
        strategy: PyObject,
        vt_orderid: &str,
    ) -> PyResult<()> {
        if vt_orderid.starts_with(STOPORDER_PREFIX) {
            self.cancel_stop_order(py, strategy, vt_orderid)?;
        } else {
            self.cancel_limit_order(py, strategy, vt_orderid)?;
        }
        Ok(())
    }

    pub fn cancel_stop_order(
        &self,
        py: Python<'_>,
        _strategy: PyObject,
        vt_orderid: &str,
    ) -> PyResult<()> {
        if let Some(stop_order) = self.active_stop_orders.lock().unwrap().remove(vt_orderid) {
            stop_order.lock().unwrap().status = StopOrderStatus::CANCELLED;
            self.strategy.as_ref().unwrap().call_method1(
                py,
                "on_stop_order",
                (stop_order.lock().unwrap().clone(),),
            )?;
        }
        Ok(())
    }

    pub fn cancel_limit_order(
        &self,
        py: Python<'_>,
        _strategy: PyObject,
        vt_orderid: &str,
    ) -> PyResult<()> {
        if let Some(order) = self.active_limit_orders.lock().unwrap().remove(vt_orderid) {
            order.lock().unwrap().status = Status::CANCELLED;
            self.strategy.as_ref().unwrap().call_method1(
                py,
                "on_order",
                (order.lock().unwrap().clone(),),
            )?;
        }
        Ok(())
    }

    pub fn cancel_all(&self, py: Python<'_>, strategy: PyObject) -> PyResult<()> {
        let vt_orderids: Vec<String> = self
            .active_limit_orders
            .lock()
            .unwrap()
            .keys()
            .cloned()
            .collect();
        for vt_orderid in vt_orderids {
            self.cancel_limit_order(py, strategy.clone_ref(py), &vt_orderid)?;
        }

        let stop_orderids: Vec<String> = self
            .active_stop_orders
            .lock()
            .unwrap()
            .keys()
            .cloned()
            .collect();
        for vt_orderid in stop_orderids {
            self.cancel_stop_order(py, strategy.clone_ref(py), &vt_orderid)?;
        }
        Ok(())
    }

    pub fn write_log(&self, msg: &str, _strategy: PyObject) {
        let msg: String = format!("{}\t{}", Local::now().naive_local(), msg);
        self.logs.lock().unwrap().push(msg);
    }

    pub fn send_email(&self, _msg: &str, _strategy: PyObject) {}

    pub fn sync_strategy_data(&self, _strategy: PyObject) {}

    pub fn get_engine_type(&self) -> EngineType {
        Self::engine_type()
    }

    pub fn get_pricetick(&self, _strategy: PyObject) -> f64 {
        self.pricetick
    }

    pub fn get_size(&self, _strategy: PyObject) -> f64 {
        self.size
    }

    pub fn put_strategy_event(&self, _strategy: PyObject) {}

    pub fn output(&self, py: Python<'_>, msg: &str) {
        match self.rs_pyfunc_output.as_ref() {
            Some(func) => {
                let _ = func.call1(py, (msg,));
            }
            None => {
                println!("{}\t{}", Local::now().naive_local(), msg);
            }
        }
    }

    pub fn get_all_trades(&self) -> Vec<TradeData> {
        self.trades
            .lock()
            .unwrap()
            .values()
            .map(|item| item.lock().unwrap().clone())
            .collect()
    }

    pub fn get_all_orders(&self) -> Vec<OrderData> {
        self.limit_orders
            .lock()
            .unwrap()
            .values()
            .map(|item| item.lock().unwrap().clone())
            .collect()
    }

    pub fn get_all_daily_results(&self) -> Vec<DailyResult> {
        self.daily_results
            .lock()
            .unwrap()
            .values()
            .cloned()
            .collect()
    }

    pub fn set_output(&mut self, func: PyObject) -> PyResult<()> {
        self.rs_pyfunc_output = Some(func);
        Ok(())
    }

    pub fn has_history_data(&self) -> bool {
        !self.history_data.is_empty()
    }

    #[getter]
    pub fn get_history_data(&self, py: Python<'_>) -> PyResult<PyObject> {
        if self.history_data.is_empty() {
            return Ok(PyList::empty(py).into());
        }
        match self.history_data.front().as_ref().unwrap() {
            MixData::BarData(_) => {
                let mut list: Vec<BarData> = Vec::new();
                for item in self.history_data.iter() {
                    match item {
                        MixData::BarData(bar_data) => {
                            list.push(bar_data.clone());
                        }
                        _ => {}
                    }
                }
                return Ok(PyList::new(py, list)?.into());
            }
            MixData::TickData(_) => {
                let mut list: Vec<TickData> = Vec::new();
                for item in self.history_data.iter() {
                    match item {
                        MixData::TickData(tick_data) => {
                            list.push(tick_data.clone());
                        }
                        _ => {}
                    }
                }
                return Ok(PyList::new(py, list)?.into());
            }
        }
    }
}

#[pyclass(get_all)]
#[derive(Default, Clone)]
pub struct DailyResult {
    date: NaiveDate,
    close_price: f64,
    pre_close: f64,

    trades: Vec<TradeData>,
    trade_count: i64,

    start_pos: f64,
    end_pos: f64,

    turnover: f64,
    commission: f64,
    slippage: f64,

    trading_pnl: f64,
    holding_pnl: f64,
    total_pnl: f64,
    net_pnl: f64,
}

impl DailyResult {
    pub fn new(date: NaiveDate, close_price: f64) -> Self {
        DailyResult {
            date,
            close_price,
            ..Default::default()
        }
    }

    pub fn add_trade(&mut self, trade: TradeData) {
        self.trades.push(trade)
    }

    fn calculate_pnl(
        &mut self,
        pre_close: f64,
        start_pos: f64,
        size: f64,
        rate: f64,
        slippage: f64,
    ) {
        // If no pre_close provided on the first day,
        // use value 1 to avoid zero division error
        if pre_close != 0.0 {
            self.pre_close = pre_close;
        } else {
            self.pre_close = 1.0;
        }

        // Holding pnl is the pnl from holding position at day start
        self.start_pos = start_pos;
        self.end_pos = start_pos;

        self.holding_pnl = self.start_pos * (self.close_price - self.pre_close) * size;

        // Trading pnl is the pnl from new trade during the day
        self.trade_count = self.trades.len() as i64;

        for trade in &self.trades {
            let pos_change;
            if trade.direction == Direction::LONG {
                pos_change = trade.volume;
            } else {
                pos_change = -trade.volume;
            }

            self.end_pos += pos_change;

            let turnover = trade.volume * size * trade.price;
            self.trading_pnl += pos_change * (self.close_price - trade.price) * size;
            self.slippage += trade.volume * size * slippage;

            self.turnover += turnover;
            self.commission += turnover * rate;
        }

        // Net pnl takes account of commission and slippage cost
        self.total_pnl = self.trading_pnl + self.holding_pnl;
        self.net_pnl = self.total_pnl - self.commission - self.slippage;
    }
}

fn load_bar_data(
    symbol: &str,
    exchange: &str,
    interval: Interval,
    start: NaiveDateTime,
    end: NaiveDateTime,
) -> LinkedList<MixData> {
    let database = get_database();

    return database.load_bar_data(symbol, exchange, interval, start, end);
}

fn load_tick_data(
    symbol: &str,
    exchange: &str,
    start: NaiveDateTime,
    end: NaiveDateTime,
) -> LinkedList<MixData> {
    let database = get_database();

    return database.load_tick_data(symbol, exchange, start, end);
}

#[pymodule]
pub fn backtesting(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<EngineType>()?;
    m.add_class::<BacktestingMode>()?;
    m.add_class::<BacktestingEngine>()?;
    Ok(())
}
