use gpui_kit::{AssetSource, Result, SharedString};
use std::{borrow::Cow, collections::BTreeSet};

gpui_kit::assets::icon_assets!(
    EditorIcons,
    [
        ArrowLeft,
        ArrowRight,
        BellDot,
        BugPlay,
        ChevronDown,
        ChevronRight,
        CirclePlay,
        Code,
        Database,
        EllipsisVertical,
        File,
        FileBox,
        FileCode,
        FileText,
        Folder,
        FolderClosed,
        FolderOpen,
        GitBranch,
        Hammer,
        LayoutPanelLeft,
        Menu,
        Minus,
        Play,
        RefreshCw,
        Save,
        Search,
        Settings,
        SlidersHorizontal,
        Square,
        Terminal,
        Workflow,
        X,
    ]
);

pub(crate) struct MergedAssets {
    editor_icons: EditorIcons,
    component_assets: gpui_kit::assets::Assets,
}

impl MergedAssets {
    pub(crate) fn new() -> Self {
        Self {
            editor_icons: EditorIcons,
            component_assets: gpui_kit::assets::Assets,
        }
    }
}

impl AssetSource for MergedAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if let Some(asset) = self.editor_icons.load(path)? {
            return Ok(Some(asset));
        }

        let result = self.component_assets.load(path);
        if let Err(error) = &result {
            if path.starts_with("icons/") {
                tracing::error!(
                    asset_path = path,
                    error = %error,
                    "icon lookup failed in editor and GPUI Kit asset bundles"
                );
            }
        }
        result
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut paths = BTreeSet::new();
        for asset_path in self.editor_icons.list(path)? {
            paths.insert(asset_path.to_string());
        }
        for asset_path in self.component_assets.list(path)? {
            paths.insert(asset_path.to_string());
        }

        Ok(paths.into_iter().map(SharedString::from).collect())
    }
}
