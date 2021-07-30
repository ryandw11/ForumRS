#[macro_use]
extern crate diesel;
extern crate handlebars;
#[macro_use]
extern crate serde;
extern crate serde_json;

use std::sync::{Arc, Mutex};

use actix_files as actixfs;
use actix_web::{App, get, HttpResponse, HttpServer, post, Responder, web};
use handlebars::{Context, Handlebars, Helper, Output, RenderContext, RenderError};

use crate::settings::{BaseSettings, SettingsManager, SqlSettings};
use crate::state::ForumRSState;


// type ForumWebData = web::Data<Arc<Mutex<ForumRSState<'static>>>>;


pub mod settings;
pub mod setup;
pub mod state;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let new_setup: bool;
    let base_settings: BaseSettings;

    // Check if the settings exist.
    if SettingsManager::settings_exist() {
        new_setup = false;
        let valid_settings = SettingsManager::validate_settings();
        if valid_settings.is_err() {
            println!("[ERROR] An error has occurred when trying to start ForumRS!");
            println!("[ERROR] {}", valid_settings.unwrap_err());
            println!("[ERROR] If this is your first time using ForumRS, then delete the settings.toml file.");
            panic!("An unexpected error has occurred! Please check the above logs.");
        }
        base_settings = SettingsManager::get_settings();
    } else {
        new_setup = true;
        println!("Detecting new setup! Creating settings file.");
        base_settings = BaseSettings::create_default();
        settings::SettingsManager::save_settings(&base_settings);
    }

    println!("Starting ForumRS on port {}.", base_settings.port);

    let mut handlebars = Handlebars::new();
    handlebars.set_dev_mode(true);
    handlebars.register_helper("get_lang", Box::new(get_lang));

    handlebars.register_templates_directory(".hbs", "./views")
        .unwrap();

    if base_settings.new_setup {
        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(ForumRSState {
                    hbs: handlebars.clone()
                }))
                .service(actixfs::Files::new("/public", "./public"))
                .service(setup::setup_router::welcome)
                .service(setup::setup_router::login)
        }).bind(format!("{}:{}", base_settings.ip, base_settings.port))?
            .run()
            .await
    } else {
        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(ForumRSState {
                    hbs: handlebars.clone()
                }))
                .service(actixfs::Files::new(".", "./public").show_files_listing())
                .service(setup::setup_router::welcome)
                .service(setup::setup_router::login)
        }).bind(format!("{}:{}", base_settings.ip, base_settings.port))?
            .run()
            .await
    }
}

fn get_lang(h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> Result<(), RenderError> {
    out.write("en")?;
    Ok(())
}
