use serde::{Serialize, Deserialize};
use std::fs;

use crate::setup::setup::SetupStage;
use crate::setup::setup::SetupStage::{General};

/**
   Base Settings are the base settings for the website.
*/
#[derive(Deserialize, Serialize)]
pub struct BaseSettings {
    /// The name of the website
    pub(crate) name: String,
    /// The ip of the website: (ex: 127.0.0.1)
    pub(crate) ip: String,
    /// The port of the website: (ex: 8080)
    pub(crate) port: u16,
    /// The type of database to be used.
    pub(crate) database_type: String,
    /// The MySQL settings. (Only exist if database_type is MySQL)
    pub(crate) mysql_settings: Option<MysqlSettings>,
    /// The SQLite settings. (Only exist if database_type is SQLite)
    pub(crate) sql_settings: Option<SqlSettings>,
    /// If the program should use SSL
    pub(crate) use_sll: bool,
    /// The options for SSL.
    pub(crate) ssl_settings: Option<SSLSettings>,
    // Responsible for storing data about setup.
    /// If the program is being setup for the first time.
    pub(crate) new_setup: bool,
    /// The current setup stage.
    pub(crate) setup_stage: Option<SetupStage>,
}

impl BaseSettings {
    pub fn create_default() -> BaseSettings {
        BaseSettings {
            name: "UnInitalized".to_string(),
            ip: "127.0.0.1".to_string(),
            port: 8080,
            database_type: "UnInitalized".to_string(),
            mysql_settings: None,
            sql_settings: None,
            use_sll: false,
            ssl_settings: None,
            new_setup: true,
            setup_stage: Some(General),
        }
    }
}

/**
   The settings for a mysql database.
*/
#[derive(Deserialize, Serialize)]
pub struct MysqlSettings {
    /// The URL of the database.
    pub(crate) database_url: String
}

/**
   The settings for a SQLite database.
*/
#[derive(Deserialize, Serialize)]
pub struct SqlSettings {
    /// Location of the database file.
    pub(crate) file_location: String
}

/**
   The settings for SSL.
*/
#[derive(Deserialize, Serialize)]
pub struct SSLSettings {
    /// The private key.
    pub(crate) private_key: String,
    /// The public key.
    pub(crate) public_key: String
}

/**
    The Manager that allows you to read and write settings.

    Settings are stored in the settings.toml file.
*/
pub struct SettingsManager {}

impl SettingsManager {
    /**
     This function validates the settings from the Settings.toml file.

     ## Returns
     An empty result if ok, an error message if not.
    */
    pub fn validate_settings() -> Result<(), String> {
        let setting_res = toml::from_str(fs::read_to_string("settings.toml").unwrap().as_str());
        if setting_res.is_err() {
            return Err(String::from("Invalid settings file! It is not a TOML file or is missing a section."));
        }

        let settings : BaseSettings = setting_res.unwrap();

        if settings.new_setup && settings.setup_stage.is_some() {
            return Ok(());
        }

        if !settings.new_setup && settings.setup_stage.is_some() {
            return Err(String::from("Invalid setup stage. Something has went wrong during the setup process!"));
        }

        match settings.database_type.as_str() {
            "sql" => {
                if settings.sql_settings.is_none() {
                    return Err(String::from("Database is marked as SQL but a SQL settings section does not exist!"));
                }
            },
            "mysql" => {
                if settings.mysql_settings.is_none() {
                    return Err(String::from("Database is marked as MySQL but a MySQL settings section does not exist!"));
                }
            },
            _ => return Err(String::from("Invalid database type! Please edit the settings.toml file manually to fix!"))
        }

        if settings.use_sll && settings.ssl_settings.is_none() {
            return Err(String::from("SSL is marked as being used, but there are no SSL settings present."));
        }

        Ok(())
    }

    /**
        Get the settings from the settings.toml file.
        The file will be read every time and there is not validate. Call #validate_settings() to
        validate the settings file first.
    */
    pub fn get_settings() -> BaseSettings {
        let setting : BaseSettings = toml::from_str(fs::read_to_string("settings.toml").unwrap().as_str()).unwrap();
        setting
    }

    /**
    Save settings to the settings file.
    */
    pub fn save_settings(base_settings: &BaseSettings) {
        let str_setting = toml::to_string(base_settings).unwrap();
        fs::write("settings.toml", str_setting);
    }

    /**
    Check if the settings file exists.
    */
    pub fn settings_exist() -> bool {
        fs::read("settings.toml").is_ok()
    }
}