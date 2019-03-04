use crate::auto_save::ShouldSave;

use serde::Serialize;

/// Super simplistic token-based authentification.
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Auth {
    pub token: String,
    #[serde(skip)]
    pub validated: bool,
    #[serde(skip)]
    pub validating: bool,
    #[serde(skip)]
    pub updated: bool,
}

impl Auth {
    pub fn valid(&self) -> bool {
        self.validated
    }
    pub fn should_validate(&self) -> bool {
        !self.validated && !self.token.is_empty() && !self.validating
    }
    pub fn set_validating(&mut self) {
        self.validating = false;
        self.validating = true;
        self.updated = false;
    }
    pub fn set_validated(&mut self, valid: bool) {
        if valid {
            self.validated = true;
            self.validating = false;
            self.updated = true;
        } else {
            self.validated = false;
            self.validating = false;
            self.updated = true;
            self.token = String::default();
        }
    }
}

impl ShouldSave for Auth {
    fn save_ready(&self) -> bool {
        self.updated
    }
    fn set_save_ready(&mut self, ready: bool) {
        self.updated = ready;
    }
}
