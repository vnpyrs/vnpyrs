use chrono::NaiveDateTime;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySqlPool;
use sqlx::Row;
use sqlx::SqlitePool;
use std::collections::LinkedList;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;

use super::object::MixData;
use super::object::TickData;
use super::{constant::Interval, object::BarData};

pub static DBMAP: Mutex<GlobalDBMap> = Mutex::new(GlobalDBMap::new());

pub struct GlobalDBMap {
    pub sqlite: Option<Arc<SqliteDatabase>>,
    pub mysql: Option<Arc<MysqlDatabase>>,
}

impl GlobalDBMap {
    pub const fn new() -> Self {
        GlobalDBMap {
            sqlite: None,
            mysql: None,
        }
    }
}

pub trait BaseDatabase {
    fn load_bar_data(
        &self,
        symbol: &str,
        exchange: &str,
        interval: Interval,
        start: NaiveDateTime,
        end: NaiveDateTime,
    ) -> LinkedList<MixData>;
    fn load_tick_data(
        &self,
        symbol: &str,
        exchange: &str,
        start: NaiveDateTime,
        end: NaiveDateTime,
    ) -> LinkedList<MixData>;
}

pub struct SqliteDatabase {
    pool: SqlitePool,
    rt: tokio::runtime::Runtime,
}

impl SqliteDatabase {
    pub fn connect(url: &str) -> Result<SqliteDatabase, Box<dyn std::error::Error>> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let pool = rt.block_on(SqlitePool::connect(url))?;
        Ok(SqliteDatabase { pool, rt })
    }
}

impl BaseDatabase for SqliteDatabase {
    fn load_bar_data(
        &self,
        symbol: &str,
        exchange: &str,
        interval: Interval,
        start: NaiveDateTime,
        end: NaiveDateTime,
    ) -> LinkedList<MixData> {
        let interval_str = interval.to_string();

        let s = self.rt.block_on(
            sqlx::query("SELECT symbol,exchange,datetime,interval,volume,turnover,open_interest,open_price,high_price,low_price,close_price FROM dbbardata WHERE symbol=? and exchange=? and interval=? and datetime>=? and datetime<=? ORDER BY datetime")
                    .bind(symbol).bind(exchange).bind(interval_str).bind(start).bind(end)
                    .fetch_all(&self.pool)).unwrap();
        let mut bars = LinkedList::new();
        for db_bar in s.iter() {
            bars.push_back(MixData::BarData(BarData {
                symbol: db_bar.get::<String, usize>(0),
                exchange: db_bar.get::<String, usize>(1),
                datetime: db_bar.get::<NaiveDateTime, usize>(2),
                interval: Interval::from_str(db_bar.get::<&str, usize>(3))
                    .expect("数据库中interval字段只能是1m,1h,d,w,tick中的一个"),
                volume: db_bar.get::<f64, usize>(4),
                turnover: db_bar.get::<f64, usize>(5),
                open_interest: db_bar.get::<f64, usize>(6),
                open_price: db_bar.get::<f64, usize>(7),
                high_price: db_bar.get::<f64, usize>(8),
                low_price: db_bar.get::<f64, usize>(9),
                close_price: db_bar.get::<f64, usize>(10),
                gateway_name: "DB",
            }));
        }
        bars
    }

