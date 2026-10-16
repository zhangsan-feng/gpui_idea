use super::{core, toolchain};
use crate::domain::project_creation::{CreateProject, ProjectLanguage};
use anyhow::{Context as _, Result, ensure};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

struct Staging {
    directory: PathBuf,
    destination: PathBuf,
}
impl Drop for Staging {
    fn drop(&mut self) {
        let directory = self.directory.clone();
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            runtime.spawn_blocking(move || {
                let _ = fs::remove_dir_all(directory);
            });
        }
    }
}

fn prepare(location: &Path) -> Result<Staging> {
    let parent = core::validate(location.parent().context("项目父目录无效")?)?;
    let destination = parent.join(location.file_name().context("项目名称无效")?);
    ensure!(
        fs::symlink_metadata(&destination)
            .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound),
        "目标目录已存在或无法访问，请选择新的项目位置"
    );
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let directory =
        std::env::temp_dir().join(format!("gpui-project-{}-{stamp}", std::process::id()));
    fs::create_dir(&directory).context("无法创建初始化临时目录")?;
    Ok(Staging {
        directory,
        destination,
    })
}

fn write(root: &Path, path: &str, content: &str) -> Result<()> {
    let path = root.join(path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content).context("写入项目模板失败")
}

fn templates(stage: &Staging, language: ProjectLanguage, name: &str, library: bool) -> Result<()> {
    let root = &stage.directory;
    let normalized: String = name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let normalized = normalized.trim_matches('-');
    let package = if normalized.is_empty() {
        "app"
    } else {
        normalized
    };
    match language {
        ProjectLanguage::Rust => {
            // Keep the new project independent when its final parent is a Cargo workspace.
            let manifest = fs::read_to_string(root.join("Cargo.toml"))?;
            write(root, "Cargo.toml", &format!("{manifest}\n[workspace]\n"))?;
            if library {
                write(
                    root,
                    "src/lib.rs",
                    "pub fn greet(name: &str) -> String {\n    format!(\"Hello, {name}!\")\n}\n",
                )?;
            }
            write(root, ".gitignore", "/target/\n")?;
        }
        ProjectLanguage::Go => {
            write(
                root,
                "main.go",
                "package main\n\nimport \"fmt\"\n\nfunc main() {\n\tfmt.Println(\"Hello, Go!\")\n}\n",
            )?;
            write(root, ".gitignore", "*.exe\n/bin/\n")?;
        }
        ProjectLanguage::Python => {
            write(
                root,
                "main.py",
                "def main() -> None:\n    print(\"Hello, Python!\")\n\n\nif __name__ == \"__main__\":\n    main()\n",
            )?;
            write(
                root,
                "pyproject.toml",
                &format!(
                    "[project]\nname = \"{package}\"\nversion = \"0.1.0\"\nrequires-python = \">=3.10\"\n"
                ),
            )?;
            write(root, ".gitignore", "__pycache__/\n.venv/\n*.pyc\n")?;
        }
        ProjectLanguage::JavaScript | ProjectLanguage::TypeScript => {
            let typed = language == ProjectLanguage::TypeScript;
            let scripts = if typed {
                serde_json::json!({"build":"tsc -p tsconfig.json", "start":"node dist/index.js"})
            } else {
                serde_json::json!({"start":"node src/index.js"})
            };
            let mut manifest = serde_json::json!({"name":package, "version":"0.1.0", "private":true, "type":"module", "scripts":scripts});
            if typed {
                manifest["devDependencies"] = serde_json::json!({"typescript":"6.0.3"});
            }
            write(
                root,
                "package.json",
                &(serde_json::to_string_pretty(&manifest)? + "\n"),
            )?;
            if typed {
                write(
                    root,
                    "src/index.ts",
                    "const message: string = \"Hello, TypeScript!\";\nconsole.log(message);\n",
                )?;
                write(
                    root,
                    "tsconfig.json",
                    "{\n  \"compilerOptions\": {\n    \"target\": \"ES2022\",\n    \"module\": \"NodeNext\",\n    \"strict\": true,\n    \"rootDir\": \"src\",\n    \"outDir\": \"dist\"\n  },\n  \"include\": [\"src/**/*.ts\"]\n}\n",
                )?;
            } else {
                write(
                    root,
                    "src/index.js",
                    "console.log(\"Hello, JavaScript!\");\n",
                )?;
                write(
                    root,
                    "jsconfig.json",
                    "{\n  \"compilerOptions\": { \"target\": \"ES2022\", \"module\": \"NodeNext\" },\n  \"include\": [\"src/**/*.js\"]\n}\n",
                )?;
            }
            write(root, ".gitignore", "node_modules/\ndist/\n")?;
        }
    }
    let commands = match language {
        ProjectLanguage::Rust if library => "cargo check\ncargo build",
        ProjectLanguage::Rust => "cargo run",
        ProjectLanguage::Go => "go run .",
        ProjectLanguage::Python => "python main.py",
        ProjectLanguage::JavaScript => "npm start",
        ProjectLanguage::TypeScript => "npm install\nnpm run build\nnpm start",
    };
    write(
        root,
        "README.md",
        &format!(
            "# {name}\n\n{} project.\n\n```sh\n{commands}\n```\n",
            language.label()
        ),
    )?;
    Ok(())
}

fn copy(source: &Path, target: &Path) -> Result<()> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let output = target.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            fs::create_dir(&output)?;
            copy(&entry.path(), &output)?;
        } else {
            let mut output = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(output)?;
            std::io::copy(&mut fs::File::open(entry.path())?, &mut output)?;
        }
    }
    Ok(())
}

pub(super) async fn create(
    request: CreateProject,
    name: String,
    environment: Vec<(String, String)>,
) -> Result<PathBuf> {
    let tool = toolchain::probe(request.language, request.toolchain).await?;
    let stage = tokio::task::spawn_blocking(move || prepare(&request.location)).await??;
    match request.language {
        ProjectLanguage::Rust => {
            toolchain::run(
                &tool.executable,
                &[
                    "init",
                    "--vcs",
                    "none",
                    "--edition",
                    "2024",
                    "--name",
                    &name,
                    if request.library { "--lib" } else { "--bin" },
                    ".",
                ],
                Some(&stage.directory),
                &environment,
            )
            .await?;
        }
        ProjectLanguage::Go => {
            toolchain::run(
                &tool.executable,
                &["mod", "init", &request.module],
                Some(&stage.directory),
                &environment,
            )
            .await?;
        }
        _ => {}
    }
    tokio::task::spawn_blocking(move || {
        templates(&stage, request.language, &name, request.library)?;
        // Reserve only after initialization succeeds; never overwrite an existing project.
        fs::create_dir(&stage.destination).context("目标目录已存在或无法创建")?;
        copy(&stage.directory, &stage.destination).with_context(|| format!("复制项目失败，请检查 {}", stage.destination.display()))?;
        tracing::info!(language = request.language.label(), path = %stage.destination.display(), "project initialized");
        Ok(stage.destination.clone())
    }).await?
}
