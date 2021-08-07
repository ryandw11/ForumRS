use std::borrow::Borrow;
use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use actix_web::{get, http, HttpMessage, HttpRequest, HttpResponse, post, Responder, web};
use actix_web::rt::Arbiter;
use actix_web::web::{Bytes, Form};
use handlebars::Handlebars;
use regex::Regex;
use serde::Deserialize;
use serde_json::json;
use sqlx::{Connection, MySqlConnection, PgConnection};
use uuid::Uuid;

use crate::schema::general::ForumRSTable;
use crate::settings::{BaseSettings, CaptchaSettings, DatabaseType, MysqlSettings, PostgreSQLSettings, SettingsManager, SqlSettings, SSLSettings};
use crate::settings::DatabaseType::{MySQL, PostgreSQL, SQLite};
use crate::setup::setup::SetupStage::{ExistingStorage, Finished, General, Security, Storage, AccountCreation};
use crate::state::SetupForumRSState;

/// The welcome (index) page for the setup process.
#[get("/")]
pub async fn welcome(data: actix_web::web::Data<SetupForumRSState>) -> impl Responder {
    let result: String = (&data.hbs).render("setup/welcome", &json!({"test": "test"})).unwrap();
    HttpResponse::Ok().body(result)
}

/// The login page for the setup process.
/// If the user is already logged in, then they are directed to where they left off in
/// the setup process.
#[get("/login")]
pub async fn login(data: actix_web::web::Data<SetupForumRSState>, req: HttpRequest) -> impl Responder {
    if data.setup_session.lock().unwrap().is_some() {
        // If the session cookie does not exist, return an error.
        if req.cookie("session").is_none() {
            let result: String = (&data.hbs).render("setup/login", &json!({"session_error": "true"})).unwrap();
            return HttpResponse::Ok().body(result);
        }
        // If the session is valid, automatically return to the next location.
        if Uuid::from_str(req.cookie("session").unwrap().value()).unwrap() == data.setup_session.lock().unwrap().unwrap() {
            return HttpResponse::Found().header("Location", format!("/{}", SettingsManager::get_settings().setup_stage.unwrap())).finish();
        }
    }

    let result: String = (&data.hbs).render("setup/login", &json!({"test": "test"})).unwrap();
    let mut builder = HttpResponse::Ok();

    // If the session cookie still exists, remove it as it cannot be valid.
    if req.cookie("session").is_some() {
        builder.del_cookie(&req.cookie("session").unwrap());
    }

    builder.body(result)
}

/// The form data for the login authorization.
#[derive(Deserialize)]
pub struct AuthLoginData {
    login_code: String
}

/// The post request used to login to the setup process.
/// If the setup was already in progress, then the user is redirected to where they left off.
#[post("/auth/login")]
pub async fn auth_login(data: actix_web::web::Data<SetupForumRSState>, form: web::Form<AuthLoginData>, req: HttpRequest) -> impl Responder {
    // Redirect to the login screen if the session already exists.
    if data.setup_session.lock().unwrap().is_some() {
        return HttpResponse::Found().header("Location", "/login").finish();
    }

    // If the UUID from the form is not valid.
    if Uuid::from_str(form.login_code.as_str()).is_err() {
        return HttpResponse::Found().header("Location", "/login?err=1").finish();
    }

    if Uuid::parse_str(form.login_code.as_str()).unwrap().to_string() == data.setup_code.to_string() {
        let new_session_id = Uuid::new_v4();
        *data.setup_session.lock().unwrap() = Some(new_session_id);

        // Create a cookie for the user.
        HttpResponse::Found()
            .cookie(
                http::Cookie::build("session", new_session_id.to_string())
                    .path("/")
                    .secure(true)
                    .finish()
            )
            .header("Location", format!("/{}", SettingsManager::get_settings().setup_stage.unwrap()))
            .finish()
    } else {
        return HttpResponse::Found().header("Location", "/login?err=1").finish();
    }
}

/// Check to see if the user accessing the setup page is logged in.
///
/// If not a response is compiled that returns the user to the login page and deletes the bad cookie if it exists.
fn check_login(data: &actix_web::web::Data<SetupForumRSState>, req: HttpRequest) -> Result<(), HttpResponse> {
    if data.setup_session.lock().unwrap().is_none() {
        return Err(HttpResponse::Found().header("Location", "/login").finish());
    }

    if req.cookie("session").is_none() {
        return Err(HttpResponse::Found().header("Location", "/login").finish());
    }

    let session_pid = Uuid::parse_str(req.cookie("session").unwrap().value());

    if session_pid.is_err() {
        return Err(HttpResponse::Found().del_cookie(&req.cookie("session").unwrap()).header("Location", "/login").finish());
    }

    if data.setup_session.lock().unwrap().unwrap() == session_pid.unwrap() {
        return Ok(());
    }

    Err(HttpResponse::Found().del_cookie(&req.cookie("session").unwrap()).header("Location", "/login").finish())
}

