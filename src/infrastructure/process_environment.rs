/// `cargo run` injects package/build metadata into the editor's environment.
/// Passing it to nested Cargo builds invalidates ring's build-script fingerprint.
/// Keep user configuration such as CARGO_HOME, CARGO_TARGET_DIR and RUSTFLAGS.
pub(crate) fn cargo_runtime_keys() -> Vec<String> {
    let mut keys: Vec<_> = std::env::vars_os()
        .filter_map(|(key, _)| key.into_string().ok())
        .filter(|key| {
            let upper = key.to_ascii_uppercase();
            key.bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
                && (upper.starts_with("CARGO_PKG_")
                    || upper.starts_with("CARGO_MANIFEST_")
                    || upper.starts_with("CARGO_CFG_")
                    || upper.starts_with("CARGO_FEATURE_")
                    || upper.starts_with("DEP_")
                    || matches!(
                        upper.as_str(),
                        "OUT_DIR" | "CARGO_BIN_NAME" | "CARGO_CRATE_NAME" | "CARGO_PRIMARY_PACKAGE"
                    ))
        })
        .collect();
    keys.sort();
    keys
}

pub(crate) fn sanitize(command: &mut std::process::Command) {
    let keys = cargo_runtime_keys();
    for key in &keys {
        command.env_remove(key);
    }
    if !keys.is_empty() {
        tracing::debug!(
            variables = keys.len(),
            "removed inherited Cargo package metadata from child environment"
        );
    }
}
