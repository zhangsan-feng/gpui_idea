use super::*;
use crate::application::{APPLICATION, lifecycle::run_application};
use gpui_kit::PathPromptOptions;

impl ProjectManagerWindow {
    pub(in crate::gui::project_manager_window) fn select_language(
        &mut self,
        language: ProjectLanguage,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.busy || self.creation.language == language {
            return;
        }
        self.creation.language = language;
        self.creation.library = false;
        self.creation.info = None;
        self.creation
            .toolchain
            .update(cx, |state, cx| state.set_value("", window, cx));
        self.error = None;
        self.inspect_toolchain(window, cx);
    }

    pub(in crate::gui::project_manager_window) fn inspect_toolchain(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.busy {
            return;
        }
        self.creation.generation = self.creation.generation.wrapping_add(1);
        let generation = self.creation.generation;
        let language = self.creation.language;
        let executable = self.creation.toolchain.read(cx).value().to_string();
        self.creation.probing = true;
        self.creation.info = None;
        self.error = None;
        cx.notify();
        cx.spawn_in(window, async move |this, cx| {
            let result = run_application(async move {
                APPLICATION.projects.toolchain(language, executable).await
            })
            .await;
            let _ = this.update_in(cx, |this, window, cx| {
                if this.creation.generation != generation {
                    return;
                }
                this.creation.probing = false;
                match result {
                    Ok(info) => {
                        this.creation.toolchain.update(cx, |state, cx| {
                            state.set_value(
                                info.executable.to_string_lossy().into_owned(),
                                window,
                                cx,
                            )
                        });
                        this.creation.info = Some(info);
                    }
                    Err(error) => this.error = Some(format!("{error:#}")),
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(in crate::gui::project_manager_window) fn choose_toolchain(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let prompt = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("选择工具链可执行文件".into()),
        });
        let generation = self.creation.generation;
        cx.spawn_in(window, async move |this, cx| {
            if let Ok(Ok(Some(paths))) = prompt.await {
                if let Some(path) = paths.into_iter().next() {
                    let _ = this.update_in(cx, |this, window, cx| {
                        if this.creation.generation != generation || this.busy {
                            return;
                        }
                        this.creation.toolchain.update(cx, |state, cx| {
                            state.set_value(path.to_string_lossy().into_owned(), window, cx)
                        });
                        this.inspect_toolchain(window, cx);
                    });
                }
            }
        })
        .detach();
    }
}
