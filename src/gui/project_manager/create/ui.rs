use super::*;
use gpui_kit::{
    assets::IconName,
    component::{
        Disableable as _, Icon, Sizable as _,
        button::{Button, ButtonVariants as _},
        h_flex,
        input::Input,
        v_flex,
    },
    prelude::*,
    *,
};

impl ProjectManager {
    pub(in crate::gui::project_manager) fn render_create(&self, cx: &mut Context<Self>) -> Div {
        let language = self.creation.language;
        let busy = self.busy;
        let probing = self.creation.probing;
        let tool_label = match language {
            ProjectLanguage::Rust => "Toolchain location:",
            ProjectLanguage::Go => "Go SDK:",
            ProjectLanguage::Python => "Python interpreter:",
            _ => "Node.js runtime:",
        };
        let info = self.creation.info.as_ref().filter(|info| {
            info.executable.to_string_lossy() == self.creation.toolchain.read(cx).value().as_str()
        });
        let version = if probing {
            "正在检测…".to_owned()
        } else {
            info.map(|info| info.version.clone())
                .unwrap_or_else(|| "未检测".into())
        };
        let mut sidebar = v_flex()
            .w(px(176.))
            .h_full()
            .flex_shrink_0()
            .p_2()
            .gap_1()
            .bg(rgb(0xffffff))
            .border_r_1()
            .border_color(rgb(0xdfe2e7));
        for (index, item) in ProjectLanguage::ALL.into_iter().enumerate() {
            sidebar = sidebar.child(
                h_flex()
                    .id(("new-project-language", index))
                    .h(px(30.))
                    .px_2()
                    .gap_2()
                    .items_center()
                    .rounded(px(3.))
                    .cursor_pointer()
                    .bg(rgb(if item == language { 0xd4e2ff } else { 0xffffff }))
                    .hover(|style| {
                        style.bg(rgb(if item == language { 0xd4e2ff } else { 0xf0f2f5 }))
                    })
                    .child(Icon::new(IconName::Code).small().text_color(rgb(0x6b778c)))
                    .child(item.label())
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.select_language(item, window, cx)
                    })),
            );
        }
        let mut fields = v_flex().flex_1().min_w_0().p_4().gap_3();
        fields = fields.child(Self::creation_field(
            "Location:",
            h_flex()
                .gap_2()
                .flex_1()
                .min_w_0()
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .child(Input::new(&self.directory).disabled(busy)),
                )
                .child(
                    Button::new("create-location-browse")
                        .icon(IconName::FolderOpen)
                        .tooltip("选择父目录")
                        .disabled(busy)
                        .on_click(
                            cx.listener(|this, _, window, cx| this.choose_directory(window, cx)),
                        ),
                )
                .into_any_element(),
        ));
        fields = fields.child(Self::creation_field(
            tool_label,
            h_flex()
                .gap_2()
                .flex_1()
                .min_w_0()
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .child(Input::new(&self.creation.toolchain).disabled(busy || probing)),
                )
                .child(
                    Button::new("create-toolchain-check")
                        .label("检测")
                        .disabled(busy || probing)
                        .on_click(
                            cx.listener(|this, _, window, cx| this.inspect_toolchain(window, cx)),
                        ),
                )
                .child(
                    Button::new("create-toolchain-browse")
                        .label("…")
                        .tooltip("选择可执行文件")
                        .disabled(busy || probing)
                        .on_click(
                            cx.listener(|this, _, window, cx| this.choose_toolchain(window, cx)),
                        ),
                )
                .into_any_element(),
        ));
        fields = fields.child(Self::creation_field(
            "Toolchain version:",
            div().min_h(px(22.)).child(version).into_any_element(),
        ));
        if language == ProjectLanguage::Rust {
            let standard = info
                .and_then(|info| info.standard_library.as_ref())
                .map(|path| path.to_string_lossy().into_owned())
                .unwrap_or_else(|| "未安装 rust-src（不影响创建项目）".into());
            fields = fields.child(Self::creation_field(
                "Standard library:",
                div()
                    .min_w_0()
                    .truncate()
                    .text_color(rgb(0x687384))
                    .child(standard)
                    .into_any_element(),
            ));
        }
        if language == ProjectLanguage::Go {
            fields = fields.child(Self::creation_field(
                "Module path:",
                Input::new(&self.creation.module)
                    .disabled(busy)
                    .into_any_element(),
            ));
        }
        if matches!(language, ProjectLanguage::Rust | ProjectLanguage::Go) {
            fields = fields.child(Self::creation_field(
                "Environment:",
                Input::new(&self.creation.environment)
                    .disabled(busy)
                    .into_any_element(),
            ));
        }
        fields = fields.child(
            h_flex()
                .mt_4()
                .gap_2()
                .items_center()
                .child("Project Template")
                .child(div().flex_1().h(px(1.)).bg(rgb(0xe0e3e8))),
        );
        let mut templates = v_flex()
            .min_h(px(150.))
            .border_1()
            .border_color(rgb(0xe0e3e8));
        for library in [false, true]
            .into_iter()
            .take(if language == ProjectLanguage::Rust {
                2
            } else {
                1
            })
        {
            templates = templates.child(
                h_flex()
                    .id(if library {
                        "create-library"
                    } else {
                        "create-application"
                    })
                    .h(px(30.))
                    .px_2()
                    .gap_2()
                    .items_center()
                    .cursor_pointer()
                    .bg(rgb(if library == self.creation.library {
                        0xd4e2ff
                    } else {
                        0xf7f8fa
                    }))
                    .hover(|style| style.bg(rgb(0xe8eefb)))
                    .child(
                        Icon::new(IconName::FileBox)
                            .small()
                            .text_color(rgb(0x778397)),
                    )
                    .child(language.template(library))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if !this.busy {
                            this.creation.library = library;
                            cx.notify();
                        }
                    })),
            );
        }
        let hint = match language {
            ProjectLanguage::Rust => "使用 cargo init 创建独立 Cargo 项目。",
            ProjectLanguage::Go => "使用 go mod init 初始化模块，并生成 main.go。",
            ProjectLanguage::Python => "生成 main.py、pyproject.toml 和 .gitignore。",
            ProjectLanguage::JavaScript => "生成 ES Modules 项目，可直接 npm start。",
            ProjectLanguage::TypeScript => {
                "生成 package.json、tsconfig.json 和入口；创建后 npm install 安装 TypeScript。"
            }
        };
        fields = fields
            .child(templates)
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(rgb(0x737b87))
                    .child(hint),
            )
            .when_some(self.error.clone(), |view, error| {
                view.child(
                    div()
                        .text_size(px(12.))
                        .text_color(rgb(0xc54343))
                        .child(error),
                )
            });
        v_flex()
            .size_full()
            .bg(rgb(0xf7f8fa))
            .text_color(rgb(0x202124))
            .text_size(px(13.))
            .child(h_flex().flex_1().min_h_0().child(sidebar).child(fields))
            .child(
                h_flex()
                    .h(px(58.))
                    .flex_shrink_0()
                    .px_4()
                    .justify_end()
                    .items_center()
                    .gap_2()
                    .border_t_1()
                    .border_color(rgb(0xdfe2e7))
                    .child(
                        Button::new("cancel-create-project")
                            .label("取消")
                            .disabled(busy)
                            .on_click(|_, window, _| window.remove_window()),
                    )
                    .child(
                        Button::new("create-language-project")
                            .primary()
                            .label(if busy { "正在初始化…" } else { "Create" })
                            .disabled(
                                busy || probing
                                    || self.directory.read(cx).value().trim().is_empty(),
                            )
                            .on_click(cx.listener(|this, _, window, cx| this.submit(window, cx))),
                    ),
            )
    }

    fn creation_field(label: &'static str, field: AnyElement) -> Div {
        h_flex()
            .min_h(px(28.))
            .gap_3()
            .items_center()
            .child(div().w(px(126.)).flex_shrink_0().child(label))
            .child(div().flex_1().min_w_0().child(field))
    }
}
