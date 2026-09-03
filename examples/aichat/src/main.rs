pub use makepad_code_editor;
pub use makepad_widgets;

use makepad_ai_hub::{
    chat_wire::{
        ChatMessage as HubChatMessage, ChatRole as HubChatRole, ProviderAvailability,
        ProviderKind,
    },
    providers::{
        claude_api::ClaudeApiChatProvider,
        octos::OctosChatProvider,
        provider::{ChatProvider, ProviderEvent, TurnInput},
    },
};
use makepad_widgets::makepad_platform::makepad_micro_serde::*;
use makepad_widgets::makepad_platform::thread::SignalToUI;
use makepad_widgets::*;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

app_main!(App);

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.CodeView

    let ChatList = #(ChatList::register_widget(vm)) {
        width: Fill
        height: Fill

        list := PortalList {
            width: Fill
            height: Fill
            flow: Down
            drag_scrolling: true
            auto_tail: true
            smooth_tail: true
            selectable: true
            // Drop (don't pool) items that leave the list so a removed glass message's overlay
            // draw list is freed — the overlay flush then clears its stuck lensing widgets.
            reuse_items: false

            User := glass.Card {
                width: Fill
                height: Fit
                // Extra vertical margin gives the (now smaller) shadow room so it isn't clipped by
                // the list-item bounds - the glass shader expands the quad by shadow_radius.
                margin: Inset{top: 8 bottom: 10 left: 34 right: 2}
                padding: Inset{left: 14 top: 10 right: 14 bottom: 10}
                flow: Overlay
                // Frosted blue glass message bubble: refracts the vector backdrop and tints it
                // blue, instead of a flat solid fill.
                draw_bg +: {
                    corner_radius: 10.0
                    tint_color: #x6fa6ff
                    tint_alpha: 0.16
                    lensing_effect: 0.5
                    border_alpha: 0.5
                    // Smaller, tighter shadow so it doesn't read as fat or get cut off.
                    shadow_radius: 9.0
                    shadow_offset: vec2(0.0, 3.0)
                }

                selectable := Markdown {
                    width: Fill
                    height: Fit
                    selectable: true
                    use_code_block_widget: true
                    use_math_widget: true
                    body: ""
                    code_block := View {
                        width: Fill
                        height: Fit
                        flow: Overlay
                        code_view := CodeView {
                            keep_cursor_at_end: false
                            editor +: {
                                height: Fit
                                draw_bg +: { color: #1a1a2e }
                            }
                        }
                    }
                    splash_block := View {
                        width: Fill
                        height: Fit
                        splash_view := Splash {
                            allow_net: true
                            width: Fill
                            height: Fit
                        }
                    }
                    inline_math := MathView {
                        font_size: 13.0
                    }
                    display_math := MathView {
                        font_size: 15.0
                    }
                }

                View {
                    width: Fill
                    height: Fit
                    align: Align{x: 1.0}
                    delete_button := ButtonFlat {
                        width: Fit
                        height: Fit
                        padding: Inset{top: 2 bottom: 2 left: 6 right: 6}
                        margin: Inset{top: 2 right: 2}
                        text: "x"
                        draw_text +: {
                            color: #888
                            text_style +: { font_size: 9 }
                        }
                    }
                }
            }

            Assistant := RoundedView {
                width: Fill
                height: Fit
                margin: Inset{top: 4 bottom: 4 left: 2 right: 34}
                padding: Inset{left: 12 top: 8 right: 12 bottom: 8}
                flow: Overlay
                show_bg: true
                // Transparent assistant bubble so glass UIs rendered inside refract the window
                // backdrop (an opaque bubble would be all the glass could "see").
                draw_bg +: {
                    color: #2a2a3a00
                    radius: 8.0
                }

                RubberView {
                    width: Fill
                    height: Fit
                    smoothing: 0.3

                    selectable := Markdown {
                        width: Fill
                        height: Fit
                        selectable: true
                        use_code_block_widget: true
                        use_math_widget: true
                        body: ""
                        draw_text +: {
                            get_color: fn() {
                                let fade_chars = 50.0
                                let dist_from_end = self.total_chars - self.char_index
                                let t = clamp(dist_from_end / fade_chars, 0.0, 1.0)
                                let alpha = pow(t, 0.5)
                                return vec4(self.color.rgb, self.color.a * alpha)
                            }
                        }
                        code_block := View {
                            width: Fill
                            height: Fit
                            flow: Overlay
                            code_view := CodeView {
                                keep_cursor_at_end: true
                                editor +: {
                                    height: Fit
                                    draw_bg +: { color: #1a1a2e }
                                }
                            }
                        }
                        splash_block := SolidView{
                            flow: Overlay
                            new_batch: true
                            width: Fill
                            height: Fit
                            splash_view := Splash {
                                allow_net: true
                                flow: Overlay
                                width: Fill
                                height: Fit
                            }
                        }
                        inline_math := MathView {
                            font_size: 13.0
                        }
                        display_math := MathView {
                            font_size: 15.0
                        }
                    }
                }

                View {
                    width: Fill
                    height: Fit
                    align: Align{x: 1.0}
                    delete_button := ButtonFlat {
                        width: Fit
                        height: Fit
                        padding: Inset{top: 2 bottom: 2 left: 6 right: 6}
                        margin: Inset{top: 2 right: 2}
                        text: "x"
                        draw_text +: {
                            color: #888
                            text_style +: { font_size: 9 }
                        }
                    }
                }
            }
        }
    }

    let ChatActionButton = ButtonIcon {
        height: 44
        padding: Inset{left: 12 right: 12 top: 0 bottom: 0}
        margin: 0
        spacing: 7
        icon_walk: Walk{width: 17 height: 17}
        draw_text +: {
            color: #xffffffff
            color_hover: #xffffffff
            color_down: #xffffffff
            color_focus: #xffffffff
            text_style: theme.font_bold{font_size: 12}
        }
        draw_icon +: {
            color: #xffffffff
            color_hover: #xffffffff
            color_down: #xffffffff
            color_focus: #xffffffff
        }
        draw_bg +: {
            color: #x25324aee
            color_hover: #x334666ff
            color_down: #x182238ff
            color_focus: #x334666ff
            border_color: #x7186a9cc
            border_color_hover: #xa8c3efff
            border_color_down: #x89a9ddff
            border_color_focus: #xa8c3efff
            border_size: 1
            border_radius: 8
        }
    }

    let SendButton = ChatActionButton {
        text: "Send"
        draw_icon.svg: crate_resource("self:resources/send.svg")
        draw_bg +: {
            color: #x2356a8ff
            color_hover: #x2f6bc5ff
            color_down: #x194486ff
            color_focus: #x2f6bc5ff
            border_color: #x73a7ffff
        }
    }

    let CancelButton = ChatActionButton {
        text: "Stop"
        draw_icon.svg: crate_resource("self:resources/stop.svg")
    }

    let ClearButton = ChatActionButton {
        text: "Clear"
        draw_icon.svg: crate_resource("self:resources/trash.svg")
    }

    startup() do #(App::script_component(vm)){
        ui: Root{
            main_window := Window{
                window.inner_size: vec2(900, 700)
                window.title: "AI Chat"
                body +: {
                    flow: Overlay
                    show_bg: true
                    draw_bg.color: #x05070e

                    // Styled backdrop: a crisp VECTOR scene (resolution-independent) so the glass
                    // UIs in the chat have real high-frequency detail to refract/blur. A pre-blurred
                    // shader gradient blurs to nothing; hard vector edges (shapes, rings, ribbons,
                    // dots) are exactly what makes the gauss lensing read as glass.
                    Svg{
                        width: Fill
                        height: Fill
                        // Drive the SVG's animateTransform clock (slowly drifting swirl drapes).
                        animating: true
                        draw_svg +: {
                            // Stretch the art to fill the window (default preserve_aspect letterboxes
                            // a fixed-ratio viewBox, leaving dead flat areas the glass can't lens).
                            preserve_aspect: false
                            svg: crate_resource("self:resources/background.svg")
                        }
                    }
                    // Barely-there veil: just enough to seat the header text, but light enough
                    // that the glass still refracts the FULL-brightness backdrop (a heavier veil
                    // darkened the gauss and made the glass look black).
                    View{
                        width: Fill
                        height: Fill
                        show_bg: true
                        draw_bg.color: #x05070e18
                    }

                    content_layer := AdaptiveView {
                        Desktop := View {
                            width: Fill
                            height: Fill
                            flow: Down
                            padding: Inset{left: 16 top: 16 right: 16 bottom: 16}
                            spacing: 12

                            View {
                                width: Fill
                                height: Fit
                                flow: Right
                                spacing: 12
                                align: Align{y: 0.5}

                                Label {
                                    text: "AI Chat"
                                    draw_text.text_style.font_size: 18
                                }
                                View { width: Fill height: 1 }
                                Label {
                                    text: "Backend:"
                                    draw_text.text_style.font_size: 12
                                }
                                backend_dropdown := DropDown {
                                    width: 170
                                    labels: ["Claude Splash" "Octos · Moonshot"]
                                    draw_text.text_style.font_size: 12
                                }
                            }

                            chat_list := ChatList {}

                            View {
                                width: Fill
                                height: Fit
                                flow: Right
                                spacing: 8
                                align: Align{y: 1.0}

                                input := NativeTextInput {
                                    width: Fill
                                    height: 44
                                    placeholder: "Type a message..."
                                }
                                send_button := SendButton { width: 96 }
                                cancel_button := CancelButton { width: 96 visible: false }
                                clear_button := ClearButton { width: 96 }
                            }

                            status_label := Label {
                                width: Fill
                                height: Fit
                                text: "Initializing..."
                                draw_text.text_style.font_size: 10
                                draw_text.color: #x9ba8bd
                            }
                        }

                        Mobile := View {
                            width: Fill
                            height: Fill
                            flow: Down
                            padding: Inset{left: 10 top: 10 right: 10 bottom: 10}
                            spacing: 8

                            View {
                                width: Fill
                                height: Fit
                                flow: Down
                                spacing: 6

                                View {
                                    width: Fill
                                    height: 36
                                    flow: Right
                                    align: Align{y: 0.5}
                                    Label {
                                        text: "AI Chat"
                                        draw_text.text_style: theme.font_bold{font_size: 17}
                                    }
                                    View { width: Fill height: 1 }
                                    Label {
                                        text: "Mobile"
                                        draw_text.color: #xa9bad3
                                        draw_text.text_style.font_size: 10
                                    }
                                }

                                backend_dropdown := DropDown {
                                    width: Fill
                                    height: 40
                                    labels: ["Claude Splash" "Octos · Moonshot"]
                                    draw_text.text_style.font_size: 12
                                }
                            }

                            chat_list := ChatList {}

                            View {
                                width: Fill
                                height: Fit
                                flow: Down
                                spacing: 7

                                input := NativeTextInput {
                                    width: Fill
                                    height: 48
                                    placeholder: "Message"
                                }

                                View {
                                    width: Fill
                                    height: 44
                                    flow: Right
                                    spacing: 7
                                    send_button := SendButton { width: Fill }
                                    cancel_button := CancelButton { width: 96 visible: false }
                                    clear_button := ClearButton { width: 96 }
                                }
                            }

                            status_label := Label {
                                width: Fill
                                height: 14
                                max_lines: 1
                                text_overflow: Ellipsis
                                text: "Initializing..."
                                draw_text.text_style.font_size: 9
                                draw_text.color: #x9ba8bd
                            }
                        }
                    }
                }
            }
        }
    }
}

