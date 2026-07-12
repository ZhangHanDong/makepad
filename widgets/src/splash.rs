use crate::{
    makepad_derive_widget::*,
    makepad_draw::*,
    view::View,
    widget::*,
    widget_async::{CxSplashVmExt, SplashVmId, MAIN_SPLASH_VM_ID},
    widget_tree::CxWidgetExt,
};

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

#[derive(Script, ScriptHook, WidgetRef, WidgetRegister)]
pub struct Splash {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[deref]
    pub view: View,
    #[live]
    body: ArcStringMut,
    #[live]
    allow_net: bool,
    #[rust]
    eval_generation: u64,
    #[rust]
    tick_timer: Timer,
    /// The unique_id used for the last full eval, so tick() runs in the same scope.
    #[rust]
    last_unique_id: usize,
    /// This Splash's own VM, allocated on first eval (upstream isolation model).
    #[rust]
    vm_id: SplashVmId,
    /// Body text of the previous eval. Used to detect streaming extensions
    /// (the new body forward-extends the old) so repeated set_text(full growing
    /// text) reuses ONE vm body instead of a fresh generation per frame.
    #[rust]
    last_eval_body: String,
}

/// Prefix for View-children mode: wraps code inside a View
const SPLASH_PREFIX_VIEW: &str = "use mod.prelude.widgets.*View{height:Fit, ";
/// Prefix for full-script mode: just imports, code must evaluate to a widget
const SPLASH_PREFIX_SCRIPT: &str = "use mod.prelude.widgets.*\n";
/// Net-enabled variants (allow_net: true) additionally import mod.net
const SPLASH_PREFIX_VIEW_NET: &str = "use mod.prelude.widgets.*\nuse mod.net\nView{height:Fit, ";
const SPLASH_PREFIX_SCRIPT_NET: &str = "use mod.prelude.widgets.*\nuse mod.net\n";
const SPLASH_EVAL_INSTRUCTION_LIMIT: usize = 200_000;

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

impl Splash {
    /// Stable identity for the streaming script body, based on pointer address.
    fn self_id(&self) -> usize {
        self as *const Self as usize
    }

    fn eval_body(&mut self, cx: &mut Cx) {
        let body = self.body.as_ref().to_string();
        if body.is_empty() {
            return;
        }

        // Stop any previous tick timer
        cx.stop_timer(self.tick_timer);

        // Allocate this Splash's own VM on first eval so streaming
        // (stream_append) evaluates in an isolated scope.
        if self.vm_id == MAIN_SPLASH_VM_ID {
            self.vm_id = cx.alloc_splash_vm_with_network(self.allow_net);
        }

        // Only start a NEW vm body (bump the generation) on a genuine content
        // replacement — NOT a streaming extension of the previous body. aichat
        // streams runsplash by calling set_text() with the full, growing block
        // string every frame; without this each frame got its own generation,
        // accumulating dozens of stale bodies whose widgets/closures lingered
        // (clicking a button then hit a stale generation -> "widget not found in
        // tree" -> the app vanished). A forward-extension reuses the same
        // unique_id so eval_with_append_source does its incremental checkpoint
        // parse (the same path stream_append uses). Compare the raw body (not the
        // prefixed code) so an is_full_script flip can't cause a false miss.
        let is_extension =
            !self.last_eval_body.is_empty() && body.starts_with(self.last_eval_body.as_str());
        if !is_extension {
            self.eval_generation += 1;
        }
        self.last_eval_body = body.clone();
        let unique_id = self.self_id().wrapping_add(self.eval_generation as usize);
        self.last_unique_id = unique_id;

        // Choose prefix based on code style (fork) x network access (upstream)
        let prefix = match (is_full_script(&body), self.allow_net) {
            (true, false) => SPLASH_PREFIX_SCRIPT,
            (true, true) => SPLASH_PREFIX_SCRIPT_NET,
            (false, false) => SPLASH_PREFIX_VIEW,
            (false, true) => SPLASH_PREFIX_VIEW_NET,
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
            "[SPLASH] eval_body: {} bytes, prefix={}, uid={}, gen={}, ext={}",
            body.len(),
            if is_full_script(&body) {
                "script"
            } else {
                "view"
            },
            unique_id,
            self.eval_generation,
            is_extension
        );

        // Evaluate in THIS Splash's own isolated vm and inject a `ui` global
        // rooted at this Splash (self.uid). That scopes `ui.<id>` to this
        // Splash's subtree (find_flood), so ids like `display` don't collide
        // with other Splash apps in the same chat. Then register the widgets
        // under this vm and mark the tree dirty so lookups can resolve them.
        let vm_id = self.vm_id;
        let self_uid = self.uid;
        let new_view = cx.with_script_vm_id(vm_id, |vm| {
            crate::widget_async::inject_scoped_ui_global(vm, self_uid);
            let value = vm.with_instruction_limit(SPLASH_EVAL_INSTRUCTION_LIMIT, |vm| {
                vm.eval_with_append_source(script_mod, &code, NIL.into())
            });
            if !value.is_err() && !value.is_nil() {
                Some(View::script_from_value(vm, value))
            } else {
                None
            }
        });

        if let Some(view) = new_view {
            self.view = view;
            self.view.set_visible(cx, true);
            crate::widget_async::inject_splash_ui_handle(cx, self.vm_id, self.view.widget_uid());
            cx.widget_tree_mark_dirty(self.uid);
        }

        // Start the 1s tick timer only if `tick` actually resolves as a
        // callable function in the eval'd scope. The old string test
        // body.contains("fn tick(") is NOT equivalent: it could start a timer
        // for a `tick` that isn't callable at top level (e.g. nested in a
        // View{}/widget block, or view-wrapped because is_full_script picked
        // view mode), producing a misleading `variable tick not found` every
        // second with no working timer.
        let tick_found = self.resolves_fn(cx, id!(tick));
        if tick_found {
            self.tick_timer = cx.start_interval(1.0);
        }
        log!(
            "[SPLASH] eval done: uid={}, gen={}, tick_found={}",
            unique_id,
            self.eval_generation,
            tick_found
        );
    }

