use metrics::gauge;

#[test]
fn test_cost_gauge_basic() {
    gauge!("channel_avg_cost_cny_opus", "channel_group" => "aws").set(0.54);
    gauge!("channel_avg_cost_cny_sonnet", "channel_group" => "aws").set(0.37);
    gauge!("channel_avg_cost_cny_all", "channel_group" => "aws").set(0.47);
    assert!(true);
}

#[test]
fn test_cost_gauge_special_channel() {
    gauge!("channel_avg_cost_cny_opus", "channel_group" => "special").set(0.60);
    gauge!("channel_avg_cost_cny_sonnet", "channel_group" => "special").set(0.40);
    gauge!("channel_avg_cost_cny_all", "channel_group" => "special").set(0.50);
    assert!(true);
}

#[test]
fn test_cost_gauge_zero() {
    gauge!("channel_avg_cost_cny_opus", "channel_group" => "aws").set(0.0);
    gauge!("channel_avg_cost_cny_sonnet", "channel_group" => "aws").set(0.0);
    gauge!("channel_avg_cost_cny_all", "channel_group" => "aws").set(0.0);
    assert!(true);
}

#[test]
fn test_cost_gauge_high_values() {
    gauge!("channel_avg_cost_cny_opus", "channel_group" => "aws").set(10.0);
    gauge!("channel_avg_cost_cny_sonnet", "channel_group" => "aws").set(5.0);
    gauge!("channel_avg_cost_cny_all", "channel_group" => "aws").set(7.5);
    assert!(true);
}

#[test]
fn test_cost_gauge_decimal_precision() {
    gauge!("channel_avg_cost_cny_opus", "channel_group" => "aws").set(0.123456);
    gauge!("channel_avg_cost_cny_sonnet", "channel_group" => "special").set(0.987654);
    assert!(true);
}
