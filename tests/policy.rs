use sk_iced::motion::{Policy, QueryFallback, system_enabled};
#[test]
fn explicit_policy_controls_motion() {
    assert!(!Policy::DISABLED.enabled());
    assert!(
        Policy {
            system_enabled: true
        }
        .enabled()
    );
}
#[test]
fn native_queries_and_refresh_agree() {
    let a = system_enabled(QueryFallback::Disabled);
    let b = system_enabled(QueryFallback::Enabled);
    assert!(!a || b);
    let mut policy = Policy::DISABLED;
    policy.refresh();
    assert_eq!(policy, Policy::detect());
}
