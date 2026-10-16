use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProjectLanguage {
    Rust,
    Go,
    Python,
    JavaScript,
    TypeScript,
}

impl ProjectLanguage {
    pub(crate) const ALL: [Self; 5] = [
        Self::Rust,
        Self::Go,
        Self::Python,
        Self::JavaScript,
        Self::TypeScript,
    ];
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Rust => "Rust",
            Self::Go => "Go",
            Self::Python => "Python",
            Self::JavaScript => "JavaScript",
            Self::TypeScript => "TypeScript",
        }
    }
    pub(crate) fn command(self) -> &'static str {
        match self {
            Self::Rust => "cargo",
            Self::Go => "go",
            Self::Python => "python",
            Self::JavaScript | Self::TypeScript => "node",
        }
    }
    pub(crate) fn template(self, library: bool) -> &'static str {
        match self {
            Self::Rust if library => "Library",
            Self::Rust => "Binary (application)",
            Self::Go => "Go Module (application)",
            Self::Python => "Python Script",
            Self::JavaScript => "Node.js (ES Modules)",
            Self::TypeScript => "Node.js (TypeScript)",
        }
    }
}

pub(crate) struct CreateProject {
    pub location: PathBuf,
    pub language: ProjectLanguage,
    pub library: bool,
    pub toolchain: String,
    pub module: String,
    pub environment: String,
}

#[derive(Clone)]
pub(crate) struct ToolchainInfo {
    pub executable: PathBuf,
    pub version: String,
    pub standard_library: Option<PathBuf>,
}
