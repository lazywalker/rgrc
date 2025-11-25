use std::collections::HashSet;

#[test]
fn test_generated_embedded_configs_consistency() {
    // Ensure compile-time feature enabled
    #[cfg(not(feature = "embed-configs"))]
    {
        // This test only runs when the feature is enabled.
        return;
    }

    // Access constants from the library
    use rgrc::{EMBEDDED_CONFIG_NAMES, EMBEDDED_CONFIGS};

    // Basic assertions
    assert!(
        !EMBEDDED_CONFIG_NAMES.is_empty(),
        "EMBEDDED_CONFIG_NAMES should not be empty"
    );
    assert_eq!(
        EMBEDDED_CONFIGS.len(),
        EMBEDDED_CONFIG_NAMES.len(),
        "Names list and configs length should match"
    );

    let names: HashSet<&str> = EMBEDDED_CONFIG_NAMES.iter().copied().collect();

    for (name, content) in EMBEDDED_CONFIGS.iter() {
        assert!(
            names.contains(name),
            "Config name {} not in names list",
            name
        );
        assert!(
            !content.is_empty(),
            "Embedded config {} should not be empty",
            name
        );
    }

    // Spot-check a commonly expected config
    assert!(
        EMBEDDED_CONFIG_NAMES.iter().any(|&s| s == "conf.ping"),
        "Expected conf.ping to be embedded"
    );
}