#[get("/general")]
pub async fn general(data: actix_web::web::Data<SetupForumRSState>, req: HttpRequest) -> impl Responder {
    // Check if the user is logged in.
    let loggedin = check_login(&data, req);
    if loggedin.is_err() {
        return loggedin.unwrap_err();
    }

    // If the user is at the wrong stage, take them to the correct one.
    if !(SettingsManager::get_settings().setup_stage.unwrap() == General) {
        return HttpResponse::Found().header("Location", format!("/{}", SettingsManager::get_settings().setup_stage.unwrap())).finish();
    }

    let result: String = (&data.hbs).render("setup/general", &json!({"test": "test"})).unwrap();

    HttpResponse::Ok().body(result)
}

#[derive(Deserialize)]
pub struct AuthGeneralForm {
    name: String,
    ip: String,
    port: String,
    domain: String
}

#[post("/auth/general")]
pub async fn auth_general(data: actix_web::web::Data<SetupForumRSState>, form: web::Form<AuthGeneralForm>, req: HttpRequest) -> impl Responder {
    // Check if the user is logged in.
    let loggedin = check_login(&data, req);
    if loggedin.is_err() {
        return loggedin.unwrap_err();
    }

    // If the user is at the wrong stage, take them to the correct one.
    if !(SettingsManager::get_settings().setup_stage.unwrap() == General) {
        return HttpResponse::Found().header("Location", format!("/{}", SettingsManager::get_settings().setup_stage.unwrap())).finish();
    }

    if form.name.len() < 1 {
        return HttpResponse::Found().header("Location", "/general?err=1").finish();
    }

    if form.ip.len() < 1 {
        return HttpResponse::Found().header("Location", "/general?err=2").finish();
    }

    if form.domain.len() < 1 {
        return HttpResponse::Found().header("Location", "/general?err=4").finish();
    }

    // Validate the ip address via regex.
    let ip_regex = Regex::new(r"^(?:(?:25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9][0-9]|[0-9])\.){3}(?:25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9][0-9]|[0-9])$").unwrap();
    if !ip_regex.is_match(form.ip.as_str()) {
        return HttpResponse::Found().header("Location", "/general?err=2").finish();
    }

    // Check if the port is an integer.
    let port_num_opt = form.port.parse::<u32>();
    if port_num_opt.is_err() {
        return HttpResponse::Found().header("Location", "/general?err=3").finish();
    }

    // Check if the port number is even valid.
    let port_num = port_num_opt.unwrap();
    if port_num > 65535 || port_num < 1 {
        return HttpResponse::Found().header("Location", "/general?err=3").finish();
    }

    let mut settings = SettingsManager::get_settings();
    settings.name = form.name.clone();
    settings.ip = form.ip.clone();
    settings.port = port_num;
    settings.domain = form.domain.clone();
    settings.setup_stage = Some(Security);

    SettingsManager::save_settings(&settings);

    return HttpResponse::Found().header("Location", "/security").finish();
}

