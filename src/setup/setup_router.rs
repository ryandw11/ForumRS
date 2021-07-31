use actix_web::{Responder, HttpResponse, get, post, web, http, HttpRequest, HttpMessage};
use crate::state::{SetupForumRSState};
use serde::Deserialize;
use serde_json::json;
use handlebars::Handlebars;
use uuid::Uuid;
use std::str::FromStr;
use crate::settings::{BaseSettings, SettingsManager};
use std::sync::Mutex;
use std::borrow::Borrow;

#[get("/")]
pub async fn welcome(data: actix_web::web::Data<SetupForumRSState>) -> impl Responder {
    let result : String = (&data.hbs).render("setup/welcome", &json!({"test": "test"})).unwrap();
    HttpResponse::Ok().body(result)
}

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
    HttpResponse::Ok().body(result)
}

#[derive(Deserialize)]
pub struct AuthLoginData {
    login_code: String
}

#[post("/auth/login")]
pub async fn auth_login(data: actix_web::web::Data<SetupForumRSState>, form: web::Form<AuthLoginData>, req: HttpRequest) -> impl Responder {
    // Redirect to the login screen if the session already exists.
    if data.setup_session.lock().unwrap().is_some() {
        return HttpResponse::Found().header("Location", "/login").finish();
    }

    println!("{}", form.login_code);

    // If the UUID from the form is not valid.
    if Uuid::from_str(form.login_code.as_str()).is_err() {
        return HttpResponse::Found().header("Location", "/login?err=1").finish();
    }

    if Uuid::parse_str(form.login_code.as_str()).unwrap().to_string() == data.setup_code.to_string() {
        let new_session_id = Uuid::new_v4();
        *data.setup_session.lock().unwrap() = Some(new_session_id);
        HttpResponse::Found()
            .cookie(
                http::Cookie::build("session", new_session_id.to_string())
                    .path("/")
                    .secure(true)
                    .finish()
            )
            .header("Location", "/general")
            .finish()
    } else {
        return HttpResponse::Found().header("Location", format!("/login?err={}----{}", form.login_code, data.setup_code.to_string())).finish()
    }
}