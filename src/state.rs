use handlebars::Handlebars;
use std::sync::Mutex;

pub struct ForumRSState {
    pub hbs: Handlebars<'static>
}