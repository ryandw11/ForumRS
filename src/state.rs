use handlebars::Handlebars;
use std::sync::Mutex;
use uuid::Uuid;

pub struct SetupForumRSState {
    pub hbs: Handlebars<'static>,
    pub setup_code: Uuid,
    pub setup_session: Mutex<Option<Uuid>>,
}