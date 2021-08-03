use serde::{Serialize, Deserialize};
use std::{fs, fmt};

use crate::setup::setup::SetupStage;
use crate::setup::setup::SetupStage::{General};
use std::fmt::Formatter;
use crate::settings::DatabaseType::SQLite;

/**
   Base Settings are the base settings for the website.
*/
#[derive(Deserialize, Serialize, Debug)]
pub struct BaseSettings {
    /// The name of the website
    pub(crate) name: String,
    /// The domain of the website.
    pub(crate) domain: String,
    /// The ip of the website: (ex: 127.0.0.1)
    pub(crate) ip: String,
    /// The port of the website: (ex: 8080)
    pub(crate) port: u32,
    /// The type of database to be used.
    pub(crate) database_type: DatabaseType,
    /// If the program should use SSL
    pub(crate) use_sll: bool,
    /// If the program should use google reCAPTCHA v3.
    pub(crate) use_captcha: bool,
    // Responsible for storing data about setup.
    /// If the program is being setup for the first time.
    pub(crate) new_setup: bool,
    /// The current setup stage.
    pub(crate) setup_stage: Option<SetupStage>,
    //
    // Setting Sections
    //
    /// The MySQL settings. (Only exist if database_type is MySQL)
    pub(crate) mysql_settings: Option<MysqlSettings>,
    /// The SQLite settings. (Only exist if database_type is SQLite)
    pub(crate) sql_settings: Option<SqlSettings>,
    /// The postgre settings. (Only exist if database_type is PostgreSQL)
    pub(crate) postgre_settings: Option<PostgreSQLSettings>,
    /// The options for SSL.
    pub(crate) ssl_settings: Option<SSLSettings>,
    /// The settings for google reCAPTCHA v3.
    pub(crate) captcha_settings: Option<CaptchaSettings>,
}

impl BaseSettings {
    pub fn create_default() -> BaseSettings {
        BaseSettings {
            name: "UnInitalized".to_string(),
            domain: "forumrs.example.com".to_string(),
            ip: "127.0.0.1".to_string(),
            port: 8080,
            database_type: SQLite,
            mysql_settings: None,
            sql_settings: None,
            postgre_settings: None,
            use_sll: false,
            ssl_settings: None,
            use_captcha: false,
            captcha_settings: None,
            new_setup: true,
            setup_stage: Some(General),
        }
    }
}

/// The types of databases that ForumRS supports.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum DatabaseType {
    MySQL,
    SQLite,
    PostgreSQL
}

impl fmt::Display for DatabaseType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self))
    }
}

impl DatabaseType {
    /// Get the lowercase string of the enum.
    fn to_lowercase_string(&self) -> String {
        format!("{}", self).to_string().to_lowercase()
    }
}

/**
   The settings for a mysql database.
*/
#[derive(Deserialize, Serialize, Debug)]
pub struct MysqlSettings {
    /// The URL of the database.
    pub(crate) url: String,
    /// The port of the database.
    pub(crate) port: u32,
    /// The username for the database.
    pub(crate) username: String,
    /// The password for the database (required)
    pub(crate) password: String,
    /// The name of the database.
    pub(crate) database_name: String,
}

/**
   The settings for a SQLite database.
*/
#[derive(Deserialize, Serialize, Debug)]
pub struct SqlSettings {
    /// Location of the database file.
    pub(crate) file_location: String
}

/// The settings for a PostgreSQL database.
#[derive(Deserialize, Serialize, Debug)]
pub struct PostgreSQLSettings {
    /// The URL of the database.
    pub(crate) url: String,
    /// The port of the database.
    pub(crate) port: u32,
    /// The username for the database.
    pub(crate) username: String,
    /// The password for the database (required)
    pub(crate) password: String,
    /// The name of the database.
    pub(crate) database_name: String,
}

/**
   The settings for SSL.
*/
#[derive(Deserialize, Serialize, Debug)]
pub struct SSLSettings {
    /// The private key.
    pub(crate) private_key: String,
    /// The public key.
    pub(crate) public_key: String
}

/// The settings for Google reCAPTCHA v3
#[derive(Deserialize, Serialize, Debug)]
pub struct CaptchaSettings {
    pub(crate) site_key: String,
    pub(crate) secret_key: String,
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

        match settings.database_type {
            DatabaseType::SQLite => {
                if settings.sql_settings.is_none() {
                    return Err(String::from("Database is marked as SQL but a SQL settings section does not exist!"));
                }
            },
            DatabaseType::MySQL => {
                if settings.mysql_settings.is_none() {
                    return Err(String::from("Database is marked as MySQL but a MySQL settings section does not exist!"));
                }
            },
            DatabaseType::PostgreSQL => {
                if settings.postgre_settings.is_none() {
                    return Err(String::from("Database is marked as PostgreSQL but a PostgreSQL settings section does not exist!"));
                }
            }
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
        if toml::to_string(base_settings).is_err() {
            println!("{:?}", base_settings);
            println!("{}", toml::to_string(base_settings).unwrap_err());
        }
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