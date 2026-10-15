use gpui_kit::*;
use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};

/// CPU diagnostics only: painting here builds the scene; GPU submission/present
/// happens later in GPUI and must not be attributed to these timings.
#[derive(Default)]
pub(in crate::gui::workspace) struct WindowDiagnostics {
    last_bounds: Option<Bounds<Pixels>>,
    report_at: Option<Instant>,
    moves: u32,
    resizes: u32,
    frames: u32,
    slow_frames: u32,
    max_us: [u128; 5],
    editor_max_us: [u128; 5],
    tree_max_us: [u128; 5],
}

impl WindowDiagnostics {
    pub(in crate::gui::workspace) fn bounds_changed(&mut self, window: &Window) -> bool {
        let bounds = window.bounds();
        let changed = self.last_bounds.is_some_and(|previous| previous != bounds);
        if let Some(previous) = self.last_bounds {
            if previous == bounds {
                return false;
            }
            if previous.size != bounds.size {
                self.resizes += 1;
            } else {
                self.moves += 1;
            }
            self.report_at.get_or_insert_with(Instant::now);
        }
        self.last_bounds = Some(bounds);
        changed
    }

    fn record(&mut self, area: &'static str, times: [u128; 5], window: &Window, cx: &App) {
        if self.report_at.is_none() {
            return;
        }
        if area != "workspace" {
            let maximum = if area == "editor" {
                &mut self.editor_max_us
            } else {
                &mut self.tree_max_us
            };
            for (maximum, sample) in maximum.iter_mut().zip(times) {
                *maximum = (*maximum).max(sample);
            }
            return;
        }
        self.frames += 1;
        let budget_us = window
            .refresh_interval(cx)
            .map_or(16_667, |v| v.as_micros());
        self.slow_frames += u32::from(times[4] > budget_us);
        for (maximum, sample) in self.max_us.iter_mut().zip(times) {
            *maximum = (*maximum).max(sample);
        }
    }

    // Report independently of paint: a pure move may produce zero rendered frames.
    pub(in crate::gui::workspace) fn report(&mut self, window: &Window, cx: &App) {
        let Some(report_at) = self.report_at.take() else {
            return;
        };
        let budget_us = window
            .refresh_interval(cx)
            .map_or(16_667, |v| v.as_micros());
        tracing::info!(
            sample_ms = report_at.elapsed().as_millis(),
            moves = self.moves,
            resizes = self.resizes,
            frames = self.frames,
            slow_frames = self.slow_frames,
            budget_us,
            build_max_us = self.max_us[0],
            layout_max_us = self.max_us[1],
            prepaint_max_us = self.max_us[2],
            paint_max_us = self.max_us[3],
            cpu_max_us = self.max_us[4],
            editor_layout_interval_max_us = self.editor_max_us[1],
            editor_prepaint_max_us = self.editor_max_us[2],
            editor_paint_max_us = self.editor_max_us[3],
            tree_layout_interval_max_us = self.tree_max_us[1],
            tree_prepaint_max_us = self.tree_max_us[2],
            tree_paint_max_us = self.tree_max_us[3],
            "window movement CPU diagnostics (excludes GPU/present)"
        );
        self.moves = 0;
        self.resizes = 0;
        self.frames = 0;
        self.slow_frames = 0;
        self.max_us = [0; 5];
        self.editor_max_us = [0; 5];
        self.tree_max_us = [0; 5];
    }
}

impl super::super::Workspace {
    pub(in crate::gui::workspace) fn record_window_bounds(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.window_diagnostics.borrow_mut().bounds_changed(window)
            || self.window_diagnostics_task.is_some()
        {
            return;
        }
        let timer = cx.background_executor().timer(Duration::from_secs(1));
        self.window_diagnostics_task = Some(cx.spawn_in(window, async move |this, cx| {
            timer.await;
            let _ = this.update_in(cx, |this, window, cx| {
                if let Some(task) = this.window_diagnostics_task.take() {
                    task.detach();
                }
                this.window_diagnostics.borrow_mut().report(window, cx);
            });
        }));
    }
}

pub(in crate::gui::workspace) struct MeasuredWorkspace {
    area: &'static str,
    child: AnyElement,
    diagnostics: Rc<RefCell<WindowDiagnostics>>,
    started: Instant,
    build_us: u128,
    layout_started: Option<Instant>,
    layout_us: u128,
    prepaint_us: u128,
}

impl MeasuredWorkspace {
    pub(in crate::gui::workspace) fn new(
        area: &'static str,
        child: AnyElement,
        diagnostics: Rc<RefCell<WindowDiagnostics>>,
        started: Instant,
    ) -> Self {
        Self {
            area,
            child,
            diagnostics,
            started,
            build_us: started.elapsed().as_micros(),
            layout_started: None,
            layout_us: 0,
            prepaint_us: 0,
        }
    }
}

impl IntoElement for MeasuredWorkspace {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl Element for MeasuredWorkspace {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        self.layout_started = Some(Instant::now());
        (self.child.request_layout(window, cx), ())
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
        // Includes layout requests and the intervening Taffy computation.
        // Subtree intervals also include work on siblings before their prepaint;
        // only prepaint/paint are isolated subtree timings.
        self.layout_us = self.layout_started.map_or(0, |at| at.elapsed().as_micros());
        let started = Instant::now();
        self.child.prepaint(window, cx);
        self.prepaint_us = started.elapsed().as_micros();
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
        let started = Instant::now();
        self.child.paint(window, cx);
        let paint_us = started.elapsed().as_micros();
        self.diagnostics.borrow_mut().record(
            self.area,
            [
                self.build_us,
                self.layout_us,
                self.prepaint_us,
                paint_us,
                self.started.elapsed().as_micros(),
            ],
            window,
            cx,
        );
    }
}
