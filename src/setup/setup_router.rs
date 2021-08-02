use std::borrow::Borrow;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use actix_web::{get, http, HttpMessage, HttpRequest, HttpResponse, post, Responder, web};
use actix_web::web::{Bytes, Form};
use handlebars::Handlebars;
use regex::Regex;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::settings::{BaseSettings, SettingsManager, SSLSettings, CaptchaSettings};
use crate::setup::setup::SetupStage::{General, Security, Storage};
use crate::state::SetupForumRSState;
use std::path::Path;

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
    let port_num_opt = form.port.parse::<u16>();
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