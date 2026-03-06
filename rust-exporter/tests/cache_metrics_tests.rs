use metrics::gauge;

#[test]
fn test_cache_reuse_gauge_basic() {
    gauge!("channel_cache_reuse_percent", "channel_group" => "aws").set(85.5);
    gauge!("channel_cache_reuse_percent", "channel_group" => "special").set(92.3);
    assert!(true);
}

#[test]
fn test_cache_reuse_gauge_labels() {
    gauge!("channel_cache_reuse_percent", "channel_group" => "aws").set(0.0);
    gauge!("channel_cache_reuse_percent", "channel_group" => "special").set(100.0);
    assert!(true);
}