    /// Whether `name` resolves to a non-nil, non-error value (a callable
    /// function) at the scope of the current (last-eval'd) body. Mirrors the
    /// lookup `call_fn` performs, so the tick timer only runs when `tick` is
    /// actually invokable.
    fn resolves_fn(&mut self, cx: &mut Cx, name: LiveId) -> bool {
        let unique_id = self.last_unique_id;
        if unique_id == 0 {
            return false;
        }
        cx.with_script_vm_id(self.vm_id, |vm| {
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
                let f = vm.bx.heap.scope_value(scope, name, vm.trap());
                !f.is_nil() && !f.is_err()
            } else {
                false
            }
        })
    }

    /// Call a named function defined in the Splash code's scope.
    pub fn call_fn(&mut self, cx: &mut Cx, name: LiveId) {
        let unique_id = self.last_unique_id;
        if unique_id == 0 {
            return;
        }

        cx.with_script_vm_id(self.vm_id, |vm| {
            // Find the body by matching the unique_id we used during eval
            // (body lives in this Splash's isolated vm, same as eval_body).
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

        let vm_id = self.vm_id;
        let self_uid = self.uid;
        let new_view = cx.with_script_vm_id(vm_id, |vm| {
            crate::widget_async::inject_scoped_ui_global(vm, self_uid);
            let value = vm.with_instruction_limit(SPLASH_EVAL_INSTRUCTION_LIMIT, |vm| {
                vm.eval_with_append_source(script_mod, &code, NIL.into())
            });
            if !value.is_err() && !value.is_nil() {
                Some(View::script_from_value(vm, value))
            } else {
                None
            }
        });

        if let Some(view) = new_view {
            self.view = view;
            // Make `ui` a global in this splash's VM (pointing at the freshly-built view root) so
            // helper `fn`s inside the block can use `ui.<id>.set_text(...)`, not just inline
            // handlers. Without this, calculators/forms that route through a helper silently fail.
            crate::widget_async::inject_splash_ui_handle(cx, self.vm_id, self.view.widget_uid());
            cx.widget_tree_mark_dirty(self.uid);
        }
    }
}

impl WidgetNode for Splash {
    fn widget_uid(&self) -> WidgetUid {
        self.uid
    }

    fn walk(&mut self, cx: &mut Cx) -> Walk {
        self.view.walk(cx)
    }

    fn area(&self) -> Area {
        self.view.area()
    }

    fn redraw(&mut self, cx: &mut Cx) {
        self.view.redraw(cx);
    }

    fn children(&self, visit: &mut dyn FnMut(LiveId, WidgetRef)) {
        self.view.children(visit);
    }
}

impl Drop for Splash {
    fn drop(&mut self) {
        // A Splash owns an isolate script VM. `Drop` has no `Cx`, so it can't free
        // the VM here; it just marks the id for reclamation. The isolate is torn
        // down later by `gc_dead_splash_isolates` (on the next isolate alloc, async
        // pump, or Splash event) while a `Cx` is available and nothing runs in it.
        crate::widget_async::mark_splash_isolate_dead(self.vm_id);
    }
}

impl Widget for Splash {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // Handle tick timer — call tick() in the Splash code's scope
        if self.tick_timer.is_event(event).is_some() {
            self.call_fn(cx, id!(tick));
        }

        if self.allow_net {
            if let Event::NetworkResponses(responses) = event {
                crate::widget_async::handle_splash_network_responses(cx, self.vm_id, responses);
            }
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

#[cfg(test)]
mod agent_module_tests {
    use super::*;

    fn with_widgets_vm<R>(f: impl FnOnce(&mut ScriptVm) -> R) -> R {
        let mut cx = Cx::new(Box::new(|_, _| {}));
        cx.with_vm(|vm| {
            crate::makepad_draw::makepad_platform::script::script_mod(vm);
            crate::script_mod(vm);
            f(vm)
        })
    }

    #[test]
    fn test_script_mod_injects_agent_global() {
        with_widgets_vm(|vm| {
            let agent = vm.eval(script! { agent });
            assert!(agent.is_object(), "agent should resolve to a module object");

            let notify = vm.eval(script! { agent.notify });
            assert!(
                notify.is_object(),
                "agent.notify should resolve to a callable function object"
            );
        });
    }

    #[test]
    fn test_register_agent_module_twice_keeps_notify_callable() {
        with_widgets_vm(|vm| {
            register_agent_module(vm);

            let notify = vm.eval(script! { agent.notify });
            assert!(
                notify.is_object(),
                "agent.notify should remain callable after double registration"
            );
        });
    }

    #[test]
    fn test_aichat_has_no_explicit_agent_registration() {
        let aichat = include_str!("../../examples/aichat/src/main.rs");

        assert!(
            !aichat.contains("register_agent_module"),
            "aichat should rely on makepad_widgets::script_mod for agent registration"
        );
    }

    #[test]
    fn test_agent_notify_missing_args_returns_nil() {
        with_widgets_vm(|vm| {
            let result = vm.eval(script! { agent.notify() });
            assert!(result.is_nil(), "agent.notify() should return NIL");
            assert!(
                !result.is_err(),
                "agent.notify() without arguments should not error"
            );

            let subsequent = vm.eval(script! { 123 });
            assert_eq!(
                subsequent.as_number(),
                Some(123.0),
                "VM should evaluate normally after agent.notify()"
            );
        });
    }
}
