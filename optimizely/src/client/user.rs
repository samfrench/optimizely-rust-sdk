// External imports
use murmur3::murmur3_32 as murmur3_hash;
use std::collections::HashMap;

// Imports from crate
use crate::conversion::Conversion;
use crate::datafile::{Experiment, FeatureFlag, Variation};
use crate::decision::{DecideOptions, Decision};

// Imports from super
use super::Client;

/// Custom type alias for user attributes
pub type UserAttributes = HashMap<String, String>;

/// Constant used for the hashing algorithm
const HASH_SEED: u32 = 1;

/// Ranges are specified between 0 and 10_000
const MAX_OF_RANGE: f64 = 10_000_f64;

/// User specific context
///
/// ```
/// use optimizely::{Client, decision::DecideOptions};
///
/// // Initialize Optimizely client using local datafile
/// let file_path = "../datafiles/sandbox.json";
/// let optimizely_client = Client::from_local_datafile(file_path)?
///     .initialize();
///
/// // Do not send any decision events
/// let decide_options = DecideOptions {
///     disable_decision_event: true,
///     ..DecideOptions::default()
/// };
///
/// // Create a user context
/// let attributes = optimizely::user_attributes! {
///     "is_employee" => "true",
///     "app_version" => "1.3.2",
/// };
/// let user_context = optimizely_client.create_user_context("123abc789xyz");
///
/// // Decide a feature flag for this user
/// let decision = user_context.decide_with_options("buy_button", &decide_options);
///
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct UserContext<'a> {
    client: &'a Client,
    user_id: &'a str,
    attributes: UserAttributes,
}

