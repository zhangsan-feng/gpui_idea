use crate::gui::component::measured_element::RenderTimings;
use gpui_kit::*;
use std::time::Instant;

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

    pub(in crate::gui::workspace) fn record(
        &mut self,
        area: &'static str,
        times: RenderTimings,
        window: &Window,
        cx: &App,
    ) {
        let times = [
            times.build_us,
            times.layout_us,
            times.prepaint_us,
            times.paint_us,
            times.cpu_us,
        ];
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
        tracing::debug!(
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
