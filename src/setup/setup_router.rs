use actix_web::{Responder, HttpResponse, get};
use crate::state::{ForumRSState};
use serde_json::json;
use handlebars::Handlebars;

#[get("/")]
pub async fn welcome(data: actix_web::web::Data<ForumRSState>) -> impl Responder {
    let result : String = (&data.hbs).render("setup/welcome", &json!({"test": "test"})).unwrap();
    HttpResponse::Ok().body(result)
}

#[get("/login")]
pub async fn login(data: actix_web::web::Data<ForumRSState>) -> impl Responder {
    let result : String = (&data.hbs).render("setup/login", &json!({"test": "test"})).unwrap();
    HttpResponse::Ok().body(result)
}