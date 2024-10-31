use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::storage;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Auth {
    pub token: String,
    pub name: String,
}

impl Default for Auth {
    fn default() -> Self {
        if let Ok(this) = storage::load_from_path::<Self>(storage::auth_path::<Self>()) {
            println!("Loaded auth from: {:?}", &this);
            return this;
        }

        let this = Self {
            name: "".into(),
            token: "".into(),
        };

        storage::save(&this);

        this
    }
}

pub type AuthConfig = Arc<Mutex<Option<Auth>>>;
