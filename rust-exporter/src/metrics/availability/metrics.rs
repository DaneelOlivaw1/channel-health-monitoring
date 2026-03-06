use metrics::gauge;
use std::sync::Arc;

pub struct AvailabilityMetrics {
    _private: (),
}

impl AvailabilityMetrics {
    pub fn new() -> Arc<Self> {
        Arc::new(Self { _private: () })
    }

    pub fn set_availability(&self, channel_group: &str, value: f64) {
        gauge!("channel_availability_percent", "channel_group" => channel_group.to_string())
            .set(value);
    }
}

impl Default for AvailabilityMetrics {
    fn default() -> Self {
        Self::new().as_ref().clone()
    }
}

impl Clone for AvailabilityMetrics {
    fn clone(&self) -> Self {
        Self { _private: () }
    }
}
