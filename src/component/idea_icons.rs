use std::borrow::Cow;

// Embed in debug builds too: loading an icon on the GUI thread never performs IO.
#[derive(rust_embed::RustEmbed)]
#[folder = "assets/idea/"]
#[include = "*.svg"]
struct IdeaIcons;

pub(super) fn load(path: &str) -> Option<Cow<'static, [u8]>> {
    IdeaIcons::get(path.strip_prefix("idea/")?).map(|asset| asset.data)
}

pub(super) fn list(prefix: &str) -> impl Iterator<Item = String> {
    IdeaIcons::iter()
        .map(|name| format!("idea/{name}"))
        .filter(move |path| path.starts_with(prefix))
}