    fn load_tick_data(
        &self,
        symbol: &str,
        exchange: &str,
        start: NaiveDateTime,
        end: NaiveDateTime,
    ) -> LinkedList<MixData> {
        let s = self.rt.block_on(
            sqlx::query("SELECT symbol,exchange,datetime,name,volume,turnover,open_interest,last_price,last_volume,limit_up,limit_down,open_price,high_price,low_price,pre_close,bid_price_1,bid_price_2,bid_price_3,bid_price_4,bid_price_5,ask_price_1,ask_price_2,ask_price_3,ask_price_4,ask_price_5,bid_volume_1,bid_volume_2,bid_volume_3,bid_volume_4,bid_volume_5,ask_volume_1,ask_volume_2,ask_volume_3,ask_volume_4,ask_volume_5,localtime FROM dbbardata WHERE symbol=? and exchange=? and datetime>=? and datetime<=? ORDER BY datetime")
                    .bind(symbol).bind(exchange).bind(start).bind(end)
                    .fetch_all(&self.pool)).unwrap();
        let mut ticks = LinkedList::new();
        for db_tick in s.iter() {
            ticks.push_back(MixData::TickData(TickData {
                symbol: db_tick.get::<String, usize>(0),
                exchange: db_tick.get::<String, usize>(1),
                datetime: db_tick.get::<NaiveDateTime, usize>(2),
                name: db_tick.get::<String, usize>(3),
                volume: db_tick.get::<f64, usize>(4),
                turnover: db_tick.get::<f64, usize>(5),
                open_interest: db_tick.get::<f64, usize>(6),
                last_price: db_tick.get::<f64, usize>(7),
                last_volume: db_tick.get::<f64, usize>(8),
                limit_up: db_tick.get::<f64, usize>(9),
                limit_down: db_tick.get::<f64, usize>(10),
                open_price: db_tick.get::<f64, usize>(11),
                high_price: db_tick.get::<f64, usize>(12),
                low_price: db_tick.get::<f64, usize>(13),
                pre_close: db_tick.get::<f64, usize>(14),
                bid_price_1: db_tick.get::<f64, usize>(15),
                bid_price_2: db_tick.get::<f64, usize>(16),
                bid_price_3: db_tick.get::<f64, usize>(17),
                bid_price_4: db_tick.get::<f64, usize>(18),
                bid_price_5: db_tick.get::<f64, usize>(19),
                ask_price_1: db_tick.get::<f64, usize>(20),
                ask_price_2: db_tick.get::<f64, usize>(21),
                ask_price_3: db_tick.get::<f64, usize>(22),
                ask_price_4: db_tick.get::<f64, usize>(23),
                ask_price_5: db_tick.get::<f64, usize>(24),
                bid_volume_1: db_tick.get::<f64, usize>(25),
                bid_volume_2: db_tick.get::<f64, usize>(26),
                bid_volume_3: db_tick.get::<f64, usize>(27),
                bid_volume_4: db_tick.get::<f64, usize>(28),
                bid_volume_5: db_tick.get::<f64, usize>(29),
                ask_volume_1: db_tick.get::<f64, usize>(30),
                ask_volume_2: db_tick.get::<f64, usize>(31),
                ask_volume_3: db_tick.get::<f64, usize>(32),
                ask_volume_4: db_tick.get::<f64, usize>(33),
                ask_volume_5: db_tick.get::<f64, usize>(34),
                localtime: db_tick.get::<NaiveDateTime, usize>(35),
                gateway_name: "DB",
            }));
        }
        ticks
    }
}

pub struct MysqlDatabase {
    pool: MySqlPool,
    rt: tokio::runtime::Runtime,
}

impl MysqlDatabase {
    pub fn connect(url: &str) -> Result<MysqlDatabase, Box<dyn std::error::Error>> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let pool = rt.block_on(MySqlPoolOptions::new().max_connections(5).connect(url))?;
        Ok(MysqlDatabase { pool, rt })
    }
}

impl BaseDatabase for MysqlDatabase {
    fn load_bar_data(
        &self,
        symbol: &str,
        exchange: &str,
        interval: Interval,
        start: NaiveDateTime,
        end: NaiveDateTime,
    ) -> LinkedList<MixData> {
        let interval_str = interval.to_string();

        let s = self.rt.block_on(
            sqlx::query("SELECT symbol,exchange,datetime,`interval`,volume,turnover,open_interest,open_price,high_price,low_price,close_price FROM dbbardata WHERE symbol=? and exchange=? and `interval`=? and datetime>=? and datetime<=? ORDER BY datetime")
                    .bind(symbol).bind(exchange).bind(interval_str).bind(start).bind(end)
                    .fetch_all(&self.pool)).unwrap();
        let mut bars = LinkedList::new();
        for db_bar in s.iter() {
            bars.push_back(MixData::BarData(BarData {
                symbol: db_bar.get::<String, usize>(0),
                exchange: db_bar.get::<String, usize>(1),
                datetime: db_bar.get::<NaiveDateTime, usize>(2),
                interval: Interval::from_str(db_bar.get::<&str, usize>(3))
                    .expect("数据库中interval字段只能是1m,1h,d,w,tick中的一个"),
                volume: db_bar.get::<f64, usize>(4),
                turnover: db_bar.get::<f64, usize>(5),
                open_interest: db_bar.get::<f64, usize>(6),
                open_price: db_bar.get::<f64, usize>(7),
                high_price: db_bar.get::<f64, usize>(8),
                low_price: db_bar.get::<f64, usize>(9),
                close_price: db_bar.get::<f64, usize>(10),
                gateway_name: "DB",
            }));
        }
        bars
    }

