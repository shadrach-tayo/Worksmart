use std::{collections::HashMap, sync::{Arc, Mutex}};

use serde::{Deserialize, Serialize};

use crate::{get_current_date, storage};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackHistory {
    pub history: HashMap<String, u64>
}

impl Default for TrackHistory {
    fn default() -> Self {
        if let Ok(this) = storage::load_from_path::<Self>(storage::data_path().join("tracker.json")) {
            println!("Loaded auth from: {:?}", &this);
            return this;
        }

        let this = Self {
            history: HashMap::new(),
        };

        storage::save_to_path(&this, storage::data_path().join("tracker.json")).unwrap();

        this
    }
}

impl TrackHistory {
    pub fn get_track_for_today(&self) -> u64 {
        let key = get_current_date();
        self.history.get(&key).map_or(0, |value| value.to_owned())
    }

    pub fn increment_track_for_today(&mut self, timestamp: u64) {
        let key = get_current_date();
        self.history.entry(key).and_modify(|current_time| *current_time += timestamp)
            .or_insert(timestamp);
    }

    pub fn save(&self) {
        dbg!(self);
        storage::save_to_path(self, storage::data_path().join("tracker.json")).unwrap();
    }

    pub fn clean_up(&mut self) {
        dbg!(&self);
        let today = get_current_date();
        self.history.retain(|k, _| k == &today);
        self.save();
    }
}

pub type TimeTrackerMap = Arc<Mutex<TrackHistory>>;
