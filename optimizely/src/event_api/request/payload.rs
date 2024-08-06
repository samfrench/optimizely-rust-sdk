// External imports
use serde::Serialize;
use std::collections::HashMap;

// Imports from super
use super::Visitor;
use crate::event_api::Event;

// Information regarding the SDK client
const CLIENT_NAME: &str = "rust-sdk";
const CLIENT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Event key for activating an experiment
const ACTIVATE_EVENT_KEY: &str = "campaign_activated";

#[derive(Serialize)]
/// HTTP request payload to send to Event API
pub struct Payload<'a> {
    account_id: String,
    visitors: Vec<Visitor>,
    enrich_decisions: bool,
    anonymize_ip: bool,
    client_name: &'a str,
    client_version: &'a str,
}

impl Payload<'_> {
    /// Construct an empty payload for a given account
    pub fn new<T: Into<String>>(account_id: T) -> Payload<'static> {
        Payload {
            account_id: account_id.into(),
            visitors: Vec::<Visitor>::new(),
            enrich_decisions: true,
            anonymize_ip: true,
            client_name: CLIENT_NAME,
            client_version: CLIENT_VERSION,
        }
    }

    /// Add a decision/conversion event to the payload
    pub fn add_event(&mut self, event: Event) {
        if event.account_id() != self.account_id {
            // TODO: return a Result instead
            panic!("Trying to add event from other account");
        }

        // TODO: look up visitor ID in existing list

        // Retrieve existing visitor or insert new one
        let mut visitor = Visitor::new(event.user_id());

        match event {
            Event::Decision {
                campaign_id,
                experiment_id,
                variation_id,
                ..
            } => {
                log::debug!("Adding decision event to log payload");

                // Use a copy of campaign_id as entity_id
                let entity_id = campaign_id.clone();

                // Add decision to visitor
                visitor.add_decision(campaign_id, experiment_id, variation_id);

                // Campaign activated event does not have tags or properties
                let properties = HashMap::default();
                let tags = HashMap::default();

                // Add campaign_activated event
                visitor.add_event(entity_id, String::from(ACTIVATE_EVENT_KEY), properties, tags);
            }
            Event::Conversion {
                event_id,
                event_key,
                properties,
                tags,
                ..
            } => {
                log::debug!("Adding conversion event to log payload");

                // Add custom event
                visitor.add_event(event_id, event_key, properties, tags);
            }
        }

        // Add to the list
        self.visitors.push(visitor);
    }
}
