use chrono::NaiveDateTime;
use chrono_tz::Tz;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySqlPool;
use sqlx::Row;
use sqlx::SqlitePool;
use std::collections::LinkedList;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;

use super::database::DB_TZ;
use super::object::MixData;
use super::object::TickData;
use super::{constant::Interval, object::BarData};

pub static DBMAP: Mutex<GlobalDBMap> = Mutex::new(GlobalDBMap::new());
static SH_TZ: Tz = Tz::Asia__Shanghai;

pub struct GlobalDBMap {
    pub sqlite: Option<Arc<SqliteDatabase>>,
    pub mysql: Option<Arc<MysqlDatabase>>,
    pub mongodb: Option<Arc<MongodbDatabase>>,
}

impl GlobalDBMap {
    pub const fn new() -> Self {
        GlobalDBMap {
            sqlite: None,
            mysql: None,
            mongodb: None,
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
        let tz = DB_TZ.clone();
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
                datetime: db_bar
                    .get::<NaiveDateTime, usize>(2)
                    .and_local_timezone(SH_TZ)
                    .unwrap()
                    .with_timezone(&tz),
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
        let tz = DB_TZ.clone();
        let s = self.rt.block_on(
            sqlx::query("SELECT symbol,exchange,datetime,name,volume,turnover,open_interest,last_price,last_volume,limit_up,limit_down,open_price,high_price,low_price,pre_close,bid_price_1,bid_price_2,bid_price_3,bid_price_4,bid_price_5,ask_price_1,ask_price_2,ask_price_3,ask_price_4,ask_price_5,bid_volume_1,bid_volume_2,bid_volume_3,bid_volume_4,bid_volume_5,ask_volume_1,ask_volume_2,ask_volume_3,ask_volume_4,ask_volume_5,localtime FROM dbbardata WHERE symbol=? and exchange=? and datetime>=? and datetime<=? ORDER BY datetime")
                    .bind(symbol).bind(exchange).bind(start).bind(end)
                    .fetch_all(&self.pool)).unwrap();
        let mut ticks = LinkedList::new();
        for db_tick in s.iter() {
            ticks.push_back(MixData::TickData(TickData {
                symbol: db_tick.get::<String, usize>(0),
                exchange: db_tick.get::<String, usize>(1),
                datetime: db_tick
                    .get::<NaiveDateTime, usize>(2)
                    .and_local_timezone(SH_TZ)
                    .unwrap()
                    .with_timezone(&tz),
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
        let tz = DB_TZ.clone();
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
                datetime: db_bar
                    .get::<NaiveDateTime, usize>(2)
                    .and_local_timezone(SH_TZ)
                    .unwrap()
                    .with_timezone(&tz),
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
        let tz = DB_TZ.clone();
        let s = self.rt.block_on(
            sqlx::query("SELECT symbol,exchange,datetime,name,volume,turnover,open_interest,last_price,last_volume,limit_up,limit_down,open_price,high_price,low_price,pre_close,bid_price_1,bid_price_2,bid_price_3,bid_price_4,bid_price_5,ask_price_1,ask_price_2,ask_price_3,ask_price_4,ask_price_5,bid_volume_1,bid_volume_2,bid_volume_3,bid_volume_4,bid_volume_5,ask_volume_1,ask_volume_2,ask_volume_3,ask_volume_4,ask_volume_5,localtime FROM dbbardata WHERE symbol=? and exchange=? and datetime>=? and datetime<=? ORDER BY datetime")
                    .bind(symbol).bind(exchange).bind(start).bind(end)
                    .fetch_all(&self.pool)).unwrap();
        let mut ticks = LinkedList::new();
        for db_tick in s.iter() {
            ticks.push_back(MixData::TickData(TickData {
                symbol: db_tick.get::<String, usize>(0),
                exchange: db_tick.get::<String, usize>(1),
                datetime: db_tick
                    .get::<NaiveDateTime, usize>(2)
                    .and_local_timezone(SH_TZ)
                    .unwrap()
                    .with_timezone(&tz),
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

use mongodb::{
    bson::{doc, Document},
    Client, Collection,
};
pub struct MongodbDatabase {
    _client: Client,
    coll_bar_data: Collection<Document>,
    coll_tick_data: Collection<Document>,
    rt: tokio::runtime::Runtime,
}

impl MongodbDatabase {
    pub fn connect(
        url: &str,
        _username: &str,
        _password: &str,
        database: &str,
    ) -> Result<MongodbDatabase, Box<dyn std::error::Error>> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let (_client, coll_bar_data, coll_tick_data) = rt.block_on(async {
            let client = Client::with_uri_str(url).await.expect("Mongodb URL error");
            let db = client.database(database);
            let coll_bar_data = db.collection("bar_data");
            let coll_tick_data = db.collection("tick_data");
            (client, coll_bar_data, coll_tick_data)
        });
        Ok(MongodbDatabase {
            _client,
            coll_bar_data,
            coll_tick_data,
            rt,
        })
    }
}

impl BaseDatabase for MongodbDatabase {
    fn load_bar_data(
        &self,
        symbol: &str,
        exchange: &str,
        interval: Interval,
        start: NaiveDateTime,
        end: NaiveDateTime,
    ) -> LinkedList<MixData> {
        let start_ = bson::DateTime::from_millis(
            start.and_local_timezone(SH_TZ).unwrap().timestamp_millis(),
        );
        let end_ =
            bson::DateTime::from_millis(end.and_local_timezone(SH_TZ).unwrap().timestamp_millis());
        let tz = DB_TZ.clone();
        self.rt.block_on(async {
            let mut bars: LinkedList<MixData> = LinkedList::new();
            let mut cursor = self
                .coll_bar_data
                .find(doc! {"symbol":symbol,"exchange":exchange,"interval":interval.to_string(),"datetime": doc! { "$gte": start_,"$lte":end_ }})
                .sort(doc! {"datetime":1})
                .await
                .unwrap();
            while cursor.advance().await.unwrap() {
                let current = cursor.current();
                bars.push_back(MixData::BarData(BarData{
                    symbol: current.get_str("symbol").unwrap().to_string(),
                    exchange: current.get_str("exchange").unwrap().to_string(),
                    datetime: current
                    .get_datetime("datetime")
                    .unwrap()
                    .to_chrono()
                    .with_timezone(&tz),
                    interval: Interval::from_str(current.get_str("interval").unwrap())
                        .expect("数据库中interval字段只能是1m,1h,d,w,tick中的一个"),
                    volume: current.get("volume").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    turnover: current.get("turnover").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    open_interest: current.get("open_interest").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    open_price: current.get("open_price").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    high_price: current.get("high_price").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    low_price: current.get("low_price").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    close_price: current.get("close_price").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    gateway_name: "DB",
                }));
            }
            bars
        })
    }

    fn load_tick_data(
        &self,
        symbol: &str,
        exchange: &str,
        start: NaiveDateTime,
        end: NaiveDateTime,
    ) -> LinkedList<MixData> {
        let start_ = bson::DateTime::from_millis(
            start.and_local_timezone(SH_TZ).unwrap().timestamp_millis(),
        );
        let end_ =
            bson::DateTime::from_millis(end.and_local_timezone(SH_TZ).unwrap().timestamp_millis());
        let tz = DB_TZ.clone();
        self.rt.block_on(async {
            let mut ticks: LinkedList<MixData> = LinkedList::new();
            let mut cursor = self
                .coll_tick_data
                .find(doc! {"symbol":symbol,"exchange":exchange,"datetime": doc! { "$gte": start_,"$lte":end_ }})
                .sort(doc! {"datetime":1})
                .await
                .unwrap();
            while cursor.advance().await.unwrap() {
                let current = cursor.current();
                ticks.push_back(MixData::TickData(TickData{
                    symbol: current.get_str("symbol").unwrap().to_string(),
                    exchange: current.get_str("exchange").unwrap().to_string(),
                    datetime: current
                    .get_datetime("datetime")
                    .unwrap()
                    .to_chrono()
                    .with_timezone(&tz),
                    name: current.get_str("name").unwrap().to_string(),
                    volume: current.get("volume").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    turnover: current.get("turnover").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    open_interest: current.get("open_interest").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    last_price: current.get("last_price").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    last_volume: current.get("last_volume").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    limit_up: current.get("limit_up").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    limit_down: current.get("limit_down").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    open_price: current.get("open_price").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    high_price: current.get("high_price").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    low_price: current.get("low_price").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    pre_close: current.get("pre_close").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    bid_price_1: current.get("bid_price_1").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    bid_price_2: current.get("bid_price_2").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    bid_price_3: current.get("bid_price_3").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    bid_price_4: current.get("bid_price_4").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    bid_price_5: current.get("bid_price_5").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    ask_price_1: current.get("ask_price_1").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    ask_price_2: current.get("ask_price_2").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    ask_price_3: current.get("ask_price_3").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    ask_price_4: current.get("ask_price_4").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    ask_price_5: current.get("ask_price_5").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    bid_volume_1: current.get("bid_volume_1").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    bid_volume_2: current.get("bid_volume_2").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    bid_volume_3: current.get("bid_volume_3").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    bid_volume_4: current.get("bid_volume_4").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    bid_volume_5: current.get("bid_volume_5").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    ask_volume_1: current.get("ask_volume_1").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    ask_volume_2: current.get("ask_volume_2").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    ask_volume_3: current.get("ask_volume_3").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    ask_volume_4: current.get("ask_volume_4").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    ask_volume_5: current.get("ask_volume_5").unwrap().unwrap().as_f64().or(Some(0.0)).unwrap(),
                    localtime: current
                    .get_datetime("localtime")
                    .unwrap_or(bson::DateTime::from_millis(0))
                    .to_chrono().naive_local(),
                    gateway_name: "DB",
                    }));
            }
            ticks
        })
    }
}