impl UserContext<'_> {
    // Only allow UserContext to be constructed from a Client
    pub(crate) fn new<'a>(client: &'a Client, user_id: &'a str, attributes: UserAttributes) -> UserContext<'a> {
        UserContext {
            client,
            user_id,
            attributes,
        }
    }

    /// Add a new attribute to a user context
    pub fn set_attribute<T: Into<String>>(&mut self, key: T, value: T) {
        // Create owned copies of the key and value
        let key = key.into();
        let value = value.into();

        // Add the attribute
        self.attributes.insert(key, value);
    }

    /// Get the client instance
    pub fn client(&self) -> &Client {
        self.client
    }

    /// Get the id of a user
    pub fn user_id(&self) -> &str {
        self.user_id
    }

    /// Get all attributes of a user
    pub fn attributes(&self) -> &UserAttributes {
        // Return borrowed reference to attributes
        &self.attributes
    }

    #[cfg(feature = "online")]
    /// Track a conversion event (without properties and tags) for this user
    pub fn track_event(&self, event_key: &str) {
        let properties = HashMap::default();
        let tags = HashMap::default();
        self.track_event_with_properties_and_tags(event_key, properties, tags)
    }

    #[cfg(feature = "online")]
    /// Track a conversion event with properties (but without tags) for this user
    pub fn track_event_with_properties(&self, event_key: &str, properties: HashMap<String, String>) {
        let tags = HashMap::default();
        self.track_event_with_properties_and_tags(event_key, properties, tags)
    }

    #[cfg(feature = "online")]
    /// Track a conversion event with properties and tags for this user
    pub fn track_event_with_properties_and_tags(
        &self, event_key: &str, properties: HashMap<String, String>, tags: HashMap<String, String>,
    ) {
        // Find the event key in the datafile
        match self.client.datafile().event(event_key) {
            Some(event) => {
                log::debug!("Logging conversion event");

                // Create conversion to send to dispatcher
                let conversion = Conversion::new(event_key, event.id(), properties, tags);

                // Ignore result of the send_decision function
                self.client
                    .event_dispatcher()
                    .send_conversion_event(self, conversion);
            }
            None => {
                log::warn!("Event key does not exist in datafile");
            }
        }
    }

    /// Decide which variation to show to a user
    pub fn decide(&self, flag_key: &str) -> Decision {
        let options = DecideOptions::default();
        self.decide_with_options(flag_key, &options)
    }

    /// Decide which variation to show to a user
    pub fn decide_with_options(&self, flag_key: &str, options: &DecideOptions) -> Decision {
        // Retrieve Flag object
        let flag = match self.client.datafile().flag(flag_key) {
            Some(flag) => flag,
            None => {
                // When flag key cannot be found, return the off variation
                // CONSIDERATION: Could have used Result<Decision, E> but this is how other Optimizely SDKs work
                // Current behaviour ignoring legacy experiments
                return Decision::off(flag_key);
                // // Allow legacy experiments to default to first variant
                // let experiment = self.client.datafile().experiment_by_name(flag_key).unwrap(); // TODO: remove unwrap

                // &FeatureFlag {
                //     key: flag_key.to_string(),
                //     rollout_id: "".to_string(),
                //     experiment_ids: vec![experiment.id().to_string()],
                //     // Add other fields as necessary
                // }
            }
        };

        // Only send decision events if the disable_decision_event option is false
        let mut send_decision = !options.disable_decision_event;

        // Get the selected variation for the given flag
        let decision = match self.decide_variation_for_flag(flag, &mut send_decision) {
            Some((experiment, variation)) => {
                // Unpack the variation and create Decision struct
                Decision::new(
                    flag_key,
                    experiment.campaign_id(),
                    experiment.id(),
                    variation.id(),
                    variation.key(),
                    variation.is_feature_enabled(),
                )
            }
            None => {
                // No experiment or rollout found, or user does not qualify for any
                Decision::off(flag_key)
            }
        };

        #[cfg(feature = "online")]
        if send_decision {
            self.client
                .event_dispatcher()
                .send_decision_event(&self, decision.clone());
        }

        // Return
        decision
    }

    fn decide_variation_for_flag(
        &self, flag: &FeatureFlag, send_decision: &mut bool,
    ) -> Option<(&Experiment, &Variation)> {
        // Find first Experiment for which this user qualifies
        let result = flag.experiments_ids().iter().find_map(|experiment_id| {
            let experiment = self.client.datafile().experiment(experiment_id);

            match experiment {
                Some(experiment) => self.decide_variation_for_experiment(experiment),
                None => None,
            }
        });

        match result {
            Some(_) => {
                // Send out a decision event for an A/B Test
                *send_decision &= true;

                result
            }
            None => {
                // Do not send any decision for a Rollout (Targeted Delivery)
                *send_decision = false;

                // No direct experiment found, let's look at the Rollout
                let rollout = self.client.datafile().rollout(flag.rollout_id()).unwrap(); // TODO: remove unwrap

                // Find the first experiment within the Rollout for which this user qualifies
                rollout
                    .experiments()
                    .iter()
                    .find_map(|experiment| self.decide_variation_for_experiment(experiment))
            }
        }
    }

    fn decide_variation_for_experiment<'a>(
        &'a self, experiment: &'a Experiment,
    ) -> Option<(&'a Experiment, &'a Variation)> {
        // Use references for the ids
        let user_id = self.user_id();
        let experiment_id = experiment.id();

        // Concatenate user id and experiment id
        let bucketing_key = format!("{user_id}{experiment_id}");

        // To hash the bucket key it needs to be converted to an array of `u8` bytes
        // Use Murmur3 (32-bit) with seed
        let mut bytes = bucketing_key.as_bytes();
        let hash_value = murmur3_hash(&mut bytes, HASH_SEED).unwrap();

        // Bring the hash into a range of 0 to 10_000
        let bucket_value = ((hash_value as f64) / (u32::MAX as f64) * MAX_OF_RANGE) as u64;

        // Get the variation ID according to the traffic allocation
        experiment
            .traffic_allocation()
            .variation(bucket_value)
            // Map it to a Variation struct
            .map(|variation_id| experiment.variation(variation_id))
            .flatten()
            // Combine it with the experiment
            .map(|variation| Some((experiment, variation)))
            .flatten()
    }
}

/// Macro to create UserAttributes
#[macro_export]
macro_rules! user_attributes {
    { $( $key: expr => $value: expr),* $(,)?} => {
        {
            let mut attribute = optimizely::client::UserAttributes::new();

            $(
                attribute.insert($key.into(), $value.into());
            )*

            attribute
        }
    };
}
