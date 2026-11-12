use super::super::*;

impl Editor {
    pub(in crate::gui) fn markdown_mode(&self) -> Option<MarkdownMode> {
        (crate::gui::component::code_editor::language_for_path(
            self.state.selected_file.relative_path(),
        ) == "markdown")
            .then(|| {
                self.state
                    .markdown_modes
                    .get(&self.state.selected_file)
                    .copied()
                    .unwrap_or_default()
            })
    }

    pub(in crate::gui) fn set_markdown_mode(
        &mut self,
        mode: MarkdownMode,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.markdown_mode().is_none() || self.markdown_mode() == Some(mode) {
            return;
        }
        self.state
            .markdown_modes
            .insert(self.state.selected_file.clone(), mode);
        let path = self.state.selected_file.path(DATA_CONTEXT.project_root());
        if let Some(view) = self.state.pending_file_views.get_mut(&path) {
            view.markdown_mode = mode;
        }
        self.send(crate::event_bus::gui_event::GuiEvent::WindowBar(
            crate::gui::main_window::window_bar::Event::ScheduleSave,
        ));
        if mode != MarkdownMode::Rendered {
            if let Some(editor) = self.state.editors.get(&self.state.selected_file) {
                editor.update(cx, |state, cx| state.focus(window, cx));
            }
        }
        tracing::debug!(file = %self.state.selected_file.relative_path().display(), ?mode, "markdown view changed");
        cx.notify();
    }
}

/// Resolve document image references without reading files on the GUI thread.
/// GPUI's image cache loads the returned resources asynchronously.
impl Editor {
    pub(in crate::gui) fn markdown_image_source(
        document: &std::path::Path,
        uri: &gpui_kit::SharedUri,
    ) -> gpui_kit::ImageSource {
        let resolved = url::Url::parse(uri.as_ref()).ok().or_else(|| {
            url::Url::from_file_path(document)
                .ok()?
                .join(uri.as_ref())
                .ok()
        });
        if let Some(path) = resolved.and_then(|url| url.to_file_path().ok()) {
            return path.into();
        }
        uri.clone().into()
    }
}

impl Editor {
    pub(in crate::gui) fn init_markdown_preview(&mut self, file: &ProjectFile) {
        if crate::gui::component::code_editor::language_for_path(file.relative_path()) == "markdown"
        {
            self.state.markdown_extensions.insert(
                file.clone(),
                gpui_kit::base::text::MarkdownExtensions::default().plugin(MarkdownImages {
                    document: file.path(DATA_CONTEXT.project_root()),
                }),
            );
        }
    }
}

pub(in crate::gui::main_window::content_area::editor) fn parse_image_paragraph(
    node: &gpui_kit::base::text::markdown_ast::Node,
    context: &gpui_kit::base::text::MarkdownParseContext<'_>,
) -> Option<gpui_kit::base::text::MarkdownNode> {
    use gpui_kit::base::text::markdown_ast::Node;
    let Node::Paragraph(paragraph) = node else {
        return None;
    };
    let mut images = Vec::<(String, String)>::new();
    for child in &paragraph.children {
        match child {
            Node::Image(image) => images.push((image.url.clone(), image.alt.clone())),
            Node::Text(text) if text.value.trim().is_empty() => {}
            Node::Break(_) => {}
            _ => return None,
        }
    }
    if images.is_empty() {
        return None;
    }
    let label = images
        .iter()
        .map(|(_, alt)| alt.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    Some(
        gpui_kit::base::text::MarkdownNode::new("document-images", images)
            .text(label)
            .markdown(context.node_source(node).unwrap_or_default().to_owned()),
    )
}

impl Editor {
    pub(in crate::gui) fn markdown_image_aspect(
        source: &gpui_kit::ImageSource,
        window: &mut Window,
        cx: &mut App,
    ) -> f32 {
        let image = match source {
            gpui_kit::ImageSource::Resource(resource) => window
                .use_asset::<gpui_kit::ImgResourceLoader>(resource, cx)
                .and_then(Result::ok),
            gpui_kit::ImageSource::Render(image) => Some(image.clone()),
            _ => None,
        };
        image
            .map(|image| image.size(0))
            .filter(|size| size.height.0 > 0)
            .map_or(16. / 9., |size| size.width.0 as f32 / size.height.0 as f32)
    }
}