#[get("/security")]
pub async fn security(data: actix_web::web::Data<SetupForumRSState>, req: HttpRequest) -> impl Responder {
    // Check if the user is logged in.
    let loggedin = check_login(&data, req);
    if loggedin.is_err() {
        return loggedin.unwrap_err();
    }

    // If the user is at the wrong stage, take them to the correct one.
    if !(SettingsManager::get_settings().setup_stage.unwrap() == Security) {
        return HttpResponse::Found().header("Location", format!("/{}", SettingsManager::get_settings().setup_stage.unwrap())).finish();
    }

    let result: String = (&data.hbs).render("setup/security", &json!({"test": "test"})).unwrap();

    HttpResponse::Ok().body(result)
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct AuthSecurityForm {
    pub(crate) useSSL: Option<String>,
    pub(crate) privateKey: Option<String>,
    pub(crate) publicKey: Option<String>,
    pub(crate) useCaptch: Option<String>,
    pub(crate) siteKey: Option<String>,
    pub(crate) secretKey: Option<String>,
}

#[post("/auth/security")]
pub async fn auth_security(data: actix_web::web::Data<SetupForumRSState>, form: Form<AuthSecurityForm>, req: HttpRequest) -> impl Responder {
    // Check if the user is logged in.
    let loggedin = check_login(&data, req);
    if loggedin.is_err() {
        return loggedin.unwrap_err();
    }

    // If the user is at the wrong stage, take them to the correct one.
    if !(SettingsManager::get_settings().setup_stage.unwrap() == Security) {
        return HttpResponse::Found().header("Location", format!("/{}", SettingsManager::get_settings().setup_stage.unwrap())).finish();
    }

    let mut settings = SettingsManager::get_settings();

    if form.useSSL.is_some() && form.useSSL.as_ref().unwrap() == "on" {
        // The keys need to exist.
        if form.privateKey.is_none() || form.publicKey.is_none() {
            return HttpResponse::Found().header("Location", "/security?err=1").finish();
        }
        let private_key = form.privateKey.as_ref().unwrap().clone();
        let public_key = form.publicKey.as_ref().unwrap().clone();

        let key = Regex::new(r"^.*\.(pem|PEM|asn1|ASN1)$").unwrap();
        if !key.is_match(private_key.as_str()) {
            return HttpResponse::Found().header("Location", "/security?err=2").finish();
        }

        if !key.is_match(public_key.as_str()) {
            return HttpResponse::Found().header("Location", "/security?err=3").finish();
        }

        if !Path::new(private_key.as_str()).exists() {
            return HttpResponse::Found().header("Location", "/security?err=4").finish();
        }

        if !Path::new(public_key.as_str()).exists() {
            return HttpResponse::Found().header("Location", "/security?err=5").finish();
        }

        settings.use_sll = true;

        let ssl_settings = SSLSettings {
            private_key,
            public_key
        };

        settings.ssl_settings = Some(ssl_settings);
    } else {
        settings.use_sll = false;
    }

    if form.useCaptch.is_some() && form.useCaptch.as_ref().unwrap() == "on" {
        // The keys need to exist.
        if form.siteKey.is_none() || form.secretKey.is_none() {
            return HttpResponse::Found().header("Location", "/security?err=6").finish();
        }
        let site_key = form.siteKey.as_ref().unwrap().clone();
        let secret_key = form.secretKey.as_ref().unwrap().clone();

        // I think the keys are always 40 in length.
        if site_key.len() != 40 || secret_key.len() != 40 {
            return HttpResponse::Found().header("Location", "/security?err=7").finish();
        }

        settings.use_captcha = true;

        let captcha_settings = CaptchaSettings {
            site_key,
            secret_key
        };

        settings.captcha_settings = Some(captcha_settings);
    }
    else {
        settings.use_captcha = false;
    }

    settings.setup_stage = Some(Storage);

    SettingsManager::save_settings(&settings);

    HttpResponse::Found().header("Location", format!("/{}", SettingsManager::get_settings().setup_stage.unwrap())).finish()
}

#[get("/storage")]
pub async fn storage(data: actix_web::web::Data<SetupForumRSState>, req: HttpRequest) -> impl Responder {
    // Check if the user is logged in.
    let loggedin = check_login(&data, req);
    if loggedin.is_err() {
        return loggedin.unwrap_err();
    }

    // If the user is at the wrong stage, take them to the correct one.
    if !(SettingsManager::get_settings().setup_stage.unwrap() == Storage) {
        return HttpResponse::Found().header("Location", format!("/{}", SettingsManager::get_settings().setup_stage.unwrap())).finish();
    }

    let result: String = (&data.hbs).render("setup/storage", &json!({"test": "test"})).unwrap();

    HttpResponse::Ok().body(result)
}

/// The form data for the auth.
#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct AuthStorageForm {
    pub(crate) dbType: DatabaseType,

    pub(crate) sqlName: Option<String>,

    pub(crate) mysqlURL: Option<String>,
    pub(crate) mysqlPort: Option<u32>,
    pub(crate) mysqlUsername: Option<String>,
    pub(crate) mysqlPassword: Option<String>,
    pub(crate) mysqlDbName: Option<String>,

    pub(crate) postURL: Option<String>,
    pub(crate) postPort: Option<u32>,
    pub(crate) postUsername: Option<String>,
    pub(crate) postPassword: Option<String>,
    pub(crate) postDbName: Option<String>,
}

