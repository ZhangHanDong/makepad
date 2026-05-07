use crate::{makepad_derive_widget::*, makepad_draw::*, view::View, widget::*};

#[derive(Clone, Debug, Default)]
pub enum SplashAction {
    Notify {
        event_id: String,
        payload: String,
    },
    #[default]
    None,
}

pub fn register_agent_module(vm: &mut ScriptVm) {
    let agent = vm.new_module(id!(agent));
    vm.add_method(
        agent,
        id_lut!(notify),
        script_args_def!(event = NIL, payload = NIL),
        |vm, args| {
            let event_value = script_value!(vm, args.event);
            let payload_value = script_value!(vm, args.payload);

            let mut event_id = String::new();
            vm.bx.heap.cast_to_string(event_value, &mut event_id);

            let mut payload = String::new();
            vm.bx.heap.to_json_inner(payload_value, &mut payload);

            Cx::post_action(SplashAction::Notify { event_id, payload });
            NIL
        },
    );
    vm.set_injected_global(id!(agent), agent.into());
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*

    mod.widgets.SplashBase = #(Splash::register_widget(vm))

    mod.widgets.Splash = set_type_default() do mod.widgets.SplashBase{
        width: Fill height: Fit
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct Splash {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    pub view: View,
    #[live]
    body: ArcStringMut,
    #[rust]
    eval_generation: u64,
    #[rust]
    tick_timer: Timer,
    /// The unique_id used for the last full eval, so tick() runs in the same scope.
    #[rust]
    last_unique_id: usize,
}

/// Prefix for View-children mode: wraps code inside a View
const SPLASH_PREFIX_VIEW: &str = "use mod.prelude.widgets.*View{height:Fit, ";
/// Prefix for full-script mode: just imports, code must evaluate to a widget
const SPLASH_PREFIX_SCRIPT: &str = "use mod.prelude.widgets.*\n";

/// Detect whether Splash code is a full script (starts with `let`, `fn`,
/// or a widget constructor like `View{`, `SolidView{`) vs View children
/// (starts with properties like `flow:`, `width:`, or lowercase names).
fn is_full_script(body: &str) -> bool {
    let trimmed = body.trim_start();
    // Only treat as full script if it starts with scripting keywords
    // (let/fn/mod) — these can't appear inside a View{} property list.
    // Uppercase widget names (View{, SolidView{, Label{) stay in View-children mode.
    trimmed.starts_with("let ") || trimmed.starts_with("fn ") || trimmed.starts_with("mod.")
}

fn splash_contains_untrusted_shader(body: &str) -> bool {
    const SHADER_FUNCTION_PROPERTIES: &[&str] = &["pixel", "vertex", "get_color"];

    let compact: String = body
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .flat_map(|ch| ch.to_lowercase())
        .collect();

    SHADER_FUNCTION_PROPERTIES
        .iter()
        .any(|name| compact.contains(&format!("{name}:fn")))
        || compact.contains("sdf2d.")
        || compact.contains("pal.premul")
}

fn splash_shader_guard_body() -> &'static str {
    r#"RoundedView{
    width: Fill height: Fit
    flow: Down spacing: 6 padding: 12
    draw_bg.color: #x2B1D1D
    draw_bg.border_color: #xA85A5A
    draw_bg.border_size: 1.0
    draw_bg.border_radius: 8.0
    Label{text: "This generated view used an unsupported custom shader." draw_text.color: #xFFD8D8}
    Label{text: "Ask the agent to regenerate it using normal widget properties only." draw_text.color: #xD8B8B8 draw_text.text_style.font_size: 10}
}"#
}

impl Splash {
    /// Stable identity for the streaming script body, based on pointer address.
    fn self_id(&self) -> usize {
        self as *const Self as usize
    }

    fn eval_body(&mut self, cx: &mut Cx) {
        let raw_body = self.body.as_ref();
        if raw_body.is_empty() {
            return;
        }
        let guarded_body;
        let body = if splash_contains_untrusted_shader(raw_body) {
            log!(
                "[SPLASH] blocked unsupported custom shader in generated body: {} bytes",
                raw_body.len()
            );
            guarded_body = splash_shader_guard_body();
            guarded_body
        } else {
            raw_body
        };

        // Stop any previous tick timer
        cx.stop_timer(self.tick_timer);

        // Use a unique generation counter so that full content replacements
        // get a fresh VM body instead of hitting the broken content_changed
        // re-parse path in eval_with_append_source.
        self.eval_generation += 1;
        let unique_id = self.self_id().wrapping_add(self.eval_generation as usize);
        self.last_unique_id = unique_id;

        // Choose prefix based on code style
        let prefix = if is_full_script(body) {
            SPLASH_PREFIX_SCRIPT
        } else {
            SPLASH_PREFIX_VIEW
        };
        let code = format!("{}{}", prefix, body);

        let script_mod = ScriptMod {
            cargo_manifest_path: String::new(),
            module_path: String::new(),
            file: String::new(),
            line: unique_id,
            column: 0,
            code: String::new(),
            values: vec![],
        };

        log!(
            "[SPLASH] eval_body: {} bytes, prefix={}",
            body.len(),
            if is_full_script(body) {
                "script"
            } else {
                "view"
            }
        );

        let mut replaced = false;
        cx.with_vm(|vm| {
            let value = vm.eval_with_append_source(script_mod, &code, NIL.into());
            if !value.is_err() && !value.is_nil() {
                self.view = View::script_from_value(vm, value);
                replaced = true;
            }
        });
        if replaced {
            self.view.set_visible(cx, true);
        }

        // If the Splash code defines fn tick(), auto-start a 1s interval
        if body.contains("fn tick(") || body.contains("fn tick (") {
            self.tick_timer = cx.start_interval(1.0);
        }
    }

    /// Call a named function defined in the Splash code's scope.
    pub fn call_fn(&mut self, cx: &mut Cx, name: LiveId) {
        let unique_id = self.last_unique_id;
        if unique_id == 0 {
            return;
        }

        cx.with_vm(|vm| {
            // Find the body by matching the unique_id we used during eval
            let scope_obj = {
                let bodies = vm.bx.code.bodies.borrow();
                let mut found = None;
                for body in bodies.iter() {
                    if let ScriptSource::Mod(m) = &body.source {
                        if m.line == unique_id {
                            found = Some(body.scope.as_object());
                            break;
                        }
                    }
                }
                found
            };

            if let Some(scope) = scope_obj {
                let tick_fn = vm.bx.heap.scope_value(scope, name, vm.trap());
                if !tick_fn.is_nil() && !tick_fn.is_err() {
                    vm.call(tick_fn, &[]);
                }
            }
        });

        cx.redraw_all();
    }

    /// Start a new streaming session. Resets the accumulated code and
    /// increments the generation so the VM creates a fresh body.
    pub fn stream_begin(&mut self, cx: &mut Cx) {
        self.eval_generation += 1;
        self.body.set("");
        // Eval a minimal empty view to clear previous content
        self.body.set("View{}");
        self.eval_body(cx);
        self.body.set("");
        cx.redraw_all();
    }

    /// Append a chunk of Splash code and incrementally re-evaluate.
    /// The VM reuses the same body (fixed line ID) so only new tokens
    /// are tokenized and parsed via checkpoint-based streaming.
    pub fn stream_append(&mut self, cx: &mut Cx, chunk: &str) {
        // Append to body
        let mut current = self.body.as_ref().to_string();
        current.push_str(chunk);
        self.body.set(&current);
        if splash_contains_untrusted_shader(&current) {
            self.eval_body(cx);
            cx.redraw_all();
            return;
        }

        let prefix = if is_full_script(&current) {
            SPLASH_PREFIX_SCRIPT
        } else {
            SPLASH_PREFIX_VIEW
        };
        let code = format!("{}{}", prefix, current);

        // Use a fixed line ID (based on self_id + current generation)
        // so eval_with_append_source finds the existing body and
        // only tokenizes/parses the new delta.
        let unique_id = self.self_id().wrapping_add(self.eval_generation as usize);

        let script_mod = ScriptMod {
            cargo_manifest_path: String::new(),
            module_path: String::new(),
            file: String::new(),
            line: unique_id,
            column: 0,
            code: String::new(),
            values: vec![],
        };

        cx.with_vm(|vm| {
            let value = vm.eval_with_append_source(script_mod, &code, NIL.into());
            if !value.is_err() && !value.is_nil() {
                self.view = View::script_from_value(vm, value);
            }
        });

        cx.redraw_all();
    }
}

#[cfg(test)]
mod tests {
    use super::splash_contains_untrusted_shader;

    #[test]
    fn splash_shader_guard_allows_plain_widget_styling() {
        let body = r##"RoundedView{
            height: Fit
            draw_bg.color: #334
            draw_bg.border_radius: 8.0
            Label{text: "ok"}
        }"##;
        assert!(!splash_contains_untrusted_shader(body));
    }

    #[test]
    fn splash_shader_guard_blocks_shader_callbacks() {
        assert!(splash_contains_untrusted_shader(
            "View{draw_bg +: { pixel: fn() { return #f00 } }}"
        ));
        assert!(splash_contains_untrusted_shader(
            "View{draw_bg +: { get_color : fn() { return self.color } }}"
        ));
        assert!(splash_contains_untrusted_shader(
            "View{draw_bg +: { let sdf = Sdf2d.viewport(self.pos) }}"
        ));
    }
}

impl Widget for Splash {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // Handle tick timer — call tick() in the Splash code's scope
        if self.tick_timer.is_event(event).is_some() {
            self.call_fn(cx, id!(tick));
        }

        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }

    fn text(&self) -> String {
        self.body.as_ref().to_string()
    }

    fn set_text(&mut self, cx: &mut Cx, v: &str) {
        if self.body.as_ref() != v {
            self.body.set(v);
            self.eval_body(cx);
            // eval_body replaces self.view with a new View whose area is not
            // yet registered in the draw system, so self.redraw(cx) would be
            // a no-op.  Force a full redraw so the parent re-layouts.
            cx.redraw_all();
        }
    }
}

impl SplashRef {
    pub fn set_text(&self, cx: &mut Cx, v: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_text(cx, v);
        }
    }

    pub fn stream_begin(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.stream_begin(cx);
        }
    }

    pub fn stream_append(&self, cx: &mut Cx, chunk: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.stream_append(cx, chunk);
        }
    }
}
