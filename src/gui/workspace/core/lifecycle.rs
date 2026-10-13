use super::super::Workspace;
use crate::application::APPLICATION;
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
            || !self.saving_files.is_empty()
            || DATA_CONTEXT.documents.has_pending_save()
        {
            self.notice = Some("保存正在进行，请稍后关闭窗口".into());
            cx.notify();
            return;
        }
        let dirty = DATA_CONTEXT.documents.dirty_documents();
        if dirty.is_empty() {
            self.closing = true;
            self.autosave_timers.clear();
            self.close_with_session(window, cx);
            return;
        }
        if window.has_active_dialog(cx) {
            return;
        }
        self.close_prompt_open = true;
        self.autosave_timers.clear();
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
                                        this.autosave_timers.clear();
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
        self.autosave_timers.clear();
        let paths = DATA_CONTEXT
            .documents
            .dirty_documents()
            .into_iter()
            .map(|doc| doc.path.clone())
            .collect::<Vec<_>>();

        let documents = &APPLICATION.documents;
        cx.spawn_in(window, async move |this, cx| {
            let result = run_application(async move {
                for path in paths {
                    documents.save(path).await?;
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
