use actix_web::{Responder, HttpResponse, get, post, web, http, HttpRequest, HttpMessage};
use crate::state::{SetupForumRSState};
use serde::Deserialize;
use serde_json::json;
use handlebars::Handlebars;
use uuid::Uuid;
use std::str::FromStr;
use crate::settings::{BaseSettings, SettingsManager};
use std::sync::{Mutex, Arc};
use std::borrow::Borrow;
use crate::setup::setup::SetupStage::{General, Security};
use regex::Regex;

/// The welcome (index) page for the setup process.
#[get("/")]
pub async fn welcome(data: actix_web::web::Data<SetupForumRSState>) -> impl Responder {
    let result : String = (&data.hbs).render("setup/welcome", &json!({"test": "test"})).unwrap();
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
            let result : String = (&data.hbs).render("setup/login", &json!({"session_error": "true"})).unwrap();
            return HttpResponse::Ok().body(result);
        }
        // If the session is valid, automatically return to the next location.
        if Uuid::from_str(req.cookie("session").unwrap().value()).unwrap() == data.setup_session.lock().unwrap().unwrap() {
            return HttpResponse::Found().header("Location", format!("/{}", SettingsManager::get_settings().setup_stage.unwrap())).finish();
        }
    }

    let result : String = (&data.hbs).render("setup/login", &json!({"test": "test"})).unwrap();
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
        return HttpResponse::Found().header("Location", "/login?err=1").finish()
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

    let result : String = (&data.hbs).render("setup/general", &json!({"test": "test"})).unwrap();

    HttpResponse::Ok().body(result)
}

#[derive(Deserialize)]
pub struct AuthGeneralForm {
    name: String,
    ip: String,
    port: String
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
    settings.setup_stage = Some(Security);

    SettingsManager::save_settings(&settings);

    return HttpResponse::Found().header("Location", "/security").finish();
}