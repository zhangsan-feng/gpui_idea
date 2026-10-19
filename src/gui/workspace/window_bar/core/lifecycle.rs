use crate::application::lifecycle::run_application;
use crate::data_context::DATA_CONTEXT;
use crate::gui::workspace::window_bar::WindowBar;
use gpui_kit::component::{
    WindowExt as _,
    button::{Button, ButtonVariants as _},
    h_flex,
};
use gpui_kit::{Context, ParentElement as _, Window};

impl WindowBar {
    pub(in crate::gui::workspace) fn request_close_core(
        &mut self,
        window: &mut Window,
        cx: &mut Context<WindowBar>,
    ) {
        if self.state.closing || self.state.close_prompt_open {
            return;
        }
        crate::application::APPLICATION.documents.pause_autosave();
        if self
            .peer_file_tree(cx)
            .read(cx)
            .state
            .file_operation_running
            || !self.peer_editor(cx).read(cx).state.saving_files.is_empty()
            || DATA_CONTEXT.documents.has_pending_save()
        {
            self.resume_autosave(cx);
            self.send(crate::event_bus::gui_event::GuiEvent::BottomBar(
                crate::gui::workspace::bottom_bar::Event::Notice(Some(
                    "文件操作或保存正在进行，请稍后关闭窗口".into(),
                )),
            ));
            crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
                crate::event_bus::refresh_gui::GuiComponent::Editor,
                crate::event_bus::refresh_gui::GuiComponent::FileTree,
                crate::event_bus::refresh_gui::GuiComponent::BottomBar,
            ]);
            return;
        }
        let dirty = DATA_CONTEXT.documents.dirty_documents();
        if dirty.is_empty() {
            self.state.closing = true;
            self.state.persistence_timer.take();
            self.close_with_session(window, cx);
            return;
        }
        if window.has_active_dialog(cx) {
            self.resume_autosave(cx);
            return;
        }
        self.state.close_prompt_open = true;
        self.state.persistence_timer.take();
        let view = cx.entity().downgrade();
        let message = format!(
            "有 {} 个文件尚未保存。保存全部后关闭，或明确放弃这些编辑。",
            dirty.len()
        );
        window.open_dialog(cx, move |dialog, _, _| {
            let save = view.clone();
            let cancel = view.clone();
            let discard = view.clone();
            let dismiss = view.clone();
            dialog
                .title("未保存的编辑")
                .child(message.clone())
                .overlay_closable(false)
                .on_cancel(move |_, _, cx| {
                    if let Some(view) = dismiss.upgrade() {
                        view.update(cx, |this, cx| {
                            this.state.close_prompt_open = false;
                            this.resume_autosave(cx);
                            crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
                                crate::event_bus::refresh_gui::GuiComponent::Editor,
                                crate::event_bus::refresh_gui::GuiComponent::FileTree,
                                crate::event_bus::refresh_gui::GuiComponent::BottomBar,
                            ]);
                        });
                    }
                    true
                })
                .footer(
                    h_flex()
                        .child(Button::new("cancel-exit").label("取消").on_click(
                            move |_, window, cx| {
                                window.close_dialog(cx);
                                if let Some(view) = cancel.upgrade() {
                                    view.update(cx, |this, cx| {
                                        this.state.close_prompt_open = false;
                                        this.resume_autosave(cx);
                                        crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
                                            crate::event_bus::refresh_gui::GuiComponent::Editor,
                                            crate::event_bus::refresh_gui::GuiComponent::FileTree,
                                            crate::event_bus::refresh_gui::GuiComponent::BottomBar,
                                        ]);
                                    });
                                }
                            },
                        ))
                        .child(Button::new("discard-exit").label("放弃并关闭").on_click(
                            move |_, window, cx| {
                                window.close_dialog(cx);
                                if let Some(view) = discard.upgrade() {
                                    view.update(cx, |this, cx| {
                                        this.state.closing = true;
                                        this.state.persistence_timer.take();
                                        this.close_with_session(window, cx);
                                    });
                                }
                            },
                        ))
                        .child(
                            Button::new("save-exit")
                                .primary()
                                .label("保存全部并关闭")
                                .on_click(move |_, window, cx| {
                                    window.close_dialog(cx);
                                    if let Some(view) = save.upgrade() {
                                        view.update(cx, |this, cx| {
                                            this.save_all_and_close(window, cx)
                                        });
                                    }
                                }),
                        ),
                )
        });
    }

    pub(in crate::gui::workspace) fn save_all_and_close(
        &mut self,
        window: &mut Window,
        cx: &mut Context<WindowBar>,
    ) {
        self.state.close_prompt_open = false;
        self.state.closing = true;
        self.state.persistence_timer.take();
        let paths = DATA_CONTEXT
            .documents
            .dirty_documents()
            .into_iter()
            .map(|doc| doc.path.clone())
            .collect::<Vec<_>>();

        cx.spawn_in(window, async move |this, cx| {
            let result = run_application(async move {
                for path in paths {
                    crate::application::APPLICATION.documents.save(path).await?;
                }
                Ok(())
            })
            .await;
            let _ = this.update_in(cx, |this, window, cx| {
                this.state.closing = false;
                match result {
                    Ok(()) if DATA_CONTEXT.documents.dirty_documents().is_empty() => {
                        this.state.closing = true;
                        this.close_with_session(window, cx);
                    }
                    Ok(()) => {
                        this.send(crate::event_bus::gui_event::GuiEvent::BottomBar(
                            crate::gui::workspace::bottom_bar::Event::Notice(Some(
                                "保存过程中内容发生变化，窗口保持打开".into(),
                            )),
                        ));
                        this.resume_autosave(cx);
                    }
                    Err(error) => {
                        this.send(crate::event_bus::gui_event::GuiEvent::BottomBar(
                            crate::gui::workspace::bottom_bar::Event::Notice(Some(format!(
                                "保存失败，窗口保持打开：{error:#}"
                            ))),
                        ));
                        this.resume_autosave(cx);
                    }
                }
                crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
                    crate::event_bus::refresh_gui::GuiComponent::Editor,
                    crate::event_bus::refresh_gui::GuiComponent::FileTree,
                    crate::event_bus::refresh_gui::GuiComponent::BottomBar,
                ]);
            });
        })
        .detach();
        crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
            crate::event_bus::refresh_gui::GuiComponent::Editor,
            crate::event_bus::refresh_gui::GuiComponent::FileTree,
            crate::event_bus::refresh_gui::GuiComponent::BottomBar,
        ]);
    }
}
