use serde_json::{Value, json};

pub(super) fn configuration(section: &str) -> Value {
    let python = json!({"analysis": {
        "autoImportCompletions": false,
        "diagnosticMode": "openFilesOnly",
        "useLibraryCodeForTypes": true
    }});
    match section {
        "" => settings(),
        "rust-analyzer" => {
            json!({"cargo": {"sysroot": "discover"}, "checkOnSave": false, "completion": {"autoimport": {"enable": false}}})
        }
        "python" => python,
        "python.analysis" => python["analysis"].clone(),
        "gopls" => json!({"completeUnimported": false, "usePlaceholders": false}),
        "javascript" | "typescript" => {
            json!({"suggest": {"autoImports": false, "completeFunctionCalls": false}})
        }
        "html" => json!({"suggest": {"html5": true}}),
        "css" => json!({"validate": true}),
        "json" => json!({"validate": {"enable": true}}),
        "yaml" => {
            json!({"completion": true, "validate": true, "hover": true, "schemaStore": {"enable": false}})
        }
        "java" => {
            json!({"autobuild": {"enabled": false}, "completion": {"guessMethodArguments": false}})
        }
        _ => Value::Null,
    }
}

pub(super) fn initialization_options(family: &str) -> Value {
    match family {
        "rust" => configuration("rust-analyzer"),
        "go" => configuration("gopls"),
        "javascript" => json!({"preferences": {
            "includeCompletionsForModuleExports": false,
            "includeCompletionsWithInsertText": true,
            "includeCompletionsWithSnippetText": false
        }}),
        "html" | "css" | "json" => json!({"provideFormatter": false}),
        "java" => json!({"settings": {"java": configuration("java")}}),
        _ => json!({}),
    }
}

pub(super) fn settings() -> Value {
    json!({
        "python": configuration("python"),
        "gopls": configuration("gopls"),
        "javascript": configuration("javascript"),
        "typescript": configuration("typescript"),
        "html": configuration("html"),
        "css": configuration("css"),
        "json": configuration("json"),
        "yaml": configuration("yaml"),
        "java": configuration("java")
    })
}