/// The backed process for setting up the storage of the program.
#[post("/auth/storage")]
pub async fn auth_storage(data: actix_web::web::Data<SetupForumRSState>, form: Form<AuthStorageForm>, req: HttpRequest) -> impl Responder {
    // Check if the user is logged in.
    let loggedin = check_login(&data, req);
    if loggedin.is_err() {
        return loggedin.unwrap_err();
    }

    // If the user is at the wrong stage, take them to the correct one.
    if !(SettingsManager::get_settings().setup_stage.unwrap() == Storage) {
        return HttpResponse::Found().header("Location", format!("/{}", SettingsManager::get_settings().setup_stage.unwrap())).finish();
    }

    let mut settings = SettingsManager::get_settings();

    match form.dbType.clone() {
        DatabaseType::SQLite => {
            if form.sqlName.is_none() {
                return HttpResponse::Found().header("Location", "/storage?err=1").finish();
            }

            let sql_name = form.sqlName.as_ref().unwrap().clone();

            let sql_regex = Regex::new(r"^.*\.(db)$").unwrap();
            if !sql_regex.is_match(&sql_name) {
                return HttpResponse::Found().header("Location", "/storage?err=1").finish();
            }

            settings.database_type = SQLite;
            settings.sql_settings = Some(SqlSettings {
                file_location: sql_name.clone()
            });


            if Path::new(sql_name.as_str()).exists() {
                settings.setup_stage = Some(ExistingStorage);
                SettingsManager::save_settings(&settings);
                return HttpResponse::Found().header("Location", "/existingstorage").finish();
            }

            settings.setup_stage = Some(AccountCreation);
            SettingsManager::save_settings(&settings);
            HttpResponse::Found().header("Location", "/accountcreation").finish();
        },
        DatabaseType::MySQL => {
            if form.mysqlURL.is_none() || form.mysqlDbName.is_none() || form.mysqlPassword.is_none() || form.mysqlPort.is_none()
                || form.mysqlUsername.is_none() {
                return HttpResponse::Found().header("Location", "/storage?err=2").finish();
            }

            let mysql_url = form.mysqlURL.as_ref().unwrap().clone();
            let mysql_port = form.mysqlPort.as_ref().unwrap().clone();
            let mysql_username = form.mysqlUsername.as_ref().unwrap().clone();
            let mysql_password = form.mysqlPassword.as_ref().unwrap().clone();
            let mysql_db_name = form.mysqlDbName.as_ref().unwrap().clone();

            let connection = MySqlConnection::connect(&format!("mysql://{}:{}@{}:{}", mysql_username, mysql_password,
            mysql_url, mysql_port)).await;

            if connection.is_err() {
                let connection_err = connection.unwrap_err();
                if connection_err.as_database_error().is_some() {
                    println!("[WARN] The following error occurred when connecting to the MySQL database: {:?}", connection_err.as_database_error().unwrap().message());
                }
                else {
                    println!("[WARN] The following error occurred when connecting to the MySQL database: {:?}", connection_err);
                    println!("[WARN] Check to make sure ForumRS can access the specified MySQL server.");
                }

                return HttpResponse::Found().header("Location", "/storage?err=3").finish();
            }

            settings.database_type = MySQL;
            settings.mysql_settings = Some(MysqlSettings {
                url: mysql_url,
                port: mysql_port,
                username: mysql_username,
                password: mysql_password,
                database_name: mysql_db_name.clone()
            });

            let mut con = connection.unwrap();
            let found_database = sqlx::query("SELECT SCHEMA_NAME FROM INFORMATION_SCHEMA.SCHEMATA WHERE SCHEMA_NAME = ?")
                .bind(mysql_db_name.clone())
                .fetch_one(&mut con).await;

            con.close().await.unwrap();

            if found_database.is_ok() {
                settings.setup_stage = Some(ExistingStorage);
                    SettingsManager::save_settings(&settings);
                    return HttpResponse::Found().header("Location", "/existingstorage").finish();
            }

            settings.setup_stage = Some(AccountCreation);
            SettingsManager::save_settings(&settings);
            return HttpResponse::Found().header("Location", "/accountcreation").finish();
        },
        DatabaseType::PostgreSQL => {
            if form.postURL.is_none() || form.postDbName.is_none() || form.postPassword.is_none() || form.postPort.is_none()
                || form.postUsername.is_none() {
                return HttpResponse::Found().header("Location", "/storage?err=4").finish();
            }

            let post_url = form.postURL.as_ref().unwrap().clone();
            let post_port = form.postPort.as_ref().unwrap().clone();
            let post_username = form.postUsername.as_ref().unwrap().clone();
            let post_password = form.postPassword.as_ref().unwrap().clone();
            let post_db_name = form.postDbName.as_ref().unwrap().clone();

            let connection = PgConnection::connect(&format!("postgresql://{}:{}@{}:{}", post_username, post_password,
                                                            post_url, post_port)).await;

            if connection.is_err() {
                let connection_err = connection.unwrap_err();
                if connection_err.as_database_error().is_some() {
                    println!("[WARN] The following error occurred when connecting to the PostgreSQL database: {:?}", connection_err.as_database_error().unwrap().message());
                }
                else {
                    println!("[WARN] The following error occurred when connecting to the PostgreSQL database: {:?}", connection_err);
                    println!("[WARN] Check to make sure ForumRS can access the specified PostgreSQL server.");
                }

                return HttpResponse::Found().header("Location", "/storage?err=5").finish();
            }

            settings.database_type = PostgreSQL;
            settings.postgre_settings = Some(PostgreSQLSettings {
                url: post_url,
                port: post_port,
                username: post_username,
                password: post_password,
                database_name: post_db_name.clone()
            });

            let mut con = connection.unwrap();
            let found_database = sqlx::query(&format!("SELECT datname FROM pg_catalog.pg_database WHERE datname = '{}';", post_db_name))
                .bind(post_db_name.clone())
                .fetch_one(&mut con).await;

            con.close().await.unwrap();

            if found_database.is_ok() {
                settings.setup_stage = Some(ExistingStorage);
                SettingsManager::save_settings(&settings);
                return HttpResponse::Found().header("Location", "/existingstorage").finish();
            }

            settings.setup_stage = Some(AccountCreation);
            SettingsManager::save_settings(&settings);
            return HttpResponse::Found().header("Location", "/accountcreation").finish();
        }
    }

    HttpResponse::Found().header("Location", "/accountcreation").finish()
}