    fn load_tick_data(
        &self,
        symbol: &str,
        exchange: &str,
        start: NaiveDateTime,
        end: NaiveDateTime,
    ) -> LinkedList<MixData> {
        let s = self.rt.block_on(
            sqlx::query("SELECT symbol,exchange,datetime,name,volume,turnover,open_interest,last_price,last_volume,limit_up,limit_down,open_price,high_price,low_price,pre_close,bid_price_1,bid_price_2,bid_price_3,bid_price_4,bid_price_5,ask_price_1,ask_price_2,ask_price_3,ask_price_4,ask_price_5,bid_volume_1,bid_volume_2,bid_volume_3,bid_volume_4,bid_volume_5,ask_volume_1,ask_volume_2,ask_volume_3,ask_volume_4,ask_volume_5,localtime FROM dbbardata WHERE symbol=? and exchange=? and datetime>=? and datetime<=? ORDER BY datetime")
                    .bind(symbol).bind(exchange).bind(start).bind(end)
                    .fetch_all(&self.pool)).unwrap();
        let mut ticks = LinkedList::new();
        for db_tick in s.iter() {
            ticks.push_back(MixData::TickData(TickData {
                symbol: db_tick.get::<String, usize>(0),
                exchange: db_tick.get::<String, usize>(1),
                datetime: db_tick.get::<NaiveDateTime, usize>(2),
                name: db_tick.get::<String, usize>(3),
                volume: db_tick.get::<f64, usize>(4),
                turnover: db_tick.get::<f64, usize>(5),
                open_interest: db_tick.get::<f64, usize>(6),
                last_price: db_tick.get::<f64, usize>(7),
                last_volume: db_tick.get::<f64, usize>(8),
                limit_up: db_tick.get::<f64, usize>(9),
                limit_down: db_tick.get::<f64, usize>(10),
                open_price: db_tick.get::<f64, usize>(11),
                high_price: db_tick.get::<f64, usize>(12),
                low_price: db_tick.get::<f64, usize>(13),
                pre_close: db_tick.get::<f64, usize>(14),
                bid_price_1: db_tick.get::<f64, usize>(15),
                bid_price_2: db_tick.get::<f64, usize>(16),
                bid_price_3: db_tick.get::<f64, usize>(17),
                bid_price_4: db_tick.get::<f64, usize>(18),
                bid_price_5: db_tick.get::<f64, usize>(19),
                ask_price_1: db_tick.get::<f64, usize>(20),
                ask_price_2: db_tick.get::<f64, usize>(21),
                ask_price_3: db_tick.get::<f64, usize>(22),
                ask_price_4: db_tick.get::<f64, usize>(23),
                ask_price_5: db_tick.get::<f64, usize>(24),
                bid_volume_1: db_tick.get::<f64, usize>(25),
                bid_volume_2: db_tick.get::<f64, usize>(26),
                bid_volume_3: db_tick.get::<f64, usize>(27),
                bid_volume_4: db_tick.get::<f64, usize>(28),
                bid_volume_5: db_tick.get::<f64, usize>(29),
                ask_volume_1: db_tick.get::<f64, usize>(30),
                ask_volume_2: db_tick.get::<f64, usize>(31),
                ask_volume_3: db_tick.get::<f64, usize>(32),
                ask_volume_4: db_tick.get::<f64, usize>(33),
                ask_volume_5: db_tick.get::<f64, usize>(34),
                localtime: db_tick.get::<NaiveDateTime, usize>(35),
                gateway_name: "DB",
            }));
        }
        ticks
    }
}
