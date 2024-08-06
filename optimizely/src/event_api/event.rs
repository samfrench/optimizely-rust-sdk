use std::collections::HashMap;

/// Representation of the events which can be *dispatched* to Optimizely Event API.
///
/// An event can either be a decision or conversion.
///
/// ```
/// use optimizely::event_api::Event;
/// use std::collections::HashMap;
///
/// // Create some example IDs
/// let account_id = "21537940595";
/// let user_id = "user0";
/// let campaign_id = "9300000133039";
/// let experiment_id = "9300000169122";
/// let variation_id = "87757";
/// let event_id = "22305150298";
/// let event_key = "purchase";
///
/// let properties = HashMap::default();
/// let tags = HashMap::default();
///
/// // Create two events from above IDs
/// let decision = Event::decision(
///     account_id,
///     user_id,
///     campaign_id,
///     experiment_id,
///     variation_id
/// );
/// let conversion = Event::conversion(
///     account_id,
///     user_id,
///     event_id,
///     event_key,
///     properties,
///     tags,
/// );
///
/// // Assertions
/// assert_eq!(decision.account_id(), account_id);
/// assert_eq!(conversion.account_id(), account_id);
/// ```
#[allow(dead_code)]
#[derive(Debug)]
pub enum Event {
    /// An event that indicates a user being bucketed into an experiment
    Decision {
        #[doc(hidden)]
        account_id: String,
        #[doc(hidden)]
        user_id: String,
        #[doc(hidden)]
        campaign_id: String,
        #[doc(hidden)]
        experiment_id: String,
        #[doc(hidden)]
        variation_id: String,
    },

    /// An event that indicates a user interacting with the application
    Conversion {
        #[doc(hidden)]
        account_id: String,
        #[doc(hidden)]
        user_id: String,
        #[doc(hidden)]
        event_id: String,
        #[doc(hidden)]
        event_key: String,
        #[doc(hidden)]
        properties: HashMap<String, String>,
        #[doc(hidden)]
        tags: HashMap<String, String>,
    },
}

impl Event {
    /// Constructor for a new decision event
    pub fn decision<T: Into<String>>(
        account_id: T, user_id: T, campaign_id: T, experiment_id: T, variation_id: T,
    ) -> Event {
        Event::Decision {
            account_id: account_id.into(),
            user_id: user_id.into(),
            campaign_id: campaign_id.into(),
            experiment_id: experiment_id.into(),
            variation_id: variation_id.into(),
        }
    }

    /// Constructor for a new decision event
    pub fn conversion<T: Into<String>>(
        account_id: T, user_id: T, event_id: T, event_key: T, properties: HashMap<String, String>,
        tags: HashMap<String, String>,
    ) -> Event {
        Event::Conversion {
            account_id: account_id.into(),
            user_id: user_id.into(),
            event_id: event_id.into(),
            event_key: event_key.into(),
            properties: properties,
            tags: tags,
        }
    }

    /// Getter for the account_id field that exists for both `Event::Decision` and `Event::Conversion`
    pub fn account_id(&self) -> &str {
        match self {
            Event::Decision { account_id, .. } => account_id,
            Event::Conversion { account_id, .. } => account_id,
        }
    }

    /// Getter for the user_id field that exists for both `Event::Decision` and `Event::Conversion`
    pub fn user_id(&self) -> &str {
        match self {
            Event::Decision { user_id, .. } => user_id,
            Event::Conversion { user_id, .. } => user_id,
        }
    }
}