#[get("/existingstorage")]
pub async fn existing_storage(data: actix_web::web::Data<SetupForumRSState>, req: HttpRequest) -> impl Responder {
    // Check if the user is logged in.
    let loggedin = check_login(&data, req);
    if loggedin.is_err() {
        return loggedin.unwrap_err();
    }

    // If the user is at the wrong stage, take them to the correct one.
    if !(SettingsManager::get_settings().setup_stage.unwrap() == ExistingStorage) {
        return HttpResponse::Found().header("Location", format!("/{}", SettingsManager::get_settings().setup_stage.unwrap())).finish();
    }

    let result: String = (&data.hbs).render("setup/existingstorage", &json!({"test": "test"})).unwrap();

    HttpResponse::Ok().body(result)
}

#[post("/auth/existingstorage/migrate")]
pub async fn auth_existing_storage_migrate(data: actix_web::web::Data<SetupForumRSState>, req: HttpRequest) -> impl Responder {
    // Check if the user is logged in.
    let loggedin = check_login(&data, req);
    if loggedin.is_err() {
        return loggedin.unwrap_err();
    }

    // If the user is at the wrong stage, take them to the correct one.
    if !(SettingsManager::get_settings().setup_stage.unwrap() == ExistingStorage) {
        return HttpResponse::Found().header("Location", format!("/{}", SettingsManager::get_settings().setup_stage.unwrap())).finish();
    }

    let mut settings = SettingsManager::get_settings();
    settings.setup_stage = Some(Finished);

    SettingsManager::save_settings(&settings);

    HttpResponse::Found().header("Location", "/finished").finish()
}

