// External imports
use serde::Serialize;
use std::collections::HashMap;

// Imports from super
use super::Snapshot;

#[derive(Serialize)]
pub struct Visitor {
    visitor_id: String,
    // TODO: add field `attributes`
    snapshots: [Snapshot; 1],
}

impl Visitor {
    pub fn new<T: Into<String>>(visitor_id: T) -> Visitor {
        Visitor {
            visitor_id: visitor_id.into(),
            snapshots: [Snapshot::new()],
        }
    }

    pub fn add_decision(&mut self, campaign_id: String, experiment_id: String, variation_id: String) {
        self.snapshots[0].add_decision(campaign_id, experiment_id, variation_id);
    }

    pub fn add_event(
        &mut self, entity_id: String, event_key: String, properties: HashMap<String, String>,
        tags: HashMap<String, String>,
    ) {
        self.snapshots[0].add_event(entity_id, event_key, properties, tags);
    }
}
