use search_agent_lib::server::lifecycle::should_shutdown_on_exit;

#[test]
fn app_close_respects_ownership() {
    assert!(should_shutdown_on_exit("self"));
    assert!(!should_shutdown_on_exit("external"));
    assert!(!should_shutdown_on_exit("none"));
}
