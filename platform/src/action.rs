use crate::thread::SignalToUI;
use crate::{
    cx::Cx,
    cx_api::{CxOsOp, NativeLabelCloseRequested, NativeTextInputCloseRequested},
};
use std::any::TypeId;
use std::fmt;
use std::fmt::Debug;

use std::sync::{mpsc::Sender, Mutex};

pub(crate) static ACTION_SENDER_GLOBAL: Mutex<Option<Sender<ActionSend>>> = Mutex::new(None);

pub trait ActionTrait: 'static {
    fn debug_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
    fn ref_cast_type_id(&self) -> TypeId
    where
        Self: 'static,
    {
        TypeId::of::<Self>()
    }
}

impl<T: 'static + Debug + ?Sized> ActionTrait for T {
    fn debug_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt(f)
    }
}

impl dyn ActionTrait {
    pub fn is<T: ActionTrait + 'static>(&self) -> bool {
        let t = TypeId::of::<T>();
        let concrete = self.ref_cast_type_id();
        t == concrete
    }
    pub fn downcast_ref<T: ActionTrait + 'static>(&self) -> Option<&T> {
        if self.is::<T>() {
            Some(unsafe { &*(self as *const dyn ActionTrait as *const T) })
        } else {
            None
        }
    }
    pub fn downcast_mut<T: ActionTrait + 'static>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            Some(unsafe { &mut *(self as *const dyn ActionTrait as *mut T) })
        } else {
            None
        }
    }
}

impl Debug for dyn ActionTrait {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug_fmt(f)
    }
}
pub type ActionSend = Box<dyn ActionTrait + Send>;
pub type Action = Box<dyn ActionTrait>;
pub type ActionsBuf = Vec<Action>;
pub type Actions = [Action];

pub trait ActionDefaultRef {
    fn default_ref() -> &'static Self;
}

pub trait ActionCast<T> {
    fn cast(&self) -> T;
}

pub trait ActionCastRef<T> {
    fn cast_ref(&self) -> &T;
}

impl<T: ActionTrait + Default + Clone> ActionCast<T> for Box<dyn ActionTrait> {
    fn cast(&self) -> T {
        if let Some(item) = (*self).downcast_ref::<T>() {
            item.clone()
        } else {
            T::default()
        }
    }
}

impl<T: ActionTrait + ActionDefaultRef> ActionCastRef<T> for Box<dyn ActionTrait> {
    fn cast_ref(&self) -> &T {
        if let Some(item) = (*self).downcast_ref::<T>() {
            item
        } else {
            T::default_ref()
        }
    }
}

impl<T: ActionTrait + ActionDefaultRef> ActionCastRef<T>
    for Option<std::sync::Arc<dyn ActionTrait>>
{
    fn cast_ref(&self) -> &T {
        if let Some(item) = self {
            if let Some(item) = item.downcast_ref::<T>() {
                return item;
            }
        }
        T::default_ref()
    }
}

impl Cx {
    pub fn handle_action_receiver(&mut self) {
        while let Ok(action) = self.action_receiver.try_recv() {
            self.handle_received_action(action);
        }
        self.handle_actions();
    }

    fn handle_received_action(&mut self, action: ActionSend) {
        let action_ref = action.as_ref() as &dyn ActionTrait;
        if let Some(close) = action_ref.downcast_ref::<NativeTextInputCloseRequested>() {
            self.platform_ops.push(CxOsOp::CloseNativeView {
                id: close.text_input_id,
            });
        } else if let Some(close) = action_ref.downcast_ref::<NativeLabelCloseRequested>() {
            self.platform_ops
                .push(CxOsOp::CloseNativeView { id: close.label_id });
        } else {
            self.new_actions.push(action);
        }
    }