// Global chat state accessible to ChatList widget
pub static CHAT_DATA: std::sync::RwLock<ChatData> = std::sync::RwLock::new(ChatData {
    messages: Vec::new(),
    streaming_text: String::new(),
    is_streaming: false,
});

const CHAT_SAVE_PATH: &str = "aichat_history.json";

#[derive(SerJson, DeJson)]
struct SavedMessage {
    role: String,
    content: String,
}

#[derive(SerJson, DeJson, Default)]
struct SavedHistory {
    messages: Vec<SavedMessage>,
}

#[derive(Clone)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub text: String,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ChatRole {
    User,
    Assistant,
}

pub struct ChatData {
    pub messages: Vec<ChatMessage>,
    pub streaming_text: String,
    pub is_streaming: bool,
}

impl ChatData {
    pub fn save_to_disk(&self) {
        let saved = SavedHistory {
            messages: self
                .messages
                .iter()
                .map(|m| SavedMessage {
                    role: match m.role {
                        ChatRole::User => "user".to_string(),
                        ChatRole::Assistant => "assistant".to_string(),
                    },
                    content: m.text.clone(),
                })
                .collect(),
        };
        let _ = std::fs::write(CHAT_SAVE_PATH, saved.serialize_json());
    }

    pub fn load_from_disk() -> Vec<ChatMessage> {
        // Use the saved log if there is one; on a fresh install (no save file yet) seed the chat
        // with the bundled default history so the showcase opens with example apps instead of blank.
        std::fs::read_to_string(CHAT_SAVE_PATH)
            .ok()
            .or_else(|| Some(include_str!("../resources/default_history.json").to_string()))
            .and_then(|s| SavedHistory::deserialize_json(&s).ok())
            .map(|saved| {
                saved
                    .messages
                    .into_iter()
                    .map(|m| ChatMessage {
                        role: if m.role == "user" {
                            ChatRole::User
                        } else {
                            ChatRole::Assistant
                        },
                        text: m.content,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

// ChatList widget wrapping PortalList for chat message display
#[derive(Script, ScriptHook, Widget)]
pub struct ChatList {
    #[deref]
    view: View,
    #[rust]
    animating_msg: Option<usize>,
}

impl Widget for ChatList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let data = CHAT_DATA.read().unwrap();

        while let Some(item) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = item.as_portal_list().borrow_mut() {
                let msg_count = data.messages.len();
                let items_len = msg_count + data.is_streaming as usize;
                list.set_item_range(cx, 0, items_len);

                while let Some(item_id) = list.next_visible_item(cx) {
                    if data.is_streaming && item_id == msg_count {
                        let just_started = self.animating_msg != Some(item_id);
                        if just_started {
                            self.animating_msg = Some(item_id);
                        }

                        let (item_widget, _) = list.item_with_existed(cx, item_id, id!(Assistant));
                        let text = if data.streaming_text.is_empty() {
                            "..."
                        } else {
                            &data.streaming_text
                        };
                        let mut markdown = item_widget.markdown(cx, ids!(selectable));
                        markdown.set_text(cx, text);
                        if just_started {
                            markdown.reset_all_streaming_animations();
                        } else {
                            markdown.start_streaming_animation();
                        }
                        item_widget.draw_all_unscoped(cx);
                        continue;
                    }

                    if let Some(msg) = data.messages.get(item_id) {
                        let is_animating = self.animating_msg == Some(item_id);
                        let template = match msg.role {
                            ChatRole::User => id!(User),
                            ChatRole::Assistant => id!(Assistant),
                        };
                        let item_widget = list.item(cx, item_id, template);
                        let mut markdown = item_widget.markdown(cx, ids!(selectable));
                        markdown.set_text(cx, &msg.text);
                        if is_animating {
                            markdown.stop_streaming_animation();
                        }
                        item_widget.draw_all_unscoped(cx);
                        if is_animating && markdown.is_streaming_animation_done() {
                            self.animating_msg = None;
                        }
                    }
                }
            }
        }
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }
}

fn claude_splash_system_prompt() -> String {
    let splash_md_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../splash.md");
    let splash_md = std::fs::read_to_string(&splash_md_path)
        .unwrap_or_else(|_| include_str!("../../../splash.md").to_string());
    format!(
        r#"You are an AI agent that can create on-demand UI using Makepad's Splash scripting language.

You can answer questions normally using markdown. But when it makes sense to show something visually — a layout, a UI mockup, a styled card, a button arrangement, an animation, or anything graphical — you should embed a ```runsplash code block in your markdown response. The content inside a ```runsplash block is live Splash script that will be rendered as real interactive UI inline in the chat.

IMPORTANT: `use mod.prelude.widgets.*` is automatically prepended to every runsplash block — do NOT include it yourself. All widget names (View, Label, Button, Image, etc.) are already in scope. AI Chat also enables the network sandbox and prepends `use mod.net`, so networked mini apps may use `net.http_request`, `http_resource(...)`, `parse_json()`, and `url_encode()` directly.

For requests to create an app, tool, form, todo app, calculator, editor, or anything with buttons/inputs/lists, produce working Splash business logic inside the ```runsplash block. Splash supports local `let` state, `fn` functions, widget callbacks such as `on_click`, `on_return`, `on_change`, and `CheckBox{{on_click: |checked| ...}}`, plus `ui.<id>.render()`, `ui.<id>.text()`, and `ui.<id>.set_text(...)`.

## ArkUI native widgets

On OpenHarmony, these are the ONLY native ArkUI widget names available in runsplash. Never invent names such as `ArkUIText`, `ArkUILabel`, or `ArkUIInput`:

- `NativeLabel{{text: "..." height: 28}}` renders text through a native ArkUI `Text`. It supports `text()` and `set_text(...)`. Keep styling on a surrounding Splash/glass surface; `NativeLabel` itself only accepts the normal layout properties plus `text` and `visible`.
- `NativeTextInput{{placeholder: "..." height: 48}}` renders a native ArkUI `TextInput`. It supports `text`, `placeholder`, `editable`, `secure`, `visible`, `on_change`, and the methods `text()`, `set_text(...)`, `focus()`, `blur()`, `select_all()`, `copy()`, `cut()`, and `paste()`.
- `GlassNativeTextInput{{placeholder: "..." height: 48}}` and `glass.NativeTextInput{{...}}` are the liquid-glass native ArkUI input. Use one of these whenever the user asks for an ArkUI/native input with glass or liquid-glass styling. It has the same properties and methods as `NativeTextInput`.

When a natural-language request asks to combine ArkUI native components with Splash, use `NativeLabel` for native text, `GlassNativeTextInput` for glass native input, and `glass.*` widgets for the surrounding Splash layout and controls. Native widgets can be named with `:=` and read or updated through `ui.<id>`. Always give native inputs a fixed numeric height. Do not place a native widget inside a clickable control or assume unsupported visual properties such as `font_size` or `font_weight` on it.

Do NOT say that event handlers, mutable state, or render hooks are unavailable in Splash. Do NOT fall back to Rust, `MatchEvent`, `PortalList`, host-app instructions, CLAUDE.md guidance, or project-file edits when the user asks for chat-rendered Splash. For UI/app generation requests, return the `runsplash` block only, with no explanatory prose before or after it.

The block content is Splash script. It gets evaluated and rendered as a live widget tree. Do NOT wrap it in Root{{}} or Window{{}} — the content is placed directly inside a container.

Here is the complete Splash scripting manual. Follow it exactly:

{splash_md}"#
    )
}

/// Default octos host (`POST /chat`). Desktop runs can override it with
/// `MAKEPAD_OCTOS_URL`; a device build carries this constant.
const OCTOS_URL: &str = "http://192.168.1.153:19401";

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum BackendType {
    #[default]
    ClaudeSplash,
    OctosMoonshot,
}

const BACKENDS: [BackendType; 2] = [BackendType::ClaudeSplash, BackendType::OctosMoonshot];

impl BackendType {
    fn to_index(self) -> usize {
        BACKENDS
            .iter()
            .position(|&backend| backend == self)
            .unwrap()
    }

    fn from_index(index: usize) -> Option<Self> {
        BACKENDS.get(index).copied()
    }

    /// OpenHarmony devices have no Claude credentials or CLI; they talk to
    /// the octos agent host on the LAN, which fronts Moonshot.
    fn default_for_platform() -> Self {
        if cfg!(target_env = "ohos") {
            BackendType::OctosMoonshot
        } else {
            BackendType::ClaudeSplash
        }
    }

    fn status_label(self, detail: &str) -> String {
        match self {
            Self::ClaudeSplash => "Active: Claude Splash (Claude API)".to_string(),
            Self::OctosMoonshot => format!("Active: Octos · Moonshot at {}", detail),
        }
    }

    /// The system block for a turn. octos gets a preamble on top of the
    /// Splash manual: it is an agent host, and left to itself it will reach
    /// for tools and files instead of answering with a `runsplash` block.
    fn system_prompt(self) -> String {
        match self {
            Self::ClaudeSplash => claude_splash_system_prompt(),
            Self::OctosMoonshot => format!(
                "You are responding to an embedded Makepad Splash renderer. Do not call tools, \
                 do not write files, and do not produce HTML. Follow these UI-generation \
                 instructions exactly.\n\n{}",
                claude_splash_system_prompt()
            ),
        }
    }
}

enum AiWorkerCommand {
    Send(TurnInput),
    Cancel,
}

enum AiWorkerEvent {
    Availability(ProviderAvailability),
    Delta(String),
    Done(String),
    Error(String),
}

struct AiWorker {
    command_tx: Sender<AiWorkerCommand>,
    event_rx: Receiver<AiWorkerEvent>,
}

impl AiWorker {
    fn new(cx: &mut Cx, backend: BackendType) -> Self {
        let (command_tx, command_rx) = mpsc::channel();
        let (event_tx, event_rx) = mpsc::channel();
        cx.spawn_thread(move || ai_worker_loop(backend, command_rx, event_tx));
        Self {
            command_tx,
            event_rx,
        }
    }

    fn send(&self, input: TurnInput) -> Result<(), String> {
        self.command_tx
            .send(AiWorkerCommand::Send(input))
            .map_err(|_| "AI provider worker ended".to_string())
    }

    fn cancel(&self) {
        let _ = self.command_tx.send(AiWorkerCommand::Cancel);
    }

    fn poll(&self) -> Vec<AiWorkerEvent> {
        self.event_rx.try_iter().collect()
    }
}

fn emit_ai_worker_event(event_tx: &Sender<AiWorkerEvent>, event: AiWorkerEvent) -> bool {
    if event_tx.send(event).is_err() {
        return false;
    }
    SignalToUI::set_ui_signal();
    true
}

/// Opens the provider for `backend`. A provider that cannot even be
/// configured (bad octos URL) is reported as unavailable instead of a
/// worker that silently never answers.
fn open_provider(backend: BackendType) -> Result<Box<dyn ChatProvider>, String> {
    match backend {
        BackendType::ClaudeSplash => Ok(Box::new(ClaudeApiChatProvider::from_env(
            ProviderKind::ClaudeCli,
            None,
        ))),
        BackendType::OctosMoonshot => Ok(Box::new(OctosChatProvider::from_env(OCTOS_URL)?)),
    }
}

fn ai_worker_loop(
    backend: BackendType,
    command_rx: Receiver<AiWorkerCommand>,
    event_tx: Sender<AiWorkerEvent>,
) {
    let mut provider = match open_provider(backend) {
        Ok(provider) => provider,
        Err(reason) => {
            emit_ai_worker_event(
                &event_tx,
                AiWorkerEvent::Availability(ProviderAvailability::Unavailable { reason }),
            );
            return;
        }
    };
    if !emit_ai_worker_event(
        &event_tx,
        AiWorkerEvent::Availability(provider.availability()),
    ) {
        return;
    }

    loop {
        match command_rx.recv_timeout(Duration::from_millis(16)) {
            Ok(AiWorkerCommand::Send(input)) => {
                if let Err(error) = provider.begin_turn(&input) {
                    if !emit_ai_worker_event(&event_tx, AiWorkerEvent::Error(error)) {
                        return;
                    }
                }
            }
            Ok(AiWorkerCommand::Cancel) => provider.cancel(),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                provider.cancel();
                return;
            }
        }

        for event in provider.poll() {
            let event = match event {
                ProviderEvent::Delta(text) => Some(AiWorkerEvent::Delta(text)),
                ProviderEvent::Done { text } => Some(AiWorkerEvent::Done(text)),
                ProviderEvent::Error(error) => Some(AiWorkerEvent::Error(error)),
                ProviderEvent::FunctionCall { .. } => Some(AiWorkerEvent::Error(
                    "AI provider requested an unsupported function".to_string(),
                )),
                ProviderEvent::Status { .. } | ProviderEvent::Serving(_) => None,
            };
            if let Some(event) = event {
                if !emit_ai_worker_event(&event_tx, event) {
                    provider.cancel();
                    return;
                }
            }
        }
    }
}

#[derive(Script, ScriptHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
    #[rust]
    ai_worker: Option<AiWorker>,
    #[rust(false)]
    current_prompt: bool,
    #[rust]
    active_backend: BackendType,
    #[rust]
    backend_available: bool,
    /// Where the provider says it lives (octos base URL, API endpoint);
    /// shown in the status row.
    #[rust]
    backend_detail: String,
    #[rust]
    backend_unavailable_reason: String,
}

impl App {
    fn create_backend_session(&mut self, cx: &mut Cx, backend: BackendType) {
        if let Some(worker) = &self.ai_worker {
            worker.cancel();
        }
        self.ai_worker = None;
        self.current_prompt = false;
        self.active_backend = backend;
        self.backend_available = false;
        self.backend_detail.clear();
        self.backend_unavailable_reason.clear();
        self.ai_worker = Some(AiWorker::new(cx, backend));
        self.update_status(cx);
    }

    fn clear_chat(&mut self, cx: &mut Cx) {
        {
            let mut data = CHAT_DATA.write().unwrap();
            data.messages.clear();
            data.streaming_text.clear();
            data.is_streaming = false;
            data.save_to_disk();
        }
        self.create_backend_session(cx, self.active_backend);
        // Full repaint (not just ui.redraw) so the window overlay pass is rebuilt — the glass
        // widgets draw into self-managed overlay draw lists, and a partial redraw can leave
        // those stale lists composited (the "stuck glass after Clear" bug).
        cx.redraw_all();
    }

    fn send_message(&mut self, cx: &mut Cx) {
        let input = self.ui.native_text_input(cx, ids!(input));
        let text = input.text();
        if text.trim().is_empty() {
            return;
        }

        let Some(worker) = &self.ai_worker else {
            return;
        };
        if !self.backend_available {
            self.update_status(cx);
            return;
        }

        let (items_len, messages) = {
            let mut data = CHAT_DATA.write().unwrap();
            data.messages.push(ChatMessage {
                role: ChatRole::User,
                text: text.clone(),
            });
            data.streaming_text.clear();
            data.is_streaming = true;
            let messages = data
                .messages
                .iter()
                .map(|message| {
                    HubChatMessage::new(
                        match message.role {
                            ChatRole::User => HubChatRole::User,
                            ChatRole::Assistant => HubChatRole::Assistant,
                        },
                        message.text.clone(),
                    )
                })
                .collect();
            (data.messages.len() + 1, messages)
        };
        input.set_text(cx, "");

        let turn = TurnInput::new(self.active_backend.system_prompt(), messages);
        if let Err(error) = worker.send(turn) {
            CHAT_DATA.write().unwrap().is_streaming = false;
            self.ui
                .label(cx, ids!(status_label))
                .set_text(cx, &format!("Error: {}", error));
            return;
        }
        self.current_prompt = true;
        if self.active_backend == BackendType::OctosMoonshot {
            // octos answers in one piece once the agent turn ends; say so
            // instead of showing a static status while nothing streams.
            self.ui
                .label(cx, ids!(status_label))
                .set_text(cx, "Generating Splash UI with Moonshot...");
        }
        self.ui.widget(cx, ids!(cancel_button)).set_visible(cx, true);

        let chat_list = self.ui.widget(cx, ids!(chat_list));
        let list = chat_list.portal_list(cx, ids!(list));
        list.set_tail_range(true);
        list.set_first_id_and_scroll(items_len.saturating_sub(1), 0.0);
        self.ui.redraw(cx);
    }

    fn cancel_request(&mut self, cx: &mut Cx) {
        if self.current_prompt {
            if let Some(worker) = &self.ai_worker {
                worker.cancel();
            }
            self.current_prompt = false;

            let mut data = CHAT_DATA.write().unwrap();
            let text = std::mem::take(&mut data.streaming_text);
            if !text.is_empty() {
                data.messages.push(ChatMessage {
                    role: ChatRole::Assistant,
                    text,
                });
            }
            data.is_streaming = false;
            drop(data);

            self.ui.widget(cx, ids!(cancel_button)).set_visible(cx, false);
            self.update_status(cx);
            self.ui.redraw(cx);
        }
    }

    fn update_status(&self, cx: &mut Cx) {
        let status = if self.backend_available {
            self.active_backend.status_label(&self.backend_detail)
        } else if self.backend_unavailable_reason.is_empty() {
            "Checking AI provider availability...".to_string()
        } else {
            format!("Unavailable: {}", self.backend_unavailable_reason)
        };
        self.ui
            .label(cx, ids!(status_label))
            .set_text(cx, &status);
    }
}

impl MatchEvent for App {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        if self.ui.button(cx, ids!(send_button)).clicked(actions) {
            self.send_message(cx);
        }
        if self.ui.button(cx, ids!(cancel_button)).clicked(actions) {
            self.cancel_request(cx);
        }
        if self.ui.button(cx, ids!(clear_button)).clicked(actions) {
            self.clear_chat(cx);
        }
        if let Some(index) = self
            .ui
            .drop_down(cx, ids!(backend_dropdown))
            .selected(actions)
        {
            if let Some(backend) = BackendType::from_index(index) {
                if backend != self.active_backend {
                    self.cancel_request(cx);
                    self.create_backend_session(cx, backend);
                }
            }
        }

        // Handle message deletion
        let chat_list = self.ui.widget(cx, ids!(chat_list));
        let list = chat_list.portal_list(cx, ids!(list));
        for (item_id, item) in list.items_with_actions(actions) {
            if item.button(cx, ids!(delete_button)).pressed(actions) {
                let mut data = CHAT_DATA.write().unwrap();
                if item_id < data.messages.len() {
                    data.messages.remove(item_id);
                    data.save_to_disk();
                }
                drop(data);
                // Full repaint so removing a glass message doesn't leave its overlay stuck.
                cx.redraw_all();
            }
        }
    }

    fn handle_startup(&mut self, cx: &mut Cx) {
        // A device build already defaulted at construction; this catches a
        // desktop binary running on OpenHarmony (simulator, sideload).
        if matches!(cx.os_type(), OsType::OpenHarmony(_)) {
            self.active_backend = BackendType::OctosMoonshot;
        }
        self.ui
            .adaptive_view(cx, ids!(content_layer))
            .set_variant_selector(|cx, parent_size| {
                if matches!(
                    cx.os_type(),
                    OsType::OpenHarmony(_) | OsType::Android(_) | OsType::Ios(_)
                ) || parent_size.x < 700.0
                {
                    live_id!(Mobile)
                } else {
                    live_id!(Desktop)
                }
            });
        self.create_backend_session(cx, self.active_backend);
        self.ui
            .drop_down(cx, ids!(backend_dropdown))
            .set_selected_item(cx, self.active_backend.to_index());
    }
}

impl AppMain for App {
    fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
        crate::makepad_widgets::script_mod(vm);
        crate::makepad_code_editor::script_mod(vm);
        self::script_mod(vm)
    }

    fn after_new_from_script(_vm: &mut ScriptVm, app: &mut Self) {
        CHAT_DATA.write().unwrap().messages = ChatData::load_from_disk();
        app.active_backend = BackendType::default_for_platform();
        app.backend_available = false;
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());

        if let Some(worker) = &self.ai_worker {
            for event in worker.poll() {
                match event {
                    AiWorkerEvent::Availability(ProviderAvailability::Available {
                        detail, ..
                    }) => {
                        self.backend_available = true;
                        self.backend_detail = detail;
                        self.backend_unavailable_reason.clear();
                        self.update_status(cx);
                    }
                    AiWorkerEvent::Availability(ProviderAvailability::Unavailable { reason }) => {
                        self.backend_available = false;
                        self.backend_detail.clear();
                        self.backend_unavailable_reason = reason;
                        self.update_status(cx);
                    }
                    AiWorkerEvent::Delta(text) => {
                        let item_id = {
                            let mut data = CHAT_DATA.write().unwrap();
                            data.streaming_text.push_str(&text);
                            data.messages.len()
                        };
                        let chat_list = self.ui.widget(cx, ids!(chat_list));
                        let list = chat_list.portal_list(cx, ids!(list));
                        if let Some((_, item)) = list.get_item(item_id) {
                            item.widget(cx, ids!(splash_view)).redraw(cx);
                        }
                        cx.redraw_all();
                    }
                    AiWorkerEvent::Done(full_text) => {
                        let mut data = CHAT_DATA.write().unwrap();
                        if data.streaming_text.is_empty() {
                            data.streaming_text = full_text;
                        }
                        let text = std::mem::take(&mut data.streaming_text);
                        if !text.is_empty() {
                            data.messages.push(ChatMessage {
                                role: ChatRole::Assistant,
                                text,
                            });
                        }
                        data.is_streaming = false;
                        data.save_to_disk();
                        drop(data);

                        self.current_prompt = false;
                        self.ui.widget(cx, ids!(cancel_button)).set_visible(cx, false);
                        self.update_status(cx);
                        cx.redraw_all();
                    }
                    AiWorkerEvent::Error(error) => {
                        CHAT_DATA.write().unwrap().is_streaming = false;
                        self.current_prompt = false;
                        self.ui.widget(cx, ids!(cancel_button)).set_visible(cx, false);
                        self.ui
                            .label(cx, ids!(status_label))
                            .set_text(cx, &format!("Error: {}", error));
                        cx.redraw_all();
                    }
                }
            }
        }
    }
}
