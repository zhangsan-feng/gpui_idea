use super::super::Workspace;
use crate::application::lifecycle::run_application;
use crate::data_context::DATA_CONTEXT;
use gpui_kit::component::{
    WindowExt as _,
    button::{Button, ButtonVariants as _},
    h_flex,
};
use gpui_kit::{Context, ParentElement as _, Window};

impl Workspace {
    pub(in crate::gui::workspace) fn request_close_core(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.closing
            || self.file_operation_running
            || !self.saving_files.is_empty()
            || DATA_CONTEXT.documents.has_pending_save()
        {
            self.notice = Some("文件操作或保存正在进行，请稍后关闭窗口".into());
            cx.notify();
            return;
        }
        let dirty = DATA_CONTEXT.documents.dirty_documents();
        if dirty.is_empty() {
            self.closing = true;
            self.persistence_timer.take();
            self.close_with_session(window, cx);
            return;
        }
        if window.has_active_dialog(cx) {
            return;
        }
        self.close_prompt_open = true;
        self.persistence_timer.take();
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
                            this.close_prompt_open = false;
                            this.resume_autosave(cx);
                            cx.notify();
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
                                        this.close_prompt_open = false;
                                        this.resume_autosave(cx);
                                        cx.notify();
                                    });
                                }
                            },
                        ))
                        .child(Button::new("discard-exit").label("放弃并关闭").on_click(
                            move |_, window, cx| {
                                window.close_dialog(cx);
                                if let Some(view) = discard.upgrade() {
                                    view.update(cx, |this, cx| {
                                        this.closing = true;
                                        this.persistence_timer.take();
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

    fn save_all_and_close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.close_prompt_open = false;
        self.closing = true;
        self.persistence_timer.take();
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
                this.closing = false;
                match result {
                    Ok(()) if DATA_CONTEXT.documents.dirty_documents().is_empty() => {
                        this.closing = true;
                        this.close_with_session(window, cx);
                    }
                    Ok(()) => {
                        this.notice = Some("保存过程中内容发生变化，窗口保持打开".into());
                        this.resume_autosave(cx);
                    }
                    Err(error) => {
                        this.notice = Some(format!("保存失败，窗口保持打开：{error:#}"));
                        this.resume_autosave(cx);
                    }
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }
}
