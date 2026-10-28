use super::*;
use gpui_kit::assets::IconName;
use gpui_kit::component::{
    Disableable as _, Icon, Sizable as _,
    button::{Button, ButtonVariants as _},
    checkbox::Checkbox,
    h_flex,
    input::Input,
    v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

impl RunConfigurationWindow {
    pub(in crate::gui) fn render_view(&self, cx: &mut Context<Self>) -> Div {
        let configuration = DATA_CONTEXT.project.run_configuration();
        let busy = configuration.loading || DATA_CONTEXT.project.run_sessions().starting;
        v_flex().size_full().gap_3().text_size(px(13.)).child(
            h_flex().flex_1().min_h(px(350.)).items_stretch().border_1().border_color(crate::component::appearance::border(0xdfe2e7)).rounded(px(4.)).child(
                v_flex().w(px(200.)).flex_shrink_0().justify_start().min_h_0().bg(crate::component::appearance::background(0xf7f8fa)).border_r_1().border_color(crate::component::appearance::border(0xdfe2e7))
                    .child(h_flex().h(px(36.)).flex_shrink_0().px_2().gap_1().items_center()
                        .child(Button::new("add-run-config").ghost().small().icon(IconName::Plus).tooltip("新增配置").disabled(busy).on_click(cx.listener(|this, _, window, cx| this.configuration_action(0, window, cx))))
                        .child(Button::new("copy-run-config").ghost().small().icon(IconName::Copy).tooltip("复制配置").disabled(busy).on_click(cx.listener(|this, _, window, cx| this.configuration_action(1, window, cx))))
                        .child(Button::new("delete-run-config").ghost().small().icon(IconName::Minus).tooltip("删除配置").disabled(busy || configuration.configurations.len() <= 1).on_click(cx.listener(|this, _, window, cx| this.configuration_action(2, window, cx)))))
                    .child(v_flex().id("run-config-list").flex_1().min_h_0().justify_start().max_h(px(350.)).overflow_y_scroll().children(configuration.configurations.iter().map(|saved| {
                        let id = saved.id;
                        h_flex().id(SharedString::from(format!("edit-config-{id}"))).h(px(32.)).px_3().gap_2().items_center().cursor_pointer()
                            .bg(crate::component::appearance::background(if self.editing == id { 0xdce8ff } else { 0xf7f8fa })).hover(|style| style.bg(crate::component::appearance::background(0xe8eef8)))
                            .child(Icon::new(IconName::CirclePlay).small()).child(div().truncate().child(saved.name.clone()))
                            .on_click(cx.listener(move |this, _, window, cx| this.select_configuration(id, window, cx)))
                    })))
            ).child(
                v_flex().flex_1().min_w_0().p_4().gap_3()
                    .child(crate::gui::component::form::field("名称", 96., Input::new(&self.name).into_any_element()))
                    .child(crate::gui::component::form::field("运行命令", 96., Input::new(&self.command).into_any_element()))
                    .child(crate::gui::component::form::field("程序参数", 96., Input::new(&self.arguments).into_any_element()))
                    .child(div().pl(px(108.)).text_size(px(12.)).text_color(crate::component::appearance::foreground(0x737b87)).child("空格分隔参数，带空格的值用引号包裹。"))
                    .child(crate::gui::component::form::field("启动实例数", 96., div().w(px(100.)).child(Input::new(&self.instances)).into_any_element()))
                    .child(div().pl(px(108.)).child(Checkbox::new("allow-parallel-run").checked(self.parallel).label("允许重复启动，保留已有实例")
                        .on_click(cx.listener(|this, checked: &bool, _, cx| { this.parallel = *checked; this.dirty = true; cx.notify(); }))))
                    .child(div().pl(px(108.)).text_size(px(12.)).text_color(crate::component::appearance::foreground(0x737b87)).child("每次启动指定数量的独立实例；关闭重复启动时，会先停止此配置的已有实例。"))
                    .child(div().flex_1())
                    .child(Button::new("redetect-run-config").ghost().small().label("重新识别启动目标").disabled(busy || self.dirty).on_click(cx.listener(|this, _, window, cx| this.redetect(window, cx))))
                    .when_some(self.error.clone().or_else(|| configuration.error.clone()), |panel, error| panel.child(div().text_color(crate::component::appearance::foreground(0xd44352)).child(error)))
            )
        ).child(h_flex().justify_end().gap_2()
            .child(Button::new("cancel-run-config").label("取消").on_click(|_, window, _| window.remove_window()))
            .child(Button::new("apply-run-config").label("应用").disabled(busy).on_click(cx.listener(|this, _, window, cx| this.apply(false, window, cx))))
            .child(Button::new("run-edited-config").primary().label("运行").disabled(busy || self.command.read(cx).value().trim().is_empty()).on_click(cx.listener(|this, _, window, cx| this.apply(true, window, cx)))))
    }
}
