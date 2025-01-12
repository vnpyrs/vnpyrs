use std::sync::{Arc, LazyLock};

use chrono_tz::Tz;

use super::{
    database_impl::{BaseDatabase, MysqlDatabase, SqliteDatabase, DBMAP},
    setting::SETTINGS,
    utility::get_file_path,
};

pub static DB_TZ: LazyLock<Tz> = LazyLock::new(|| SETTINGS.database_timezone.parse().expect("配置文件中database.timezone错误"));

pub fn get_database() -> Arc<dyn BaseDatabase> {
    // Read database related global setting
    match SETTINGS.database_name.as_str() {
        "sqlite" => {
            if DBMAP.lock().unwrap().sqlite.is_none() {
                DBMAP.lock().unwrap().sqlite = Some(Arc::new(
                    SqliteDatabase::connect(
                        &get_file_path(&SETTINGS.database_database)
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
        _ => {
            unreachable!("unsupported Database")
        }
    }
}
