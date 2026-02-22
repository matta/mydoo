use tasklens_core::types::DocMetadata;

#[test]
fn test_doc_metadata_debug_leaks_secret() {
    let secret_url = "automerge:1234567890abcdef";
    let meta = DocMetadata {
        automerge_url: Some(secret_url.to_string()),
    };
    let debug_output = format!("{:?}", meta);

    // Assert that the secret is redacted
    assert!(!debug_output.contains(secret_url));

    // Verify structure
    assert!(debug_output.contains("DocMetadata"));
    assert!(debug_output.contains("automerge_url"));
    assert!(debug_output.contains("..."));

    // Check prefix
    assert!(debug_output.contains("automerge:1234"));

    // Check suffix
    assert!(debug_output.contains("cdef"));

    // Check handling of short secret
    let short_secret = "automerge:short";
    let meta_short = DocMetadata {
        automerge_url: Some(short_secret.to_string()),
    };
    let debug_short = format!("{:?}", meta_short);
    assert!(debug_short.contains("[REDACTED]"));
    assert!(!debug_short.contains("automerge:short"));

    // Check None
    let meta_none = DocMetadata {
        automerge_url: None,
    };
    let debug_none = format!("{:?}", meta_none);
    assert!(debug_none.contains("None"));

    // Check non-ASCII handling (avoid panic)
    // "automerge:a" is 11 bytes.
    // "🚀" is 4 bytes.
    // "automerge:a🚀..."
    // Byte 14 falls inside the first emoji. is_char_boundary(14) should be false.
    let bad_boundary_secret = "automerge:a🚀🚀🚀🚀🚀🚀";
    let meta_bad = DocMetadata {
        automerge_url: Some(bad_boundary_secret.to_string()),
    };
    // This should NOT panic and should fallback to REDACTED
    let debug_bad = format!("{:?}", meta_bad);
    assert!(debug_bad.contains("[REDACTED]"));

    println!("DEBUG OUTPUT: {}", debug_output);
}
