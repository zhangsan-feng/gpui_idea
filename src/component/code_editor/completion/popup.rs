use super::CompletionDropdown;
use gpui_kit::{component::input::EditorState, *};

// Resolve caret geometry after the editor's prepaint, without depending on
// gpui-component's private popover implementation.
pub(super) struct CompletionPopup {
    menu: Entity<CompletionDropdown>,
    editor: WeakEntity<EditorState>,
    content: Option<AnyElement>,
}

impl CompletionPopup {
    pub(super) fn new(menu: Entity<CompletionDropdown>, editor: WeakEntity<EditorState>) -> Self {
        Self {
            menu,
            editor,
            content: None,
        }
    }
}

impl Element for CompletionPopup {
    type RequestLayoutState = ();
    type PrepaintState = ();
    fn id(&self) -> Option<ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }
    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        (window.request_layout(Style::default(), [], cx), ())
    }
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        let Some(editor) = self.editor.upgrade() else {
            return;
        };
        let state = editor.read(cx);
        let Some(geometry) = state.prepaint_caret_geometry() else {
            return;
        };
        let caret = geometry.cursor_bounds();
        let viewport = window.viewport_size();
        let width = px(540.).min((viewport.width - px(16.)).max(px(0.)));
        let mut origin = point(
            (caret.left() - px(4.))
                .max(px(8.))
                .min((viewport.width - width - px(8.)).max(px(0.))),
            caret.top() + geometry.scroll_offset().y + geometry.line_height() + px(4.),
        );
        let mut content = self.menu.update(cx, |menu, cx| menu.menu(width, cx));
        let size = content.layout_as_root(
            Size {
                width: AvailableSpace::Definite(width),
                height: AvailableSpace::Definite(viewport.height),
            },
            window,
            cx,
        );
        if origin.y + size.height > viewport.height - px(8.) {
            origin.y =
                (caret.top() + geometry.scroll_offset().y - size.height - px(4.)).max(px(8.));
        }
        content.prepaint_at(origin, window, cx);
        self.content = Some(content);
    }
    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        if let Some(content) = &mut self.content {
            content.paint(window, cx);
        }
    }
}

impl IntoElement for CompletionPopup {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}
