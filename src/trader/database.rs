use std::sync::{Arc, LazyLock};

use chrono_tz::Tz;

use super::{
    database_impl::{BaseDatabase, MongodbDatabase, MysqlDatabase, SqliteDatabase, DBMAP},
    setting::SETTINGS,
    utility::get_file_path,
};

pub static DB_TZ: LazyLock<Tz> = LazyLock::new(|| {
    SETTINGS
        .database_timezone
        .parse()
        .expect("配置文件中database.timezone错误")
});

pub fn get_database() -> Arc<dyn BaseDatabase> {
    // Read database related global setting
    match SETTINGS.database_name.as_str() {
        "sqlite" => {
            if DBMAP.lock().unwrap().sqlite.is_none() {
                DBMAP.lock().unwrap().sqlite = Some(Arc::new(
                    SqliteDatabase::connect(
                        &get_file_path("database.db")
                            .into_os_string()
                            .into_string()
                            .unwrap(),
                    )
                    .expect("Sqlite数据库打开失败"),
                ));
            }
            return DBMAP.lock().unwrap().sqlite.as_ref().unwrap().clone();
        }
        "mysql" => {
            if DBMAP.lock().unwrap().mysql.is_none() {
                DBMAP.lock().unwrap().mysql = Some(Arc::new(
                    MysqlDatabase::connect(&format!(
                        "mysql://{}:{}@{}:{}/{}",
                        SETTINGS.database_user,
                        SETTINGS.database_password,
                        SETTINGS.database_host,
                        SETTINGS.database_port,
                        SETTINGS.database_database
                    ))
                    .expect("Mysql数据库打开失败"),
                ));
            }
            return DBMAP.lock().unwrap().mysql.as_ref().unwrap().clone();
        }
        "mongodb" => {
            if DBMAP.lock().unwrap().mongodb.is_none() {
                DBMAP.lock().unwrap().mongodb = Some(Arc::new(
                    MongodbDatabase::connect(
                        &format!(
                            "mongodb://{}:{}",
                            SETTINGS.database_host, SETTINGS.database_port
                        ),
                        &SETTINGS.database_user,
                        &SETTINGS.database_password,
                        &SETTINGS.database_database,
                    )
                    .expect("MongoDB数据库打开失败"),
                ));
            }
            return DBMAP.lock().unwrap().mongodb.as_ref().unwrap().clone();
        }
        _ => {
            unreachable!("unsupported Database")
        }
    }
}
