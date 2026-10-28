use super::*;
use crate::{
    application::APPLICATION,
    domain::run::{RunAction, RunConfigurationEdit},
};

impl RunConfigurationWindow {
    pub(in crate::gui) fn redetect(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let id = self.editing;
        let file = DATA_CONTEXT
            .project
            .run_configuration()
            .selected()
            .and_then(|saved| saved.file.clone());
        cx.spawn_in(window, async move |this, cx| {
            let result = crate::application::lifecycle::run_application(async move {
                APPLICATION
                    .project
                    .refresh_run_configuration(id, file)
                    .await
            })
            .await;
            let _ = this.update_in(cx, |this, _, cx| {
                if this.editing == id {
                    if let Err(error) = result {
                        this.error = Some(format!("{error:#}"));
                    }
                    cx.notify();
                }
            });
        })
        .detach();
    }
    pub(in crate::gui) fn load_selected(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let configuration = DATA_CONTEXT.project.run_configuration();
        let Some(saved) = configuration.selected() else {
            return;
        };
        self.editing = saved.id;
        self.loaded_request = configuration.request;
        self.was_loading = configuration.loading;
        self.dirty = false;
        self.command_dirty = false;
        self.parallel = saved.allow_parallel;
        for (input, value) in [
            (&self.name, saved.name.clone()),
            (&self.command, saved.command.clone()),
            (&self.arguments, saved.arguments.clone()),
            (&self.instances, saved.instances.to_string()),
        ] {
            input.update(cx, |state, cx| state.set_value(value, window, cx));
        }
        self.error = None;
    }

    pub(in crate::gui) fn configuration_action(
        &mut self,
        action: u8,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let result = match action {
            0 => APPLICATION.project.add_run_configuration(false),
            1 => APPLICATION.project.add_run_configuration(true),
            _ => APPLICATION.project.delete_run_configuration(self.editing),
        };
        match result {
            Ok(()) => self.load_selected(window, cx),
            Err(error) => self.error = Some(format!("{error:#}")),
        }
        cx.notify();
    }

    pub(in crate::gui) fn select_configuration(
        &mut self,
        id: u64,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match APPLICATION.project.select_run_configuration(id) {
            Ok(()) => self.load_selected(window, cx),
            Err(error) => self.error = Some(format!("{error:#}")),
        }
        cx.notify();
    }

    pub(in crate::gui) fn apply(&mut self, run: bool, window: &mut Window, cx: &mut Context<Self>) {
        let edit = RunConfigurationEdit {
            id: self.editing,
            name: self.name.read(cx).value().to_string(),
            command: self.command.read(cx).value().to_string(),
            arguments: self.arguments.read(cx).value().to_string(),
            instances: self.instances.read(cx).value().to_string(),
            allow_parallel: self.parallel,
        };
        if let Err(error) = APPLICATION.project.save_run_configuration(edit) {
            self.error = Some(format!("{error:#}"));
            cx.notify();
            return;
        }
        self.error = None;
        if run {
            let parent = self.parent.clone();
            let result = self.parent_window.update(cx, |_, _main_window, cx| {
                parent.update(cx, |_parent, cx| {
                    crate::event_bus::EVENT_BUS.gui_event.send(
                        cx.entity().entity_id(),
                        crate::event_bus::gui_event::GuiEvent::BottomPanel(
                            crate::gui::workspace::bottom_panel::Event::StartExecution(
                                RunAction::Run,
                            ),
                        ),
                    )
                })
            });
            if result.is_ok_and(|result| result.is_ok()) {
                window.remove_window();
            } else {
                self.error = Some("项目工作区已关闭，无法启动运行".into());
            }
        } else {
            self.load_selected(window, cx);
        }
        cx.notify();
    }
}
