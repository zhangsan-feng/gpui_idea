use crate::gui::workspace::ProjectFile;
use crate::gui::workspace::editor::Editor;
use crate::{
    application::{definition, lifecycle::run_application},
    data_context::DATA_CONTEXT,
};
use gpui_kit::{component::WindowExt as _, prelude::*, *};

impl Editor {
    pub(in crate::gui::workspace) fn go_to_definition(
        &mut self,
        file: ProjectFile,
        offset: usize,
        window: &mut Window,
        cx: &mut Context<Editor>,
    ) {
        if self.peer_window_bar(cx).read(cx).state.closing
            || self
                .peer_file_tree(cx)
                .read(cx)
                .state
                .file_operation_running
            || window.has_active_dialog(cx)
        {
            return;
        }
        let Some(editor) = self.state.editors.get(&file) else {
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
        self.state.definition_request = self.state.definition_request.wrapping_add(1);
        let request = self.state.definition_request;
        let (cancel, cancelled) = tokio::sync::watch::channel(false);
        let observed_path = path.clone();
        self.state.definition_observer =
            Some(
                cx.observe_in(editor, window, move |this, editor, window, cx| {
                    if this.state.definition_request != request {
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
                    this.state.definition_request = this.state.definition_request.wrapping_add(1);
                    this.state.definition_observer = None;
                    {
                        this.send(crate::event_bus::gui_event::GuiEvent::BottomBar(
                            crate::gui::workspace::bottom_bar::Event::ClearNoticeIf(
                                "正在定位源码…首次启动语言服务可能需要索引项目".into(),
                            ),
                        ));
                    }
                    let _ = cancel.send(true);
                    crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
                        crate::event_bus::refresh_gui::GuiComponent::Editor,
                        crate::event_bus::refresh_gui::GuiComponent::BottomBar,
                        crate::event_bus::refresh_gui::GuiComponent::WindowBar,
                    ]);
                }),
            );
        self.send(crate::event_bus::gui_event::GuiEvent::BottomBar(
            crate::gui::workspace::bottom_bar::Event::Notice(Some(
                "正在定位源码…首次启动语言服务可能需要索引项目".into(),
            )),
        ));
        tracing::info!(file = %path.display(), offset, language, request, "go to definition requested");
        crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
            crate::event_bus::refresh_gui::GuiComponent::Editor,
            crate::event_bus::refresh_gui::GuiComponent::BottomBar,
            crate::event_bus::refresh_gui::GuiComponent::WindowBar,
        ]);
        cx.spawn_in(window, async move |this, cx| {
            let source_path = path.clone();
            let result = run_application(async move {
                definition::resolve(path, buffer, offset, language, cancelled).await
            })
            .await;
            let _ = this.update_in(cx, |this, window, cx| {
                if this.state.definition_request == request {
                    this.state.definition_observer = None;
                }
                if this.state.definition_request != request
                    || this.peer_window_bar(cx).read(cx).state.closing
                    || this.state.selected_file != file
                    || !window.is_window_active()
                    || this.state.editors.get(&file).is_none_or(|editor| {
                        editor.read(cx).cursor() != offset
                            || !editor.read(cx).focus_handle(cx).is_focused(window)
                    })
                    || DATA_CONTEXT
                        .documents
                        .snapshot(&source_path)
                        .map(|doc| doc.revision)
                        != revision
                {
                    if this.state.definition_request == request {
                        this.send(crate::event_bus::gui_event::GuiEvent::BottomBar(
                            crate::gui::workspace::bottom_bar::Event::ClearNoticeIf(
                                "正在定位源码…首次启动语言服务可能需要索引项目".into(),
                            ),
                        ));
                        crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
                            crate::event_bus::refresh_gui::GuiComponent::Editor,
                            crate::event_bus::refresh_gui::GuiComponent::BottomBar,
                            crate::event_bus::refresh_gui::GuiComponent::WindowBar,
                        ]);
                    }
                    return;
                }
                match result {
                    Ok(result) if result.targets.is_empty() => {
                        this.send(crate::event_bus::gui_event::GuiEvent::BottomBar(
                            crate::gui::workspace::bottom_bar::Event::Notice(Some(format!(
                                "未找到 {} 的定义；语言服务可能仍在索引，或该符号没有可访问源码",
                                result.symbol
                            ))),
                        ));
                    }
                    Ok(result) if result.targets.len() == 1 => {
                        this.open_definition_target(
                            result.targets.into_iter().next().unwrap(),
                            window,
                            cx,
                        );
                        this.send(crate::event_bus::gui_event::GuiEvent::BottomBar(
                            crate::gui::workspace::bottom_bar::Event::ClearNoticeIf(
                                "正在定位源码…首次启动语言服务可能需要索引项目".into(),
                            ),
                        ));
                    }
                    Ok(result) => {
                        this.send(crate::event_bus::gui_event::GuiEvent::BottomBar(
                            crate::gui::workspace::bottom_bar::Event::Notice(Some(format!(
                                "{} 找到 {} 个定义，请选择目标",
                                result.symbol,
                                result.targets.len()
                            ))),
                        ));
                        this.show_definition_targets(result, window, cx);
                    }
                    Err(error) => {
                        tracing::warn!(%error, request, "go to definition failed");
                        this.send(crate::event_bus::gui_event::GuiEvent::BottomBar(
                            crate::gui::workspace::bottom_bar::Event::Notice(Some(format!(
                                "源码跳转失败：{error:#}"
                            ))),
                        ));
                    }
                }
                crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
                    crate::event_bus::refresh_gui::GuiComponent::Editor,
                    crate::event_bus::refresh_gui::GuiComponent::BottomBar,
                    crate::event_bus::refresh_gui::GuiComponent::WindowBar,
                ]);
            });
        })
        .detach();
    }

    pub(in crate::gui::workspace) fn open_definition_target(
        &mut self,
        mut target: crate::domain::project::ContentMatch,
        window: &mut Window,
        cx: &mut Context<Editor>,
    ) {
        if !target.path.starts_with(DATA_CONTEXT.project_root()) {
            self.state
                .external_source_files
                .insert(ProjectFile::for_path(&target.path));
        }
        target.path = ProjectFile::for_path(&target.path).0;
        self.open_content_match(target, window, cx);
    }

    pub(in crate::gui::workspace) fn show_definition_targets(
        &mut self,
        result: crate::domain::definition::DefinitionResult,
        window: &mut Window,
        cx: &mut Context<Editor>,
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
