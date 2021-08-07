#[macro_use]
extern crate diesel;
extern crate handlebars;
#[macro_use]
extern crate serde;
extern crate serde_json;

use std::sync::{Arc, Mutex};

use actix_files as actixfs;
use actix_web::{App, get, HttpResponse, HttpServer, post, Responder, web};
use handlebars::{Context, Handlebars, Helper, Output, Renderable, RenderContext, RenderError};
use uuid::Uuid;

use crate::settings::{BaseSettings, SettingsManager, SqlSettings};
use crate::state::SetupForumRSState;


pub mod settings;
pub mod setup;
pub mod state;
pub mod schema;

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
    handlebars.register_helper("ifEq", Box::new(if_eq));

    handlebars.register_templates_directory(".hbs", "./views")
        .unwrap();

    if base_settings.new_setup {
        let console_session_login = Uuid::new_v4();
        let setup_form_state = SetupForumRSState {
            hbs: handlebars.clone(),
            setup_code: console_session_login,
            setup_session: Arc::new(Mutex::new(None)),
        };
        println!("The Configuration Login code is: {}", console_session_login);
        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(setup_form_state.clone()))
                .service(actixfs::Files::new("/public", "./public"))
                .service(setup::setup_router::welcome)
                .service(setup::setup_router::login)
                .service(setup::setup_router::auth_login)
                .service(setup::setup_router::general)
                .service(setup::setup_router::auth_general)
                .service(setup::setup_router::security)
                .service(setup::setup_router::auth_security)
                .service(setup::setup_router::storage)
                .service(setup::setup_router::auth_storage)
                .service(setup::setup_router::finished)
                .service(setup::setup_router::auth_finished)
                .service(setup::setup_router::existing_storage)
                .service(setup::setup_router::auth_existing_storage_migrate)
                .service(setup::setup_router::auth_existing_storage_reset)
                .service(setup::setup_router::account_creation)
                .service(setup::setup_router::auth_account_creation)
        }).bind(format!("{}:{}", base_settings.ip, base_settings.port))?
            .run()
            .await
    } else {
        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(SetupForumRSState {
                    hbs: handlebars.clone(),
                    setup_code: Uuid::new_v4(),
                    setup_session: Arc::new(Mutex::new(None)),
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

/**
   Handlebars helper method to check if two things are equal to each other. Note: All values are kept as strings for the comparison.

   ```
    {{#ifEq some_value "test"}}
    <p>Some value is equal to test!</p>
    {{else}}
    <p>Some value is not equal to test!</p>
    {{/ifEq}}
   ```
*/
fn if_eq<'reg, 'rc>(h: &Helper<'reg, 'rc>, r: &'reg Handlebars, ctx: &'rc Context, rc: &mut RenderContext<'reg, 'rc>, out: &mut dyn Output) -> Result<(), RenderError> {
    let param1 = h.param(0).ok_or_else(|| RenderError::new("Param 0 not found for ifEq."))?.value().as_str();
    let param2 = h.param(1).ok_or_else(|| RenderError::new("Param 1 not found for ifEq."))?.value().as_str();

    // Check to make sure the parameters actually have a value.
    // Note: null == null is not valid.
    if param1.is_none() { return Ok(()); }
    if param2.is_none() { return Ok(()); }

    // Get either the if content or the else content.
    let tmpl = if param1.unwrap() == param2.unwrap() { h.template() } else { h.inverse() };
    // Render if found.
    match tmpl {
        Some(t) => t.render(r, ctx, rc, out),
        None => Ok(()),
    }
}
