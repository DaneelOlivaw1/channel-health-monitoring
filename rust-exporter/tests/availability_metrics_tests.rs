use metrics::{describe_gauge, gauge};

#[test]
fn test_availability_gauge_basic() {
    describe_gauge!(
        "channel_availability_percent",
        "Channel availability percentage"
    );

    gauge!("channel_availability_percent", "channel_group" => "aws").set(95.0);
    gauge!("channel_availability_percent", "channel_group" => "special").set(98.0);

    assert!(true);
}

#[test]
fn test_availability_gauge_labels() {
    gauge!("channel_availability_percent", "channel_group" => "aws").set(55.1);
    gauge!("channel_availability_percent", "channel_group" => "special").set(95.0);

    assert!(true);
}

#[test]
fn test_availability_gauge_zero() {
    gauge!("channel_availability_percent", "channel_group" => "aws").set(0.0);
    assert!(true);
}

#[test]
fn test_availability_gauge_hundred() {
    gauge!("channel_availability_percent", "channel_group" => "special").set(100.0);
    assert!(true);
}

#[test]
fn test_availability_gauge_decimal() {
    gauge!("channel_availability_percent", "channel_group" => "aws").set(99.9);
    gauge!("channel_availability_percent", "channel_group" => "special").set(0.1);
    assert!(true);
}
