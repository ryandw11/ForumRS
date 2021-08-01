use handlebars::Handlebars;
use std::sync::{Mutex, Arc};
use uuid::Uuid;

/// This is the state of the Application for the Setup process.
#[derive(Clone)]
pub struct SetupForumRSState {
    /// The instance of Handlebars (Constant)
    pub hbs: Handlebars<'static>,
    /// The setup UUID (Constant)
    pub setup_code: Uuid,
    /// The session id of the setup user.
    pub setup_session: Arc<Mutex<Option<Uuid>>>,
}