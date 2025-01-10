use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::{io::Write, sync::LazyLock};

use crate::trader::utility::get_file_path;

pub fn get_localzone_name() -> String {
    Python::with_gil(|py| {
        let tzlocal = PyModule::import(py, "tzlocal").unwrap();
        tzlocal
            .getattr("get_localzone_name")
            .unwrap()
            .call0()
            .unwrap()
            .extract()
            .unwrap()
    })
}

#[derive(Serialize, Deserialize)]
pub struct SettingDict {
    #[serde(default, rename = "font.family")]
    pub font_family: String,
    #[serde(default, rename = "font.size")]
    pub font_size: u16,

    #[serde(default, rename = "email.server")]
    pub email_server: String,
    #[serde(default, rename = "email.port")]
    pub email_port: u16,
    #[serde(default, rename = "email.username")]
    pub email_username: String,
    #[serde(default, rename = "email.password")]
    pub email_password: String,
    #[serde(default, rename = "email.sender")]
    pub email_sender: String,
    #[serde(default, rename = "email.receiver")]
    pub email_receiver: String,

    #[serde(default, rename = "datafeed.name")]
    pub datafeed_name: String,
    #[serde(default, rename = "datafeed.username")]
    pub datafeed_username: String,
    #[serde(default, rename = "datafeed.password")]
    pub datafeed_password: String,

    #[serde(default, rename = "database.timezone")]
    pub database_timezone: String,
    #[serde(default, rename = "database.name")]
    pub database_name: String,
    #[serde(default, rename = "database.database")]
    pub database_database: String,
    #[serde(default, rename = "database.host")]
    pub database_host: String,
    #[serde(default, rename = "database.port")]
    pub database_port: u16,
    #[serde(default, rename = "database.user")]
    pub database_user: String,
    #[serde(default, rename = "database.password")]
    pub database_password: String,
}

const SETTING_FILENAME: &str = "vt_setting.json";
pub static SETTINGS: LazyLock<SettingDict> = LazyLock::new(|| {
    let mut setting = SettingDict {
        font_family: "微软雅黑".to_string(),
        font_size: 12,
        email_server: "smtp.qq.com".to_string(),
        email_port: 465,
        email_username: "".to_string(),
        email_password: "".to_string(),
        email_sender: "".to_string(),
        email_receiver: "".to_string(),

        datafeed_name: "".to_string(),
        datafeed_username: "".to_string(),
        datafeed_password: "".to_string(),

        database_timezone: get_localzone_name(),
        database_name: "sqlite".to_string(),
        database_database: "database.db".to_string(),
        database_host: "".to_string(),
        database_port: 0,
        database_user: "".to_string(),
        database_password: "".to_string(),
    };
    if let Ok(file_content) = std::fs::read_to_string(get_file_path(SETTING_FILENAME)) {
        let in_json_file: SettingDict = serde_json::from_str(&file_content).unwrap();
        if !in_json_file.font_family.is_empty() {
            setting.font_family = in_json_file.font_family;
        }
        if in_json_file.font_size != 0 {
            setting.font_size = in_json_file.font_size;
        }
        if !in_json_file.email_server.is_empty() {
            setting.email_server = in_json_file.email_server;
        }
        if in_json_file.email_port != 0 {
            setting.email_port = in_json_file.email_port;
        }
        if !in_json_file.email_username.is_empty() {
            setting.email_username = in_json_file.email_username;
        }
        if !in_json_file.email_password.is_empty() {
            setting.email_password = in_json_file.email_password;
        }
        if !in_json_file.email_sender.is_empty() {
            setting.email_sender = in_json_file.email_sender;
        }
        if !in_json_file.email_receiver.is_empty() {
            setting.email_receiver = in_json_file.email_receiver;
        }
        if !in_json_file.datafeed_name.is_empty() {
            setting.datafeed_name = in_json_file.datafeed_name;
        }
        if !in_json_file.datafeed_username.is_empty() {
            setting.datafeed_username = in_json_file.datafeed_username;
        }
        if !in_json_file.datafeed_password.is_empty() {
            setting.datafeed_password = in_json_file.datafeed_password;
        }
        if !in_json_file.database_timezone.is_empty() {
            setting.database_timezone = in_json_file.database_timezone;
        }
        if !in_json_file.database_name.is_empty() {
            setting.database_name = in_json_file.database_name;
        }
        if !in_json_file.database_database.is_empty() {
            setting.database_database = in_json_file.database_database;
        }
        if !in_json_file.database_host.is_empty() {
            setting.database_host = in_json_file.database_host;
        }
        if in_json_file.database_port != 0 {
            setting.database_port = in_json_file.database_port;
        }
        if !in_json_file.database_user.is_empty() {
            setting.database_user = in_json_file.database_user;
        }
        if !in_json_file.database_password.is_empty() {
            setting.database_password = in_json_file.database_password;
        }
    } else {
        println!("文件{}未找到，使用默认值", SETTING_FILENAME);
        let to_json_file = serde_json::to_string_pretty(&setting).unwrap();
        let mut file = std::fs::File::create(get_file_path(SETTING_FILENAME)).unwrap();
        file.write_all(to_json_file.as_bytes()).unwrap();
    }
    setting
});
