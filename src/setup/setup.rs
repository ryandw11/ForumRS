use std::fmt;
use std::fmt::Formatter;

#[derive(Deserialize, Serialize, Debug)]
pub enum SetupStage {
    General,
}

/**
    Convert the SetupStage enum to string.
*/
impl fmt::Display for SetupStage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{}", self).to_string().to_lowercase())
    }
}