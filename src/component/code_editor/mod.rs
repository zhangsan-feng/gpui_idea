mod completion;
mod core;
mod definition_cursor;
mod external;

pub(crate) use external::{
    attach_completions, completion_dropdown, definition_pointer, dismiss_completions, init,
    language_for_path, language_label,
};

struct EditorLanguage {
    name: &'static str,
    label: &'static str,
    extensions: &'static [&'static str],
}

// Keep this list aligned with the individual gpui-kit language features in Cargo.toml.
// JSON is included by gpui-component's base tree-sitter feature.
const LANGUAGES: &[EditorLanguage] = &[
    EditorLanguage {
        name: "c",
        label: "C",
        extensions: &["c", "h"],
    },
    EditorLanguage {
        name: "cpp",
        label: "C++",
        extensions: &[
            "cpp", "cc", "cxx", "c++", "hpp", "hh", "hxx", "ipp", "tpp", "ixx",
        ],
    },
    EditorLanguage {
        name: "cmake",
        label: "CMake",
        extensions: &["cmake"],
    },
    EditorLanguage {
        name: "rust",
        label: "Rust",
        extensions: &["rs"],
    },
    EditorLanguage {
        name: "python",
        label: "Python",
        extensions: &["py", "pyw", "pyi"],
    },
    EditorLanguage {
        name: "php",
        label: "PHP",
        extensions: &[
            "php", "phtml", "php3", "php4", "php5", "php7", "php8", "phps",
        ],
    },
    EditorLanguage {
        name: "go",
        label: "Go",
        extensions: &["go"],
    },
    EditorLanguage {
        name: "java",
        label: "Java",
        extensions: &["java"],
    },
    EditorLanguage {
        name: "javascript",
        label: "JavaScript",
        extensions: &["js", "mjs", "cjs", "jsx"],
    },
    EditorLanguage {
        name: "typescript",
        label: "TypeScript",
        extensions: &["ts", "mts", "cts"],
    },
    EditorLanguage {
        name: "tsx",
        label: "TSX",
        extensions: &["tsx"],
    },
    EditorLanguage {
        name: "toml",
        label: "TOML",
        extensions: &["toml"],
    },
    EditorLanguage {
        name: "yaml",
        label: "YAML",
        extensions: &["yaml", "yml"],
    },
    EditorLanguage {
        name: "markdown",
        label: "Markdown",
        extensions: &["md", "markdown", "mdown", "mkd"],
    },
    EditorLanguage {
        name: "html",
        label: "HTML",
        extensions: &["html", "htm"],
    },
    EditorLanguage {
        name: "css",
        label: "CSS",
        extensions: &["css"],
    },
    EditorLanguage {
        name: "json",
        label: "JSON",
        extensions: &["json"],
    },
];
