#[test]
fn core_version_is_nonempty() {
    assert!(!belljar_core::version().is_empty());
}
