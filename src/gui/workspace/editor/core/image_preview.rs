use crate::gui::workspace::editor::Editor;
use crate::gui::workspace::{ImagePreviewView, ProjectFile};
use crate::{
    application::{APPLICATION, lifecycle::run_application},
    data_context::DATA_CONTEXT,
    domain::image_preview::PreviewImageFormat,
};
use gpui_kit::{Context, Image, ImageFormat, Window};
use std::sync::Arc;

impl Editor {
    pub(in crate::gui::workspace) fn open_image_preview(
        &mut self,
        file: ProjectFile,
        window: &mut Window,
        cx: &mut Context<Editor>,
    ) {
        let path = file.path(DATA_CONTEXT.project_root());
        if self.state.image_previews.get(&file).is_some_and(|view| {
            DATA_CONTEXT
                .images
                .snapshot(&path)
                .is_some_and(|snapshot| snapshot.revision == view.revision)
        }) {
            crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
                crate::event_bus::refresh_gui::GuiComponent::Editor,
                crate::event_bus::refresh_gui::GuiComponent::BottomBar,
                crate::event_bus::refresh_gui::GuiComponent::WindowBar,
            ]);
            return;
        }
        if !self.state.loading_files.insert(file.clone()) {
            return;
        }
        self.state.load_errors.remove(&file);
        crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
            crate::event_bus::refresh_gui::GuiComponent::Editor,
            crate::event_bus::refresh_gui::GuiComponent::BottomBar,
            crate::event_bus::refresh_gui::GuiComponent::WindowBar,
        ]);
        cx.spawn_in(window, async move |this, cx| {
            let result = run_application(async move {
                let snapshot = APPLICATION.documents.open_image_preview(path).await?;
                let image = if snapshot.error.is_some() { None } else {
                    Some(Arc::new(Image::from_bytes(match snapshot.format {
                        PreviewImageFormat::Png => ImageFormat::Png,
                        PreviewImageFormat::Jpeg => ImageFormat::Jpeg,
                        PreviewImageFormat::Gif => ImageFormat::Gif,
                        PreviewImageFormat::Webp => ImageFormat::Webp,
                        PreviewImageFormat::Bmp => ImageFormat::Bmp,
                        PreviewImageFormat::Ico => ImageFormat::Ico,
                        PreviewImageFormat::Svg => ImageFormat::Svg,
                        PreviewImageFormat::Tiff => ImageFormat::Tiff,
                        PreviewImageFormat::Pnm => ImageFormat::Pnm,
                    }, snapshot.bytes.to_vec())))
                };
                Ok(ImagePreviewView { image, revision: snapshot.revision, bytes: snapshot.bytes.len(), error: snapshot.error.clone() })
            }).await;
            let _ = this.update_in(cx, |this, window, cx| { {
                this.state.loading_files.remove(&file);
                if !this.state.open_files.contains(&file) {
                    APPLICATION.documents.release_image_preview(&file.path(DATA_CONTEXT.project_root()));
                    return;
                }
                match result {
                    Ok(view) => { this.state.image_previews.insert(file.clone(), view); }
                    Err(error) => {
                        tracing::warn!(file = %file.relative_path().display(), %error, "image preview open failed");
                        this.state.load_errors.insert(file, format!("{error:#}"));
                    }
                }
                this.sync_image_previews(window, cx);
                crate::event_bus::EVENT_BUS.refresh_gui.notify(&[crate::event_bus::refresh_gui::GuiComponent::Editor, crate::event_bus::refresh_gui::GuiComponent::BottomBar, crate::event_bus::refresh_gui::GuiComponent::WindowBar]);
             } });
        }).detach();
    }

    pub(in crate::gui::workspace) fn sync_image_previews(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Editor>,
    ) {
        for file in self.state.open_files.clone() {
            if let Some(snapshot) = DATA_CONTEXT
                .images
                .snapshot(&file.path(DATA_CONTEXT.project_root()))
            {
                if !self
                    .state
                    .image_previews
                    .get(&file)
                    .is_some_and(|view| view.revision == snapshot.revision)
                {
                    self.open_image_preview(file, window, cx);
                }
            }
        }
    }

    pub(in crate::gui::workspace) fn release_image_tab(&mut self, file: &ProjectFile) {
        if PreviewImageFormat::for_path(file.relative_path()).is_none() {
            return;
        }
        self.state.image_previews.remove(file);
        self.state.editor_order.retain(|cached| cached != file);
        APPLICATION
            .documents
            .release_image_preview(&file.path(DATA_CONTEXT.project_root()));
    }
}
