use crate::{
    application::{APPLICATION, lifecycle::run_application},
    data_context::DATA_CONTEXT,
    event_bus::gui_event::GuiEvent,
    gui::workspace::{
        bottom_bar::Event as BottomBarEvent,
        editor::{Editor, ProjectFile},
    },
};
use gpui_kit::{ClipboardItem, Context};

impl Editor {
    pub(in crate::gui::workspace::editor) fn copy_file_name(
        &self,
        file: &ProjectFile,
        cx: &mut Context<Self>,
    ) {
        cx.write_to_clipboard(ClipboardItem::new_string(file.name()));
        self.send(GuiEvent::BottomBar(BottomBarEvent::Notice(Some(
            "已复制文件名".into(),
        ))));
    }

    pub(in crate::gui::workspace::editor) fn copy_current_path(
        &self,
        absolute: bool,
        cx: &mut Context<Self>,
    ) {
        if self.state.open_files.is_empty() {
            return;
        }
        let file = self.state.selected_file.path(DATA_CONTEXT.project_root());
        let path = if absolute {
            file.as_path()
        } else {
            let Ok(relative) = file.strip_prefix(DATA_CONTEXT.project_root()) else {
                return;
            };
            if !DATA_CONTEXT.has_project() {
                return;
            }
            relative
        };
        cx.write_to_clipboard(ClipboardItem::new_string(
            crate::component::path::display_path(path),
        ));
        self.send(GuiEvent::BottomBar(BottomBarEvent::Notice(Some(
            if absolute {
                "已复制全盘路径"
            } else {
                "已复制项目路径"
            }
            .into(),
        ))));
    }

    pub(in crate::gui::workspace::editor) fn reveal_current_file(&self, cx: &mut Context<Self>) {
        if self.state.open_files.is_empty() {
            return;
        }
        let path = self.state.selected_file.path(DATA_CONTEXT.project_root());
        cx.spawn(async move |this, cx| {
            let result =
                run_application(async move { APPLICATION.documents.reveal_location(path).await })
                    .await;
            let _ = this.update(cx, |this, _| {
                let notice = match result {
                    Ok(()) => "已打开文件所在目录".to_owned(),
                    Err(error) => {
                        tracing::warn!(%error, "editor file location reveal failed");
                        format!("无法打开文件所在目录：{error:#}")
                    }
                };
                this.send(GuiEvent::BottomBar(BottomBarEvent::Notice(Some(notice))));
            });
        })
        .detach();
    }
}