    /// Enqueues an action from a background thread context.
    ///
    /// This will produce a bare action, *not* a widget action,
    /// so you cannot use `as_widget_action()` when handling this action.
    ///
    /// If there is no live `Cx` to receive the action — because one has not
    /// been created yet, or (commonly on mobile) because the app is shutting
    /// down or being backgrounded and the `Cx` was already dropped — the
    /// action is silently discarded. A background thread (a tokio task, a
    /// `robius-*` crate callback, etc.) racing app teardown is normal and
    /// must not panic the whole process.
    pub fn post_action(action: impl ActionTrait + Send) {
        let Ok(mut sender_guard) = ACTION_SENDER_GLOBAL.lock() else {
            // The mutex is poisoned (a thread panicked while holding it).
            // Nothing useful we can do — drop the action.
            return;
        };
        let Some(sender) = sender_guard.as_mut() else {
            // No `Cx` has installed an action sender (yet / anymore).
            return;
        };
        // `send` fails once the `Cx`'s receiver has been dropped at shutdown.
        // Only signal the UI when the action was actually enqueued.
        if sender.send(Box::new(action)).is_ok() {
            SignalToUI::set_action_signal();
        }
    }

    pub fn try_post_action(action: impl ActionTrait + Send) -> bool {
        let Ok(mut sender) = ACTION_SENDER_GLOBAL.lock() else {
            return false;
        };
        let Some(sender) = sender.as_mut() else {
            return false;
        };
        if sender.send(Box::new(action)).is_err() {
            return false;
        }
        SignalToUI::set_action_signal();
        true
    }

    pub fn action(&mut self, action: impl ActionTrait) {
        self.new_actions.push(Box::new(action));
    }

    /// Adds the given `actions` back into the set of existing queued actions.
    ///
    /// This is useful when you want to allow other widgets elsewhere in the UI tree
    /// to receive the given `actions`, e.g., after you have previously captured them
    /// using `capture_actions()`.
    pub fn extend_actions(&mut self, actions: ActionsBuf) {
        self.new_actions.extend(actions);
    }

    pub fn map_actions<F, G, R>(&mut self, f: F, g: G) -> R
    where
        F: FnOnce(&mut Cx) -> R,
        G: FnOnce(&mut Cx, ActionsBuf) -> ActionsBuf,
    {
        let start = self.new_actions.len();
        let r = f(self);
        let end = self.new_actions.len();
        if start != end {
            let buf = self.new_actions.drain(start..end).collect();
            let buf = g(self, buf);
            self.new_actions.extend(buf);
        }
        r
    }

    pub fn mutate_actions<F, G, R>(&mut self, f: F, g: G) -> R
    where
        F: FnOnce(&mut Cx) -> R,
        G: FnOnce(&mut [Action]),
    {
        let start = self.new_actions.len();
        let r = f(self);
        let end = self.new_actions.len();
        if start != end {
            g(&mut self.new_actions[start..end]);
        }
        r
    }

    /// Captures the actions emitted by the given closure `f` and returns them.
    ///
    /// This allows you to handle the actions directly before they are delivered
    /// to other widgets in the UI tree, enabling you to optionally prevent some or all
    /// of the actions from being delivered to all other widgets.
    ///
    /// If you *do* want some or all of the returned `actions` to be delivered
    /// to other widgets in the UI tree, you can call `extend_actions()` to enqueue them
    /// back into the set of existing actions.
    pub fn capture_actions<F>(&mut self, f: F) -> ActionsBuf
    where
        F: FnOnce(&mut Cx),
    {
        let mut actions = Vec::new();
        std::mem::swap(&mut self.new_actions, &mut actions);
        f(self);
        std::mem::swap(&mut self.new_actions, &mut actions);
        actions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{event::Event, makepad_live_id::LiveId};

    fn test_cx() -> Cx {
        Cx::new(Box::new(|_cx: &mut Cx, _event: &Event| {}))
    }

    #[test]
    fn native_text_input_close_request_becomes_platform_close_op() {
        let mut cx = test_cx();
        let text_input_id = LiveId(101);

        cx.handle_received_action(Box::new(NativeTextInputCloseRequested::new(text_input_id)));

        assert_eq!(cx.new_actions.len(), 0);
        assert!(matches!(
            cx.platform_ops.as_slice(),
            [CxOsOp::CloseNativeView { id }] if *id == text_input_id
        ));
    }

    #[test]
    fn native_label_close_request_becomes_platform_close_op() {
        let mut cx = test_cx();
        let label_id = LiveId(202);

        cx.handle_received_action(Box::new(NativeLabelCloseRequested::new(label_id)));

        assert_eq!(cx.new_actions.len(), 0);
        assert!(matches!(
            cx.platform_ops.as_slice(),
            [CxOsOp::CloseNativeView { id }] if *id == label_id
        ));
    }
}
