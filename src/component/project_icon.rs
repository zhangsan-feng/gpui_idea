use gpui_kit::{Img, Styled, img, px};
use serde::Deserialize;
use std::{collections::HashMap, path::Path, sync::LazyLock};

#[derive(Deserialize)]
#[serde(default)]
struct IconRules {
    file_names: HashMap<String, String>,
    extensions: HashMap<String, String>,
    directory_names: HashMap<String, String>,
    source_roots: Vec<String>,
    source_directory_icon: String,
    fallback_file: String,
    fallback_directory: String,
}

impl Default for IconRules {
    fn default() -> Self {
        Self {
            file_names: HashMap::new(),
            extensions: HashMap::new(),
            directory_names: HashMap::new(),
            source_roots: Vec::new(),
            source_directory_icon: "idea/folder.svg".into(),
            fallback_file: "idea/unknown.svg".into(),
            fallback_directory: "idea/folder.svg".into(),
        }
    }
}

static RULES: LazyLock<IconRules> = LazyLock::new(|| {
    let mut rules: IconRules = serde_json::from_str(include_str!(
        "../../assets/idea/file-icons.json"
    ))
    .unwrap_or_else(|error| {
        tracing::error!(%error, "file icon rules could not be parsed; using generic icons");
        IconRules::default()
    });
    // Rules are case-insensitive, including names supplied by future configuration edits.
    for table in [
        &mut rules.file_names,
        &mut rules.extensions,
        &mut rules.directory_names,
    ] {
        *table = std::mem::take(table)
            .into_iter()
            .map(|(key, value)| (key.to_ascii_lowercase(), value))
            .collect();
    }
    rules
});

pub(crate) fn project_icon(path: &Path, directory: bool) -> Img {
    img(icon_path(path, directory))
        .w(px(16.))
        .h(px(16.))
        .flex_shrink_0()
}

fn icon_path(path: &Path, directory: bool) -> String {
    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_ascii_lowercase();
    let fallback = if directory {
        &RULES.fallback_directory
    } else {
        &RULES.fallback_file
    };
    let asset = if directory {
        RULES.directory_names.get(&name).or_else(|| {
            RULES
                .source_roots
                .iter()
                .any(|root| path.starts_with(Path::new(root)))
                .then_some(&RULES.source_directory_icon)
        })
    } else {
        RULES.file_names.get(&name).or_else(|| {
            // Check longest compound suffix first (e.g. d.ts, tar.gz), then its final extension.
            name.match_indices('.')
                .filter_map(|(index, _)| RULES.extensions.get(&name[index + 1..]))
                .next()
        })
    }
    .unwrap_or(fallback);
    if super::idea_icons::load(asset).is_some() {
        return asset.clone();
    }
    tracing::warn!(asset, "file icon asset is missing; using generic icon");
    if super::idea_icons::load(fallback).is_some() {
        fallback.clone()
    } else {
        let defaults = IconRules::default();
        if directory {
            defaults.fallback_directory
        } else {
            defaults.fallback_file
        }
    }
}