#[post("/auth/existingstorage/reset")]
pub async fn auth_existing_storage_reset(data: actix_web::web::Data<SetupForumRSState>, req: HttpRequest) -> impl Responder {
    // Check if the user is logged in.
    let loggedin = check_login(&data, req);
    if loggedin.is_err() {
        return loggedin.unwrap_err();
    }

    // If the user is at the wrong stage, take them to the correct one.
    if !(SettingsManager::get_settings().setup_stage.unwrap() == ExistingStorage) {
        return HttpResponse::Found().header("Location", format!("/{}", SettingsManager::get_settings().setup_stage.unwrap())).finish();
    }

    let mut settings = SettingsManager::get_settings();

    match settings.database_type {
        DatabaseType::SQLite => {
            std::fs::remove_file(Path::new(settings.sql_settings.as_ref().unwrap().file_location.as_str())).unwrap();
        },
        DatabaseType::MySQL => {
            let mysql_settings = settings.mysql_settings.as_ref().unwrap();
            let mut connection = MySqlConnection::connect(&format!("mysql://{}:{}@{}:{}", mysql_settings.username, mysql_settings.password,
                                                               mysql_settings.url, mysql_settings.port)).await.unwrap();

            let found_database = sqlx::query(&format!("DROP DATABASE IF EXISTS {};", mysql_settings.database_name))
                .execute(&mut connection).await.unwrap();
            connection.close().await.unwrap();
        },
        DatabaseType::PostgreSQL => {
            let post_settings = settings.postgre_settings.as_ref().unwrap();
            let mut connection = PgConnection::connect(&format!("postgresql://{}:{}@{}:{}", post_settings.username, post_settings.password,
                                                            post_settings.url, post_settings.port)).await.unwrap();

            // Drop the database if it exists.
            let found_database = sqlx::query(&format!("DROP DATABASE IF EXISTS \"{}\";", post_settings.database_name))
                .execute(&mut connection).await.unwrap();

            connection.close().await.unwrap();
        }
    }

    settings.setup_stage = Some(AccountCreation);

    SettingsManager::save_settings(&settings);

    HttpResponse::Found().header("Location", "/accountcreation").finish()
}

#[get("/accountcreation")]
pub async fn account_creation(data: actix_web::web::Data<SetupForumRSState>, req: HttpRequest) -> impl Responder {
    // Check if the user is logged in.
    let loggedin = check_login(&data, req);
    if loggedin.is_err() {
        return loggedin.unwrap_err();
    }

    // If the user is at the wrong stage, take them to the correct one.
    if !(SettingsManager::get_settings().setup_stage.unwrap() == AccountCreation) {
        return HttpResponse::Found().header("Location", format!("/{}", SettingsManager::get_settings().setup_stage.unwrap())).finish();
    }

    let result: String = (&data.hbs).render("setup/accountcreation", &json!({"test": "test"})).unwrap();

    HttpResponse::Ok().body(result)
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct AuthAccountCreation {
    pub username: String,
    pub email: String,
    pub password: String,
    pub confirmPassword: String,
}

#[post("/auth/accountcreation")]
pub async fn auth_account_creation(data: actix_web::web::Data<SetupForumRSState>, form: Form<AuthAccountCreation>, req: HttpRequest) -> impl Responder {
    // Check if the user is logged in.
    let loggedin = check_login(&data, req);
    if loggedin.is_err() {
        return loggedin.unwrap_err();
    }

    // If the user is at the wrong stage, take them to the correct one.
    if !(SettingsManager::get_settings().setup_stage.unwrap() == Storage) {
        return HttpResponse::Found().header("Location", format!("/{}", SettingsManager::get_settings().setup_stage.unwrap())).finish();
    }

    let mut settings = SettingsManager::get_settings();

    unimplemented!()
}

#[get("/finished")]
pub async fn finished(data: actix_web::web::Data<SetupForumRSState>, req: HttpRequest) -> impl Responder {
    // Check if the user is logged in.
    let loggedin = check_login(&data, req);
    if loggedin.is_err() {
        return loggedin.unwrap_err();
    }

    // If the user is at the wrong stage, take them to the correct one.
    if !(SettingsManager::get_settings().setup_stage.unwrap() == Finished) {
        return HttpResponse::Found().header("Location", format!("/{}", SettingsManager::get_settings().setup_stage.unwrap())).finish();
    }

    let result: String = (&data.hbs).render("setup/finished", &json!({"test": "test"})).unwrap();

    HttpResponse::Ok().body(result)
}

#[post("/auth/finished")]
pub async fn auth_finished(data: actix_web::web::Data<SetupForumRSState>, req: HttpRequest) -> impl Responder {
    // Check if the user is logged in.
    let loggedin = check_login(&data, req);
    if loggedin.is_err() {
        return loggedin.unwrap_err();
    }

    // If the user is at the wrong stage, take them to the correct one.
    if !(SettingsManager::get_settings().setup_stage.unwrap() == Finished) {
        return HttpResponse::Found().header("Location", format!("/{}", SettingsManager::get_settings().setup_stage.unwrap())).finish();
    }

    let mut settings = SettingsManager::get_settings();
    settings.new_setup = false;
    settings.setup_stage = None;

    SettingsManager::save_settings(&settings);

    println!("[INFO] Configuration complete. Please restart the server to launch ForumRS.");

    Arbiter::current().stop();

    std::process::exit(0);
}