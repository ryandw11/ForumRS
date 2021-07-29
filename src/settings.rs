use serde::{Serialize, Deserialize};
use std::fs;

#[derive(Deserialize, Serialize)]
pub struct BaseSettings {
    pub(crate) new_setup: bool,
    pub(crate) ip: String,
    pub(crate) port: u16,
    pub(crate) database_type: String,
    pub(crate) mysql_settings: Option<MysqlSettings>,
    pub(crate) sql_settings: Option<SqlSettings>
}

impl BaseSettings {
    pub fn create_default() -> BaseSettings {
        BaseSettings {
            new_setup: true,
            ip: "127.0.0.1".to_string(),
            port: 8080,
            database_type: "UnInitalized".to_string(),
            mysql_settings: None,
            sql_settings: None
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct MysqlSettings {
    pub(crate) database_url: String
}

#[derive(Deserialize, Serialize)]
pub struct SqlSettings {
    pub(crate) file_location: String
}

pub struct SettingsManager {}

impl SettingsManager {
    pub fn get_settings() -> BaseSettings {
        let setting : BaseSettings = toml::from_str(fs::read_to_string("settings.toml").unwrap().as_str()).unwrap();
        setting
    }

    pub fn save_settings(base_settings: &BaseSettings) {
        let str_setting = toml::to_string(base_settings).unwrap();
        fs::write("settings.toml", str_setting);
    }

    pub fn settings_exist() -> bool {
        fs::read("settings.toml").is_ok()
    }
}