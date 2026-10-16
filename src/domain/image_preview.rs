use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Clone, Copy, Debug)]
pub(crate) enum PreviewImageFormat {
    Png,
    Jpeg,
    Gif,
    Webp,
    Bmp,
    Ico,
    Svg,
    Tiff,
    Pnm,
}

impl PreviewImageFormat {
    pub(crate) fn for_path(path: &Path) -> Option<Self> {
        Some(
            match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
                "png" => Self::Png,
                "jpg" | "jpeg" | "jpe" => Self::Jpeg,
                "gif" => Self::Gif,
                "webp" => Self::Webp,
                "bmp" => Self::Bmp,
                "ico" => Self::Ico,
                "svg" => Self::Svg,
                "tif" | "tiff" => Self::Tiff,
                "pnm" | "pbm" | "pgm" | "ppm" => Self::Pnm,
                _ => return None,
            },
        )
    }
}

pub(crate) struct ImagePreviewSnapshot {
    pub(crate) path: PathBuf,
    pub(crate) format: PreviewImageFormat,
    pub(crate) bytes: Arc<[u8]>,
    pub(crate) revision: u64,
    pub(crate) error: Option<String>,
}
