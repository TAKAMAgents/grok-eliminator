use grok_eliminator::strip_credential_exports;

#[test]
fn report_boundary_never_needs_the_secret_value() {
    let (sanitized, changed) = strip_credential_exports("XAI_API_KEY=do-not-print\n");
    assert!(changed);
    assert!(sanitized.is_empty());
    assert!(!sanitized.contains("do-not-print"));
}
