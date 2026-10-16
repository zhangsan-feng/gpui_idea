use super::{EditorLanguage, LANGUAGES};
use anyhow::{Context as _, Result, ensure};
use gpui_kit::{
    App,
    component::{
        Theme,
        highlighter::{HighlightTheme, LanguageRegistry},
    },
};
use std::{path::Path, sync::Arc};

pub(super) fn language_for_path(path: &Path) -> &'static str {
    language_definition(path).map_or("plaintext", |language| language.name)
}

pub(super) fn language_definition(path: &Path) -> Option<&'static EditorLanguage> {
    if path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("CMakeLists.txt"))
    {
        return LANGUAGES.iter().find(|language| language.name == "cmake");
    }
    let extension = path.extension()?.to_str()?;
    LANGUAGES.iter().find(|language| {
        language
            .extensions
            .iter()
            .any(|known| extension.eq_ignore_ascii_case(known))
    })
}

pub(super) fn install(cx: &mut App) -> Result<()> {
    // Palette reference: JetBrains/intellij-community, expUI_lightScheme.xml.
    let theme_bytes =
        crate::infrastructure::assets::load("editor/idea-light.json").unwrap_or_else(|| {
            std::borrow::Cow::Borrowed(include_bytes!("../../../assets/editor/idea-light.json"))
        });
    let theme: HighlightTheme =
        serde_json::from_slice(&theme_bytes).context("IDEA 浅色代码配色格式无效")?;

    let registry = LanguageRegistry::singleton();
    for language in LANGUAGES {
        let config = registry
            .language(language.name)
            .with_context(|| format!("{} 语法解析器未启用，请检查对应语言特性", language.label))?;
        ensure!(config.has_grammar(), "{} 缺少语法解析器", language.label);
    }

    let mut c = registry.language("c").context("C 语法解析器未启用")?;
    c.highlights = c
        .highlights
        .replace("@delimiter", "@punctuation.delimiter")
        .replace("(primitive_type) @type", "(primitive_type) @keyword")
        .replace("(char_literal) @number", "(char_literal) @string")
        .into();
    registry.register("c", &c);

    // The bundled C++ query contains only additions to C, so retain the C rules.
    let mut cpp = registry.language("cpp").context("C++ 语法解析器未启用")?;
    cpp.highlights = format!(
        "{}\n{}\n(this) @keyword\n[(true) (false) (null)] @keyword\n",
        c.highlights, cpp.highlights
    )
    .into();
    registry.register("cpp", &cpp);

    // gpui-component provides the CMake parser with an empty highlight query.
    let mut cmake = registry
        .language("cmake")
        .context("CMake 语法解析器未启用")?;
    cmake.highlights = include_str!("../../../assets/editor/cmake-highlights.scm").into();
    registry.register("cmake", &cmake);

    let mut rust = registry.language("rust").context("Rust 语法解析器未启用")?;
    // Adapt the bundled Rust query to the theme's distinct semantic categories.
    // The default query groups numeric and boolean literals under `constant`.
    rust.highlights = rust
        .highlights
        .replace(
            "(boolean_literal) @constant.builtin",
            "(boolean_literal) @boolean",
        )
        .replace(
            "(integer_literal) @constant.builtin",
            "(integer_literal) @number",
        )
        .replace(
            "(float_literal) @constant.builtin",
            "(float_literal) @number",
        )
        .replace(
            "(escape_sequence) @escape",
            "(escape_sequence) @string.escape",
        )
        .replace("(self) @variable.builtin", "(self) @keyword")
        .replace("@function.macro", "@preproc")
        .into();
    registry.register("rust", &rust);

    let mut python = registry
        .language("python")
        .context("Python 语法解析器未启用")?;
    python.highlights = python
        .highlights
        .replace("@constant.builtin", "@keyword")
        .replace("@escape", "@string.escape")
        .replace("(decorator) @function", "(decorator) @attribute")
        .replace(
            "(decorator\n  (identifier) @function)",
            "(decorator\n  (identifier) @attribute)",
        )
        .into();
    registry.register("python", &python);

    let mut javascript = registry
        .language("javascript")
        .context("JavaScript 语法解析器未启用")?;
    let javascript_query = adapt_javascript(&javascript.highlights);
    let jsx_query = include_str!("../../../assets/editor/jsx-highlights.scm");
    javascript.highlights = format!("{javascript_query}\n{jsx_query}").into();
    registry.register("javascript", &javascript);

    let mut typescript = registry
        .language("typescript")
        .context("TypeScript 语法解析器未启用")?;
    typescript.highlights = adapt_javascript(&typescript.highlights).into();
    registry.register("typescript", &typescript);

    // The bundled TSX query only covers TypeScript additions. Include the full
    // TypeScript/JavaScript query so TSX expressions and JSX tags are highlighted.
    let mut tsx = registry.language("tsx").context("TSX 语法解析器未启用")?;
    tsx.highlights = format!("{}\n{jsx_query}", typescript.highlights).into();
    tsx.injections = typescript.injections.clone();
    tsx.injection_languages = typescript.injection_languages.clone();
    registry.register("tsx", &tsx);

    let mut java = registry.language("java").context("Java 语法解析器未启用")?;
    java.highlights = java
        .highlights
        .replace("@constant.builtin", "@keyword")
        .replace("@type.builtin", "@keyword")
        .replace("(this) @variable.builtin", "(this) @keyword")
        .replace("(super) @function.builtin", "(super) @keyword")
        .into();
    registry.register("java", &java);

    let mut go = registry.language("go").context("Go 语法解析器未启用")?;
    go.highlights = go
        .highlights
        .replace("@variable.member", "@property")
        .replace("@constant.builtin", "@keyword")
        .replace("@namespace", "@type")
        .into();
    registry.register("go", &go);

    let mut markdown = registry
        .language("markdown")
        .context("Markdown 语法解析器未启用")?;
    markdown.highlights = markdown
        .highlights
        .replace("@text.uri", "@link_uri")
        .replace("@text.reference", "@link_text")
        .into();
    registry.register("markdown", &markdown);

    let mut html = registry.language("html").context("HTML 语法解析器未启用")?;
    html.highlights = html
        .highlights
        .replace("(doctype) @constant", "(doctype) @tag.doctype")
        .into();
    registry.register("html", &html);

    Theme::update(cx, |current| current.highlight_theme = Arc::new(theme));
    let languages: Vec<_> = LANGUAGES.iter().map(|language| language.name).collect();
    tracing::info!(
        ?languages,
        theme = "IDEA Light",
        "code highlighting initialized"
    );
    Ok(())
}

fn adapt_javascript(query: &str) -> String {
    let query = query
        .replace("@constant.builtin", "@keyword")
        .replace("@type.builtin", "@keyword")
        .replace("(this) @variable.builtin", "(this) @keyword")
        .replace("(super) @variable.builtin", "(super) @keyword");
    format!("{query}\n(escape_sequence) @string.escape\n")
}
