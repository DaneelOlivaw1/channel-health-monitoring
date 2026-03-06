use metrics::gauge;

pub struct CostMetrics;

impl CostMetrics {
    pub fn new() -> Self {
        Self
    }

    pub fn set_opus_cost(&self, channel_group: &str, value: f64) {
        gauge!("channel_avg_cost_cny_opus", "channel_group" => channel_group.to_string())
            .set(value);
    }

    pub fn set_sonnet_cost(&self, channel_group: &str, value: f64) {
        gauge!("channel_avg_cost_cny_sonnet", "channel_group" => channel_group.to_string())
            .set(value);
    }

    pub fn set_all_cost(&self, channel_group: &str, value: f64) {
        gauge!("channel_avg_cost_cny_all", "channel_group" => channel_group.to_string()).set(value);
    }
}
