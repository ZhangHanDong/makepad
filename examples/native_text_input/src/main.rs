pub use makepad_widgets;

use makepad_widgets::*;

app_main!(App);

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    let SmokeButton = Button{
        width: Fit
        height: 32
        padding: theme.mspace_1{left: theme.space_2, right: theme.space_2}
        draw_bg +: {
            color: #xdbeafe
            color_hover: #xbfdbfe
            color_down: #x2563eb
            color_focus: #x2563eb
            border_color: #x93c5fd
            border_color_hover: #x60a5fa
            border_color_down: #x1d4ed8
            border_color_focus: #x1d4ed8
            border_radius: 5.0
        }
        draw_text +: {
            color: #x0f172a
            color_hover: #x0f172a
            color_down: #xffffff
            color_focus: #xffffff
            text_style: theme.font_bold{font_size: theme.font_size_p}
        }
    }

    let app = startup() do #(App::script_component(vm)){
        ui: Root{
            main_window := Window{
                pass.clear_color: #xf8fafc
                window.inner_size: vec2(820, 720)
                body +: {
                    width: Fill height: Fill
                    flow: Down spacing: 0

                    ScrollYView{
                        width: Fill height: Fill
                        flow: Down spacing: 14
                        padding: 24

                        View{
                            width: Fill height: Fit
                            flow: Down spacing: 4
                            Label{
                                text: "NativeTextInput macOS MVP"
                                draw_text.color: #x0f172a
                                draw_text.text_style: theme.font_bold{font_size: theme.font_size_2}
                            }
                            Label{
                                text: "Studio RunItem smoke surface"
                                draw_text.color: #x475569
                            }
                        }

                        RoundedView{
                            width: Fill height: Fit
                            flow: Down spacing: 10
                            padding: 14
                            draw_bg.color: #xffffff
                            draw_bg.border_color: #xe2e8f0
                            draw_bg.border_size: 1.0
                            draw_bg.border_radius: 8.0

                            Label{
                                text: "Native controls"
                                draw_text.color: #x0f172a
                                draw_text.text_style: theme.font_bold{font_size: theme.font_size_4}
                            }

                            native_label := NativeLabel{
                                width: Fill
                                height: 28
                                text: "NativeLabel shares the macOS host registry"
                                draw_bg +: {color: #xf8fafc}
                            }

                            Label{text: "Primary native input" draw_text.color: #x475569}
                            native_input := NativeTextInput{
                                width: Fill
                                height: 36
                                placeholder: "Type here"
                                draw_bg +: {color: #xdbeafe}
                            }

                            Label{text: "Secondary native input" draw_text.color: #x475569}
                            secondary_native_input := NativeTextInput{
                                width: Fill
                                height: 36
                                placeholder: "Secondary native field"
                                draw_bg +: {color: #xdbeafe}
                            }

                            View{
                                width: Fill height: Fit
                                flow: Flow.Right{wrap: true} spacing: 8

                                focus_button := SmokeButton{text: "Primary Focus"}
                                set_button := SmokeButton{text: "Primary Set"}
                                blur_button := SmokeButton{text: "Primary Blur"}
                                select_button := SmokeButton{text: "Primary Select"}
                                copy_button := SmokeButton{text: "Primary Copy"}
                                cut_button := SmokeButton{text: "Primary Cut"}
                                paste_button := SmokeButton{text: "Primary Paste"}
                            }

                            View{
                                width: Fill height: Fit
                                flow: Flow.Right{wrap: true} spacing: 8

                                secondary_focus_button := SmokeButton{text: "Secondary Focus"}
                                secondary_set_button := SmokeButton{text: "Secondary Set"}
                                secondary_blur_button := SmokeButton{text: "Secondary Blur"}
                                secondary_select_button := SmokeButton{text: "Secondary Select"}
                                secondary_copy_button := SmokeButton{text: "Secondary Copy"}
                                secondary_cut_button := SmokeButton{text: "Secondary Cut"}
                                secondary_paste_button := SmokeButton{text: "Secondary Paste"}
                                label_button := SmokeButton{text: "Set Label"}
                                set_both_button := SmokeButton{text: "Set Both"}
                            }
                        }

                        RoundedView{
                            width: Fill height: Fit
                            flow: Down spacing: 8
                            padding: 14
                            draw_bg.color: #xffffff
                            draw_bg.border_color: #xe2e8f0
                            draw_bg.border_size: 1.0
                            draw_bg.border_radius: 8.0

                            Label{
                                text: "Event state"
                                draw_text.color: #x0f172a
                                draw_text.text_style: theme.font_bold{font_size: theme.font_size_4}
                            }
                            status_label := Label{text: "Status: ready" draw_text.color: #x334155}
                            value_label := Label{text: "Value: " draw_text.color: #x334155}
                            focus_state_label := Label{text: "Focus: idle" draw_text.color: #x334155}
                            selection_state_label := Label{text: "Selection: idle" draw_text.color: #x334155}
                            command_state_label := Label{text: "Commands: 0" draw_text.color: #x334155}
                            change_state_label := Label{text: "Changed events: 0" draw_text.color: #x334155}
                            label_state_label := Label{text: "NativeLabel: initial" draw_text.color: #x334155}
                            host_state_label := Label{text: "Host: attached" draw_text.color: #x334155}
                        }

                        RoundedView{
                            width: Fill height: Fit
                            flow: Down spacing: 10
                            padding: 14
                            draw_bg.color: #xffffff
                            draw_bg.border_color: #xe2e8f0
                            draw_bg.border_size: 1.0
                            draw_bg.border_radius: 8.0

                            Label{
                                text: "Composition and clipping"
                                draw_text.color: #x0f172a
                                draw_text.text_style: theme.font_bold{font_size: theme.font_size_4}
                            }

                            ScrollYView{
                                width: Fill
                                height: 128
                                flow: Down
                                spacing: 8
                                padding: 8

                                Label{text: "Clipped native host area" draw_text.color: #x475569}
                                clipped_native_input := NativeTextInput{
                                    width: Fill
                                    height: 36
                                    placeholder: "Scroll/clipped host"
                                    draw_bg +: {color: #xdbeafe}
                                }
                                Label{text: "The native field is mounted inside the shared host view" draw_text.color: #x64748b}
                                Label{text: "The screenshot evidence should show the input inside this scroller" draw_text.color: #x64748b}
                            }
                        }
                    }
                }
            }
        }
    }
    app
}

#[derive(Script, ScriptHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
    #[rust]
    command_count: usize,
    #[rust]
    change_count: usize,
    #[rust]
    focus_count: usize,
    #[rust]
    selection_count: usize,
    #[rust]
    label_count: usize,
    #[rust]
    diag_frame_count: usize,
    #[rust]
    diag_closed: bool,
    #[rust]
    diag_quit_requested: bool,
}

impl App {
    fn set_label(&mut self, cx: &mut Cx, id: &[LiveId], text: &str) {
        self.ui.label(cx, id).set_text(cx, text);
    }

    fn record_command(&mut self, cx: &mut Cx, status: &str) {
        self.command_count += 1;
        self.set_label(cx, ids!(status_label), status);
        self.set_label(
            cx,
            ids!(command_state_label),
            &format!("Commands: {}", self.command_count),
        );
    }
}

impl MatchEvent for App {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        let input = self.ui.native_text_input(cx, ids!(native_input));
        let secondary_input = self.ui.native_text_input(cx, ids!(secondary_native_input));
        let native_label = self.ui.native_label(cx, ids!(native_label));

        if self.ui.button(cx, ids!(focus_button)).clicked(actions) {
            input.focus(cx);
            self.record_command(cx, "Status: focus requested");
            self.set_label(cx, ids!(focus_state_label), "Focus: requested");
            log!("NativeTextInput smoke: focus requested");
        }

        if self.ui.button(cx, ids!(set_button)).clicked(actions) {
            input.set_text(cx, "hello native");
            self.record_command(cx, "Status: primary text set");
            self.set_label(cx, ids!(value_label), "Value: hello native");
            log!("NativeTextInput smoke: primary text set");
        }

        if self.ui.button(cx, ids!(blur_button)).clicked(actions) {
            input.blur(cx);
            self.record_command(cx, "Status: blur requested");
            self.set_label(cx, ids!(focus_state_label), "Focus: blur requested");
            log!("NativeTextInput smoke: blur requested");
        }

        if self.ui.button(cx, ids!(select_button)).clicked(actions) {
            input.select_all(cx);
            self.record_command(cx, "Status: primary select all requested");
            self.set_label(
                cx,
                ids!(selection_state_label),
                "Selection: primary select all requested",
            );
            log!("NativeTextInput smoke: primary select all requested");
        }

        if self
            .ui
            .button(cx, ids!(secondary_focus_button))
            .clicked(actions)
        {
            secondary_input.focus(cx);
            self.record_command(cx, "Status: secondary focus requested");
            self.set_label(cx, ids!(focus_state_label), "Focus: secondary requested");
            log!("NativeTextInput smoke: secondary focus requested");
        }

        if self
            .ui
            .button(cx, ids!(secondary_set_button))
            .clicked(actions)
        {
            secondary_input.set_text(cx, "secondary hello native");
            self.record_command(cx, "Status: secondary text set");
            self.set_label(
                cx,
                ids!(value_label),
                "Secondary value: secondary hello native",
            );
            log!("NativeTextInput smoke: secondary text set");
        }

        if self
            .ui
            .button(cx, ids!(secondary_blur_button))
            .clicked(actions)
        {
            secondary_input.blur(cx);
            self.record_command(cx, "Status: secondary blur requested");
            self.set_label(
                cx,
                ids!(focus_state_label),
                "Focus: secondary blur requested",
            );
            log!("NativeTextInput smoke: secondary blur requested");
        }

        if self
            .ui
            .button(cx, ids!(secondary_select_button))
            .clicked(actions)
        {
            secondary_input.select_all(cx);
            self.record_command(cx, "Status: secondary select all requested");
            self.set_label(
                cx,
                ids!(selection_state_label),
                "Selection: secondary select all requested",
            );
            log!("NativeTextInput smoke: secondary select all requested");
        }

        if self.ui.button(cx, ids!(label_button)).clicked(actions) {
            self.label_count += 1;
            let label_text = format!("NativeLabel update #{}", self.label_count);
            native_label.set_text(cx, &label_text);
            self.record_command(cx, "Status: native label updated");
            self.set_label(
                cx,
                ids!(label_state_label),
                &format!("NativeLabel: update #{}", self.label_count),
            );
            log!(
                "NativeTextInput smoke: native label updated #{}",
                self.label_count
            );
        }

        if self.ui.button(cx, ids!(set_both_button)).clicked(actions) {
            input.set_text(cx, "same frame native input");
            secondary_input.set_text(cx, "same frame secondary input");
            native_label.set_text(cx, "NativeLabel updated in same frame");
            self.record_command(cx, "Status: same-frame native updates requested");
            self.set_label(
                cx,
                ids!(value_label),
                "Value: primary + secondary same-frame update",
            );
            self.set_label(
                cx,
                ids!(label_state_label),
                "NativeLabel: same-frame update",
            );
            self.set_label(
                cx,
                ids!(host_state_label),
                "Host: same-frame input + label update",
            );
            log!("NativeTextInput smoke: same-frame native updates requested");
        }

        if self.ui.button(cx, ids!(copy_button)).clicked(actions) {
            input.copy(cx);
            self.record_command(cx, "Status: primary copy requested");
            log!("NativeTextInput smoke: primary copy requested");
        }

        if self.ui.button(cx, ids!(cut_button)).clicked(actions) {
            input.cut(cx);
            self.record_command(cx, "Status: primary cut requested");
            log!("NativeTextInput smoke: primary cut requested");
        }

        if self.ui.button(cx, ids!(paste_button)).clicked(actions) {
            input.paste(cx);
            self.record_command(cx, "Status: primary paste requested");
            log!("NativeTextInput smoke: primary paste requested");
        }

        if self
            .ui
            .button(cx, ids!(secondary_copy_button))
            .clicked(actions)
        {
            secondary_input.copy(cx);
            self.record_command(cx, "Status: secondary copy requested");
            log!("NativeTextInput smoke: secondary copy requested");
        }

        if self
            .ui
            .button(cx, ids!(secondary_cut_button))
            .clicked(actions)
        {
            secondary_input.cut(cx);
            self.record_command(cx, "Status: secondary cut requested");
            log!("NativeTextInput smoke: secondary cut requested");
        }

        if self
            .ui
            .button(cx, ids!(secondary_paste_button))
            .clicked(actions)
        {
            secondary_input.paste(cx);
            self.record_command(cx, "Status: secondary paste requested");
            log!("NativeTextInput smoke: secondary paste requested");
        }

        if let Some(text) = input.changed(actions) {
            self.change_count += 1;
            self.set_label(cx, ids!(value_label), &format!("Value: {}", text));
            self.set_label(cx, ids!(status_label), "Status: changed action received");
            self.set_label(
                cx,
                ids!(change_state_label),
                &format!("Changed events: {}", self.change_count),
            );
            log!("NativeTextInput smoke: changed action received: {}", text);
        }

        if let Some(text) = secondary_input.changed(actions) {
            self.change_count += 1;
            self.set_label(cx, ids!(value_label), &format!("Secondary value: {}", text));
            self.set_label(
                cx,
                ids!(status_label),
                "Status: secondary changed action received",
            );
            self.set_label(
                cx,
                ids!(change_state_label),
                &format!("Changed events: {}", self.change_count),
            );
            log!(
                "NativeTextInput smoke: secondary changed action received: {}",
                text
            );
        }

        if input.key_focus(actions) {
            self.focus_count += 1;
            self.set_label(cx, ids!(status_label), "Status: focus action received");
            self.set_label(
                cx,
                ids!(focus_state_label),
                &format!("Focus: active ({})", self.focus_count),
            );
            log!("NativeTextInput smoke: focus action received");
        }

        if secondary_input.key_focus(actions) {
            self.focus_count += 1;
            self.set_label(
                cx,
                ids!(status_label),
                "Status: secondary focus action received",
            );
            self.set_label(
                cx,
                ids!(focus_state_label),
                &format!("Focus: secondary active ({})", self.focus_count),
            );
            log!("NativeTextInput smoke: secondary focus action received");
        }

        if input.key_focus_lost(actions) {
            self.set_label(cx, ids!(status_label), "Status: blur action received");
            self.set_label(cx, ids!(focus_state_label), "Focus: inactive");
            log!("NativeTextInput smoke: blur action received");
        }

        if secondary_input.key_focus_lost(actions) {
            self.set_label(
                cx,
                ids!(status_label),
                "Status: secondary blur action received",
            );
            self.set_label(cx, ids!(focus_state_label), "Focus: inactive");
            log!("NativeTextInput smoke: secondary blur action received");
        }

        if let Some((start, end)) = input.selection_changed(actions) {
            self.selection_count += 1;
            self.set_label(
                cx,
                ids!(status_label),
                &format!("Status: selection {}..{}", start, end),
            );
            self.set_label(
                cx,
                ids!(selection_state_label),
                &format!("Selection: {}..{} ({})", start, end, self.selection_count),
            );
            log!("NativeTextInput smoke: selection {}..{}", start, end);
        }

        if let Some((start, end)) = secondary_input.selection_changed(actions) {
            self.selection_count += 1;
            self.set_label(
                cx,
                ids!(status_label),
                &format!("Status: secondary selection {}..{}", start, end),
            );
            self.set_label(
                cx,
                ids!(selection_state_label),
                &format!(
                    "Selection: secondary {}..{} ({})",
                    start, end, self.selection_count
                ),
            );
            log!(
                "NativeTextInput smoke: secondary selection {}..{}",
                start,
                end
            );
        }
    }
}

impl AppMain for App {
    fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
        crate::makepad_widgets::script_mod(vm);
        self::script_mod(vm)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        #[cfg(feature = "diag-native-leak-count")]
        {
            if matches!(event, Event::Startup) {
                self.diag_frame_count = 0;
                self.diag_closed = false;
                self.diag_quit_requested = false;
                cx.new_next_frame();
            }
            if matches!(event, Event::NextFrame(_)) && !self.diag_quit_requested {
                self.diag_frame_count += 1;
                if self.diag_frame_count < 2 {
                    cx.new_next_frame();
                } else if !self.diag_closed {
                    self.ui.native_text_input(cx, ids!(native_input)).close(cx);
                    self.ui
                        .native_text_input(cx, ids!(secondary_native_input))
                        .close(cx);
                    self.ui
                        .native_text_input(cx, ids!(clipped_native_input))
                        .close(cx);
                    self.ui.native_label(cx, ids!(native_label)).close(cx);
                    self.diag_closed = true;
                    log!("NativeTextInput smoke: diagnostic close requested");
                    cx.new_next_frame();
                } else {
                    self.diag_quit_requested = true;
                    log!("NativeTextInput smoke: diagnostic quit requested");
                    cx.quit();
                }
            }
        }
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}
