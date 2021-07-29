use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use crate::settings::{BaseSettings, SqlSettings, SettingsManager};

#[macro_use]
extern crate diesel;
extern crate serde;


pub mod settings;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let new_setup: bool;
    let base_settings : BaseSettings;

    if SettingsManager::settings_exist() {
        new_setup = false;
        base_settings = SettingsManager::get_settings();
    }
    else {
        new_setup = true;
        println!("Detecting new setup! Creating settings file.");
        base_settings = BaseSettings::create_default();
        settings::SettingsManager::save_settings(&base_settings);
    }

    println!("Starting ForumRS on port {}.", base_settings.port);

    HttpServer::new(|| {
        App::new()
            .service(hello)
    }).bind(format!("{}:{}", base_settings.ip, base_settings.port))?
        .run()
        .await
}
