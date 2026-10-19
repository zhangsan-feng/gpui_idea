use crate::application::lifecycle::run_application;
use crate::data_context::DATA_CONTEXT;
use crate::gui::workspace::ProjectFile;
use crate::gui::workspace::editor::Editor;
use gpui_kit::{Context, Window};

impl Editor {
    pub(in crate::gui::workspace) fn save_current(
        &mut self,
        _: &mut Window,
        cx: &mut Context<Editor>,
    ) {
        if self.state.open_files.is_empty() {
            return;
        }
        self.send(crate::event_bus::gui_event::GuiEvent::WindowBar(
            crate::gui::workspace::window_bar::Event::ScheduleSave,
        ));
        self.save_file(self.state.selected_file.clone(), false, cx);
    }

    pub(in crate::gui::workspace) fn save_file(
        &mut self,
        file: ProjectFile,
        automatic: bool,
        cx: &mut Context<Editor>,
    ) {
        if self.peer_window_bar(cx).read(cx).state.closing
            || self.peer_window_bar(cx).read(cx).state.close_prompt_open
        {
            return;
        }
        let Some(document) = DATA_CONTEXT
            .documents
            .snapshot(&file.path(DATA_CONTEXT.project_root()))
        else {
            return;
        };
        if (!document.dirty() && (automatic || !DATA_CONTEXT.settings.get().auto_format))
            || document.saving
            || (automatic && document.save_error.is_some())
            || !self.state.saving_files.insert(file.clone())
        {
            return;
        }
        self.send(crate::event_bus::gui_event::GuiEvent::BottomBar(
            crate::gui::workspace::bottom_bar::Event::Notice(Some(format!(
                "正在{}保存 {}…",
                if automatic { "自动" } else { "" },
                file.name()
            ))),
        ));
        tracing::info!(file = %file.relative_path().display(), revision = document.revision, automatic, "document save requested");

        let path = document.path.clone();
        cx.spawn(async move |this, cx| {
            let result = run_application(async move { crate::application::APPLICATION.documents.save(path).await }).await;
            let _ = this.update(cx, |this, cx| { {
                this.state.saving_files.remove(&file);
                match result {
                    Ok(()) => {
                        let dirty = DATA_CONTEXT.documents.snapshot(&file.path(DATA_CONTEXT.project_root())).is_some_and(|document| document.dirty());
                        if dirty {
                            // Preserve a timer already scheduled by newer edits.
                            if this.peer_window_bar(cx).read(cx).state.persistence_timer.is_none() {
                                this.send(crate::event_bus::gui_event::GuiEvent::WindowBar(crate::gui::workspace::window_bar::Event::ScheduleSave));
                            }
                            this.send(crate::event_bus::gui_event::GuiEvent::BottomBar(crate::gui::workspace::bottom_bar::Event::Notice(Some(format!("{} 有新修改，等待自动保存", file.name())))));
                        } else {
                            this.send(crate::event_bus::gui_event::GuiEvent::BottomBar(crate::gui::workspace::bottom_bar::Event::Notice(Some(format!("已{}保存 {}", if automatic { "自动" } else { "" }, file.name())))));
                        }
                        this.prune_editors();
                    }
                    Err(error) => {
                        tracing::error!(file = %file.relative_path().display(), automatic, error = %error, "document automatic saving paused after save failure");
                        this.send(crate::event_bus::gui_event::GuiEvent::BottomBar(crate::gui::workspace::bottom_bar::Event::Notice(Some(format!("{} 保存失败：{error:#}；Ctrl+S 可重试", file.name())))));
                    }
                }
                crate::event_bus::EVENT_BUS.refresh_gui.notify(&[crate::event_bus::refresh_gui::GuiComponent::Editor, crate::event_bus::refresh_gui::GuiComponent::FileTree, crate::event_bus::refresh_gui::GuiComponent::BottomBar]);
             } });
        }).detach();
        crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
            crate::event_bus::refresh_gui::GuiComponent::Editor,
            crate::event_bus::refresh_gui::GuiComponent::FileTree,
            crate::event_bus::refresh_gui::GuiComponent::BottomBar,
        ]);
    }
}
