use crate::domain::settings::FontAsset;
use anyhow::{Context as _, Result, ensure};
use std::{collections::BTreeSet, io::Read, path::Path};

const MAX_FONT: u64 = 16 * 1024 * 1024;

const MAX_CACHE: usize = 128 * 1024 * 1024;

fn directory() -> std::path::PathBuf {
    crate::infrastructure::paths::get().data.join("fonts")
}

fn supported(path: &Path) -> bool {
    path.extension().is_some_and(|extension| {
        ["ttf", "otf", "ttc"]
            .iter()
            .any(|kind| extension.eq_ignore_ascii_case(kind))
    })
}

fn read(path: &Path, limit: u64) -> Result<Vec<u8>> {
    let file = std::fs::File::open(path)?;
    ensure!(
        file.metadata()?.len() <= limit,
        "字体文件或 VSIX 超过大小限制"
    );
    let mut bytes = Vec::new();
    file.take(limit + 1).read_to_end(&mut bytes)?;
    ensure!(bytes.len() as u64 <= limit, "字体文件或 VSIX 超过大小限制");
    Ok(bytes)
}

fn parse(bytes: Vec<u8>) -> Result<FontAsset> {
    let count = ttf_parser::fonts_in_collection(&bytes).unwrap_or(1);
    ensure!(count > 0 && count <= 64, "字体集合数量无效");
    let mut families = BTreeSet::new();
    for index in 0..count {
        let face = ttf_parser::Face::parse(&bytes, index)
            .context("字体格式无效，仅支持 TTF / OTF / TTC")?;
        let names = face.names();
        let family = [
            ttf_parser::name_id::TYPOGRAPHIC_FAMILY,
            ttf_parser::name_id::FAMILY,
        ]
        .into_iter()
        .find_map(|id| {
            names.into_iter().find_map(|name| {
                (name.name_id == id)
                    .then(|| name.to_string())
                    .flatten()
                    .filter(|name| !name.is_empty())
            })
        })
        .context("字体缺少可用的字体家族名称")?;
        families.insert(family);
    }
    let hash = bytes.iter().fold(0xcbf29ce484222325u64, |hash, byte| {
        (hash ^ *byte as u64).wrapping_mul(0x100000001b3)
    });
    Ok(FontAsset {
        id: format!("{hash:016x}"),
        families: families.into_iter().collect(),
        bytes: bytes.into(),
    })
}

pub(super) fn load() -> Result<Vec<FontAsset>> {
    let directory = directory();
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut fonts = Vec::new();
    let mut total = 0;
    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() || !supported(&entry.path()) {
            continue;
        }
        let result = read(&entry.path(), MAX_FONT).and_then(parse);
        match result {
            Ok(font) => {
                total += font.bytes.len();
                ensure!(total <= MAX_CACHE, "已导入字体超过 128 MB");
                fonts.push(font);
            }
            Err(error) => {
                tracing::warn!(path = %entry.path().display(), %error, "ignoring invalid cached font")
            }
        }
    }
    Ok(fonts)
}
