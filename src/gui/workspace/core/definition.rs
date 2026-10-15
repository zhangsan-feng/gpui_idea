use super::super::{ProjectFile, Workspace};
use crate::{
    application::{definition, lifecycle::run_application},
    data_context::DATA_CONTEXT,
};
use gpui_kit::{component::WindowExt as _, prelude::*, *};

impl Workspace {
    pub(in crate::gui::workspace) fn go_to_definition(
        &mut self,
        file: ProjectFile,
        offset: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.closing || self.file_operation_running || window.has_active_dialog(cx) {
            return;
        }
        let Some(editor) = self.editors.get(&file) else {
            return;
        };
        let buffer = editor.read(cx).text().clone();
        let path = file.path(DATA_CONTEXT.project_root());
        let revision = DATA_CONTEXT
            .documents
            .snapshot(&path)
            .map(|doc| doc.revision);
        let language = crate::component::code_editor::language_for_path(file.relative_path());
        crate::component::code_editor::dismiss_completions(editor, cx);
        self.definition_request = self.definition_request.wrapping_add(1);
        let request = self.definition_request;
        let (cancel, cancelled) = tokio::sync::watch::channel(false);
        let observed_path = path.clone();
        self.definition_observer = Some(cx.observe_in(
            editor,
            window,
            move |this, editor, window, cx| {
                if this.definition_request != request {
                    return;
                }
                let state = editor.read(cx);
                if state.cursor() == offset
                    && state.selected_range().is_empty()
                    && state.focus_handle(cx).is_focused(window)
                    && DATA_CONTEXT
                        .documents
                        .snapshot(&observed_path)
                        .map(|document| document.revision)
                        == revision
                {
                    return;
                }
                this.definition_request = this.definition_request.wrapping_add(1);
                this.definition_observer = None;
                if this.notice.as_deref() == Some("正在定位源码…首次启动语言服务可能需要索引项目")
                {
                    this.notice = None;
                }
                let _ = cancel.send(true);
                cx.notify();
            },
        ));
        self.notice = Some("正在定位源码…首次启动语言服务可能需要索引项目".into());
        tracing::info!(file = %path.display(), offset, language, request, "go to definition requested");
        cx.notify();
        cx.spawn_in(window, async move |this, cx| {
            let source_path = path.clone();
            let result = run_application(async move {
                definition::resolve(path, buffer, offset, language, cancelled).await
            })
            .await;
            let _ = this.update_in(cx, |this, window, cx| {
                if this.definition_request == request {
                    this.definition_observer = None;
                }
                if this.definition_request != request
                    || this.closing
                    || this.selected_file != file
                    || !window.is_window_active()
                    || this.editors.get(&file).is_none_or(|editor| {
                        editor.read(cx).cursor() != offset
                            || !editor.read(cx).focus_handle(cx).is_focused(window)
                    })
                    || DATA_CONTEXT
                        .documents
                        .snapshot(&source_path)
                        .map(|doc| doc.revision)
                        != revision
                {
                    if this.definition_request == request
                        && this.notice.as_deref()
                            == Some("正在定位源码…首次启动语言服务可能需要索引项目")
                    {
                        this.notice = None;
                        cx.notify();
                    }
                    return;
                }
                match result {
                    Ok(result) if result.targets.is_empty() => {
                        this.notice = Some(format!(
                            "未找到 {} 的定义；语言服务可能仍在索引，或该符号没有可访问源码",
                            result.symbol
                        ));
                    }
                    Ok(result) if result.targets.len() == 1 => {
                        this.open_definition_target(
                            result.targets.into_iter().next().unwrap(),
                            window,
                            cx,
                        );
                        this.notice = None;
                    }
                    Ok(result) => {
                        this.notice = Some(format!(
                            "{} 找到 {} 个定义，请选择目标",
                            result.symbol,
                            result.targets.len()
                        ));
                        this.show_definition_targets(result, window, cx);
                    }
                    Err(error) => {
                        tracing::warn!(%error, request, "go to definition failed");
                        this.notice = Some(format!("源码跳转失败：{error:#}"));
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn open_definition_target(
        &mut self,
        mut target: crate::domain::project::ContentMatch,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !target.path.starts_with(DATA_CONTEXT.project_root()) {
            self.external_source_files
                .insert(self.project_file(&target.path));
        }
        target.path = self.project_file(&target.path).0;
        self.open_content_match(target, window, cx);
    }

    fn show_definition_targets(
        &mut self,
        result: crate::domain::definition::DefinitionResult,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let view = cx.weak_entity();
        let targets = result.targets;
        let title = format!(
            "选择 {} 的定义{}",
            result.symbol,
            if result.truncated {
                "（仅显示前 100 项）"
            } else {
                ""
            }
        );
        window.open_dialog(cx, move |dialog, _, _| {
            let mut rows = div()
                .id("definition-targets")
                .flex()
                .flex_col()
                .max_h(px(360.))
                .overflow_y_scroll();
            for (index, target) in targets.iter().enumerate() {
                let target = target.clone();
                let view = view.clone();
                rows = rows.child(
                    div()
                        .id(index)
                        .flex()
                        .flex_col()
                        .flex_shrink_0()
                        .p_2()
                        .gap_1()
                        .rounded_sm()
                        .cursor_pointer()
                        .hover(|row| row.bg(rgb(0xe8eefb)))
                        .child(div().truncate().child(format!(
                            "{}:{}:{}",
                            target.path.display(),
                            target.line,
                            target.column
                        )))
                        .child(
                            div()
                                .truncate()
                                .font_family("Cascadia Mono")
                                .text_size(px(12.))
                                .text_color(rgb(0x737b87))
                                .child(target.preview.clone()),
                        )
                        .on_click(move |_, window, cx| {
                            window.close_dialog(cx);
                            let _ = view.update(cx, |this, cx| {
                                this.open_definition_target(target.clone(), window, cx)
                            });
                        }),
                );
            }
            dialog.title(title.clone()).w(px(780.)).child(rows)
        });
    }
}
