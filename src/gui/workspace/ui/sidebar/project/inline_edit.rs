use super::*;
use gpui_kit::component::input::{Escape, Input};
use gpui_kit::component::tooltip::Tooltip;

impl Workspace {
    pub(super) fn inline_file_row(&self, depth: usize, cx: &mut Context<Self>) -> AnyElement {
        let Some(edit) = &self.inline_file_edit else {
            return div().into_any_element();
        };
        let palette = ProjectTreePalette::from_theme(cx.theme());
        let rename = matches!(
            edit.action,
            crate::gui::workspace::FileTreeMenuAction::Rename
        );
        let path = if rename {
            edit.source.clone()
        } else {
            edit.directory.join(edit.input.read(cx).value().as_str())
        };
        h_flex()
            .id("inline-project-file")
            .w_full()
            .min_w_0()
            .h(px(PROJECT_TREE_ROW_HEIGHT))
            .pl(px(
                PROJECT_TREE_LEFT_INSET + depth as f32 * PROJECT_TREE_INDENT
            ))
            .pr_2()
            .gap_1()
            .items_center()
            .child(div().w(px(13.)).flex_shrink_0())
            .child(project_entry_icon(&path, edit.is_directory, false, palette))
            .child(
                div()
                    .id("inline-project-file-input")
                    .flex_1()
                    .min_w_0()
                    .h(px(22.))
                    .border_1()
                    .border_color(if edit.error.is_some() {
                        palette.error
                    } else {
                        palette.accent
                    })
                    .overflow_hidden()
                    .when_some(edit.error.clone(), |row, error| {
                        row.tooltip(move |window, cx| Tooltip::new(error.clone()).build(window, cx))
                    })
                    .on_action(cx.listener(|this, _: &Escape, _, cx| {
                        this.cancel_inline_file_edit(cx);
                    }))
                    .child(
                        Input::new(&edit.input)
                            .small()
                            .appearance(false)
                            .bordered(false)
                            .disabled(self.file_operation_running)
                            .h(px(20.))
                            .w_full(),
                    ),
            )
            .into_any_element()
    }
}
