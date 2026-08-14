// Minimal repro: a Splash widget INSIDE a PortalList item template, driven the
// way robrix2's timeline drives its splash cards — the item is created during
// draw via `list.item(..)` and the splash body is injected with `set_text`
// from the draw pass. The plain `isolate` example (Splash as a static child)
// renders fine; this exercises the template/recycle path.
pub use makepad_widgets;

use makepad_widgets::*;

app_main!(App);

const SPLASH_BODY: &str = r#"
width: Fill
height: Fit
flow: Down
spacing: 8
padding: 10
draw_bg +: { color: #1F6F7A }

title := Label { text: "SPLASH IN PORTAL LIST" }
status := Label { text: "NOT CLICKED" }
row := View {
    width: Fill
    height: Fit
    flow: Right
    spacing: 8
    approve := Button {
        text: "Approve"
        on_click: || { ui.status.set_text("CLICKED approve") }
    }
    reject := Button {
        text: "Reject"
        on_click: || { ui.status.set_text("CLICKED reject") }
    }
}
"#;

script_mod! {
    use mod.prelude.widgets.*

    startup() do #(App::script_component(vm)){
        ui: Root{
            main_window := Window{
                window.inner_size: vec2(640, 480)
                body +: {
                    cards := #(CardList::register_widget(vm)) {
                        width: Fill
                        height: Fill
                        // robrix2's timeline gives the message list its own GPU
                        // draw list (dsl.rs "new_batch: true"); the splash cards
                        // render inside it.
                        new_batch: true

                        list := PortalList {
                            width: Fill
                            height: Fill

                            Card := View {
                                width: Fill
                                height: Fit
                                flow: Down
                                padding: 8
                                spacing: 4

                                card_label := Label { text: "plain item" }
                                splash_card := Splash { width: Fill height: Fit }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct CardList {
    #[deref]
    view: View,
}

impl Widget for CardList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        while let Some(item) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = item.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, 3);
                while let Some(item_id) = list.next_visible_item(cx) {
                    let item_widget = list.item(cx, item_id, id!(Card));
                    item_widget
                        .label(cx, ids!(card_label))
                        .set_text(cx, &format!("item {item_id}"));
                    // item 1 carries the splash card, like a bot message
                    let splash = item_widget.splash(cx, ids!(splash_card));
                    if item_id == 1 {
                        splash.set_visible(cx, true);
                        splash.set_text(cx, SPLASH_BODY);
                    } else {
                        splash.set_visible(cx, false);
                    }
                    item_widget.draw_all_unscoped(cx);
                }
            }
        }
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }
}

#[derive(Script, ScriptHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
}

impl MatchEvent for App {}

impl AppMain for App {
    fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
        crate::makepad_widgets::script_mod(vm);
        self::script_mod(vm)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        // robrix2 requests a full script reapply on preference/language changes
        // (app_preferences.rs). Mimic that on mouse-down so the driver's click
        // separates a pre-reapply frame from a post-reapply frame.
        if let Event::MouseDown(_) = event {
            log!("[REPRO] request_script_reapply()");
            cx.request_script_reapply();
        }
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}
