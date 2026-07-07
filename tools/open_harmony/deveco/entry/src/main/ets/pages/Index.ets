import { pasteboard } from '@kit.BasicServicesKit';
import {
  ArkGlue,
  MakepadXComponentInterface,
  NativeLabelState,
  NativeTextInputState,
  XComponentAttrs
} from '../makepad/makepad';

// Read-only native label overlay; same @Observed/@ObjectLink pattern as the
// text input item so programmatic text/layout updates reach the screen.
@Component
struct NativeLabelItem {
  @ObjectLink state: NativeLabelState;

  build() {
    Text(this.state.text)
      .id('native_label_' + this.state.inputId)
      .position({ x: this.state.left, y: this.state.top })
      .width(this.state.width)
      .height(this.state.height)
      .fontColor('#334155')
      .visibility(this.state.visible ? Visibility.Visible : Visibility.Hidden)
  }
}

// Per-input overlay item. @ObjectLink is what makes programmatic state
// changes (set_text, layout, visibility, editable) reach the on-screen
// TextInput: ForEach reuses items whose key is unchanged, so a plain
// interface in the parent never re-renders the reused child. It also means
// callbacks below always read the live state object, never a stale render
// snapshot.
@Component
struct NativeTextInputItem {
  @ObjectLink state: NativeTextInputState;
  traceCb: (message: string) => void = (_message: string) => {};
  changedCb: (inputId: string, value: string) => void = (_inputId: string, _value: string) => {};
  focusCb: (inputId: string, hasFocus: boolean) => void = (_inputId: string, _hasFocus: boolean) => {};
  selectionCb: (inputId: string, start: number, end: number) => void = (_inputId: string, _start: number, _end: number) => {};
  appearCb: (inputId: string) => void = (_inputId: string) => {};

  build() {
    TextInput({ text: this.state.text, placeholder: this.state.placeholder, controller: this.state.controller })
      .id('native_text_input_' + this.state.inputId)
      .position({ x: this.state.left, y: this.state.top })
      .width(this.state.width)
      .height(this.state.height)
      .enabled(this.state.editable)
      .type(this.state.secure ? InputType.Password : InputType.Normal)
      .visibility(this.state.visible ? Visibility.Visible : Visibility.Hidden)
      .onAppear(() => {
        this.traceCb('attached id=' + this.state.inputId);
        this.state.attached = true;
        this.appearCb(this.state.inputId);
      })
      .onAreaChange((_oldValue: Area, newValue: Area) => {
        this.traceCb('area id=' + this.state.inputId
          + ' arkui_global=' + JSON.stringify(newValue.globalPosition)
          + ' arkui_size=' + newValue.width + 'x' + newValue.height);
      })
      .onChange((value: string) => {
        if (this.state.pendingProgrammaticEchoes > 0 && value === this.state.lastProgrammaticText) {
          this.state.pendingProgrammaticEchoes -= 1;
          this.state.text = value;
          this.traceCb('changed id=' + this.state.inputId + ' programmatic_echo=true len=' + value.length);
          return;
        }
        // A real user edit invalidates any echoes that never fired.
        this.state.pendingProgrammaticEchoes = 0;
        this.state.text = value;
        this.state.selectionStart = Math.min(this.state.selectionStart, value.length);
        this.state.selectionEnd = Math.min(this.state.selectionEnd, value.length);
        this.traceCb('changed id=' + this.state.inputId + ' programmatic_echo=false len=' + value.length);
        this.changedCb(this.state.inputId, value);
      })
      .onFocus(() => {
        this.focusCb(this.state.inputId, true);
      })
      .onBlur(() => {
        this.focusCb(this.state.inputId, false);
      })
      .onTextSelectionChange((start: number, end: number) => {
        this.state.selectionStart = Math.min(start, end);
        this.state.selectionEnd = Math.max(start, end);
        this.selectionCb(this.state.inputId, start, end);
      })
  }
}

@Entry
@Component
struct Index {
  xComponentContext: MakepadXComponentInterface | undefined = undefined;
  @State nativeTextInputs: NativeTextInputState[] = [];
  @State nativeLabels: NativeLabelState[] = [];
  private pendingCommands: Map<string, string[]> = new Map<string, string[]>();
  xComponentAttrs: XComponentAttrs = {
    id: 'Entry',
    type: XComponentType.SURFACE,
    // Must match the Rust NAPI module name registered by libmakepad.so.
    // 'entry' loads a nonexistent libentry.so: the XComponent then never
    // delivers __NATIVE_XCOMPONENT_OBJ__ to Rust, no surface callbacks fire,
    // and the app shows a black screen with zero makepad-ohos hilog output.
    libraryname: 'makepad',
  }

  private traceNTI(message: string): void {
    console.info('[MakepadNTI] ' + message);
  }

  private upsertNativeTextInput(input: NativeTextInputState): void {
    let inputs = this.nativeTextInputs.slice();
    let index = inputs.findIndex((item: NativeTextInputState) => item.inputId === input.inputId);
    if (index >= 0) {
      inputs[index] = input;
    } else {
      inputs.push(input);
    }
    this.nativeTextInputs = inputs;
  }

  // Mutates the live @Observed state object; @ObjectLink in the item
  // component propagates the change to the rendered TextInput.
  private updateNativeTextInputState(inputId: string, updater: (input: NativeTextInputState) => void): void {
    let input = this.nativeTextInput(inputId);
    if (input === undefined) {
      // Unlike commands, state updates have no pre-create queue: Rust must
      // create before updating. Trace instead of silently dropping so an
      // ordering violation is visible in the evidence logs.
      this.traceNTI('update id=' + inputId + ' dropped=unknown_id');
      return;
    }
    updater(input);
  }

  private nativeTextInput(inputId: string): NativeTextInputState | undefined {
    return this.nativeTextInputs.find((item: NativeTextInputState) => item.inputId === inputId);
  }

  private requestNativeFocus(focusId: string): void {
    focusControl.requestFocus(focusId);
  }

  private clampSelection(input: NativeTextInputState): [number, number] {
    let textLength = input.text.length;
    let start = Math.max(0, Math.min(input.selectionStart, textLength));
    let end = Math.max(0, Math.min(input.selectionEnd, textLength));
    return start <= end ? [start, end] : [end, start];
  }

  private setNativeTextInputSelection(input: NativeTextInputState, start: number, end: number): void {
    let textLength = input.text.length;
    let selectionStart = Math.max(0, Math.min(start, textLength));
    let selectionEnd = Math.max(0, Math.min(end, textLength));
    let nextStart = Math.min(selectionStart, selectionEnd);
    let nextEnd = Math.max(selectionStart, selectionEnd);
    input.selectionStart = nextStart;
    input.selectionEnd = nextEnd;
    this.requestNativeFocus('native_text_input_' + input.inputId);
    try {
      input.controller.setTextSelection(nextStart, nextEnd);
    } catch (err) {
      console.error('NativeTextInput setTextSelection failed: ' + JSON.stringify(err));
    }
    this.xComponentContext?.handleNativeTextInputSelectionChanged(input.inputId, nextStart, nextEnd);
  }

  private selectedNativeTextInputText(input: NativeTextInputState): string {
    let selection = this.clampSelection(input);
    let start = selection[0];
    let end = selection[1];
    if (start === end) {
      return '';
    }
    return input.text.substring(start, end);
  }

  private setSystemClipboardText(text: string): boolean {
    try {
      let data = pasteboard.createData(pasteboard.MIMETYPE_TEXT_PLAIN, text);
      pasteboard.getSystemPasteboard().setDataSync(data);
      return true;
    } catch (err) {
      console.error('NativeTextInput set clipboard failed: ' + JSON.stringify(err));
      return false;
    }
  }

  private systemClipboardText(): string | undefined {
    try {
      let systemPasteboard = pasteboard.getSystemPasteboard();
      if (!systemPasteboard.hasDataSync()) {
        return undefined;
      }
      return systemPasteboard.getDataSync().getPrimaryText();
    } catch (err) {
      console.error('NativeTextInput get clipboard failed: ' + JSON.stringify(err));
      return undefined;
    }
  }

  private replaceNativeTextInputSelection(input: NativeTextInputState, replacement: string): void {
    let selection = this.clampSelection(input);
    let start = selection[0];
    let end = selection[1];
    let nextText = input.text.substring(0, start) + replacement + input.text.substring(end);
    let nextSelection = start + replacement.length;
    // The state-driven text change re-renders the TextInput and may fire
    // onChange; arm an echo so the explicit changed callback here stays the
    // single changed event for this edit.
    input.text = nextText;
    input.pendingProgrammaticEchoes += 1;
    input.lastProgrammaticText = nextText;
    input.selectionStart = nextSelection;
    input.selectionEnd = nextSelection;
    this.xComponentContext?.handleNativeTextInputChanged(input.inputId, nextText);
    this.xComponentContext?.handleNativeTextInputSelectionChanged(input.inputId, nextSelection, nextSelection);
    this.requestNativeFocus('native_text_input_' + input.inputId);
    try {
      input.controller.setTextSelection(nextSelection, nextSelection);
    } catch (err) {
      console.error('NativeTextInput update selection failed: ' + JSON.stringify(err));
    }
  }

  private runNativeTextInputCommand(input: NativeTextInputState, command: string): void {
    if (command === 'focus') {
      this.requestNativeFocus('native_text_input_' + input.inputId);
      this.traceNTI('command id=' + input.inputId + ' command=focus result=request_focus');
      return;
    }
    if (command === 'blur') {
      let stoppedEditing = false;
      try {
        input.controller.stopEditing();
        stoppedEditing = true;
      } catch (err) {
        console.error('NativeTextInput stopEditing failed: ' + JSON.stringify(err));
      }
      if (!stoppedEditing) {
        // Documented OHOS host fallback: move focus to the Makepad
        // XComponent as a focus sink when the controller path fails.
        this.requestNativeFocus('makepad_xcomponent');
      }
      this.traceNTI('command id=' + input.inputId + ' command=blur controller_stop_editing=' + stoppedEditing);
      return;
    }
    if (command === 'select_all') {
      this.setNativeTextInputSelection(input, 0, input.text.length);
      this.traceNTI('command id=' + input.inputId + ' command=select_all result=selection 0..' + input.text.length);
      return;
    }
    if (command === 'copy') {
      this.requestNativeFocus('native_text_input_' + input.inputId);
      let selectedText = this.selectedNativeTextInputText(input);
      if (selectedText.length > 0) {
        let copied = this.setSystemClipboardText(selectedText);
        this.traceNTI('command id=' + input.inputId + ' command=copy result=' + (copied ? 'copied len=' + selectedText.length : 'clipboard_error'));
      } else {
        this.traceNTI('command id=' + input.inputId + ' command=copy result=empty_selection');
      }
      return;
    }
    if (command === 'cut') {
      if (!input.editable) {
        this.traceNTI('command id=' + input.inputId + ' command=cut result=rejected_not_editable');
        return;
      }
      this.requestNativeFocus('native_text_input_' + input.inputId);
      let selectedText = this.selectedNativeTextInputText(input);
      if (selectedText.length > 0 && this.setSystemClipboardText(selectedText)) {
        this.replaceNativeTextInputSelection(input, '');
        this.traceNTI('command id=' + input.inputId + ' command=cut result=cut len=' + selectedText.length);
      } else {
        this.traceNTI('command id=' + input.inputId + ' command=cut result=empty_selection');
      }
      return;
    }
    if (command === 'paste') {
      if (!input.editable) {
        this.traceNTI('command id=' + input.inputId + ' command=paste result=rejected_not_editable');
        return;
      }
      this.requestNativeFocus('native_text_input_' + input.inputId);
      let clipboardText = this.systemClipboardText();
      if (clipboardText !== undefined && clipboardText.length > 0) {
        this.replaceNativeTextInputSelection(input, clipboardText);
        this.traceNTI('command id=' + input.inputId + ' command=paste result=pasted len=' + clipboardText.length);
      } else {
        this.traceNTI('command id=' + input.inputId + ' command=paste result=empty_clipboard');
      }
      return;
    }
    this.traceNTI('command id=' + input.inputId + ' command=' + command + ' result=rejected_unknown_command');
  }

  private dispatchNativeTextInputCommand(inputId: string, command: string): void {
    let input = this.nativeTextInput(inputId);
    if (input === undefined || !input.attached) {
      let queue = this.pendingCommands.get(inputId);
      if (queue === undefined) {
        queue = [];
      }
      queue.push(command);
      this.pendingCommands.set(inputId, queue);
      let reason: string = input === undefined ? 'not_created' : 'not_attached';
      this.traceNTI('command id=' + inputId + ' command=' + command + ' queued=true reason=' + reason);
      return;
    }
    this.runNativeTextInputCommand(input, command);
  }

  private flushPendingCommands(inputId: string): void {
    let queue = this.pendingCommands.get(inputId);
    this.pendingCommands.delete(inputId);
    if (queue === undefined || queue.length === 0) {
      return;
    }
    for (let command of queue) {
      this.traceNTI('command id=' + inputId + ' command=' + command + ' queued=false flushed=true');
      this.dispatchNativeTextInputCommand(inputId, command);
    }
  }

  createNativeTextInput(inputId: string, text: string, placeholder: string, editable: boolean, secure: boolean): void {
    let existing = this.nativeTextInput(inputId);
    let attached = existing !== undefined ? existing.attached : false;
    this.traceNTI('create id=' + inputId + ' editable=' + editable + ' secure=' + secure + ' attached=' + attached);
    let state = new NativeTextInputState();
    state.inputId = inputId;
    state.text = text;
    state.placeholder = placeholder;
    state.editable = editable;
    state.secure = secure;
    state.attached = attached;
    state.controller = existing !== undefined ? existing.controller : new TextInputController();
    this.upsertNativeTextInput(state);
  }

  updateNativeTextInput(inputId: string, left: number, top: number, width: number, height: number, visible: boolean): void {
    let appliedWidth = Math.max(1, width);
    let appliedHeight = Math.max(1, height);
    this.traceNTI('layout id=' + inputId
      + ' makepad_rect=' + left + ',' + top + ',' + width + ',' + height
      + ' applied_size=' + appliedWidth + 'x' + appliedHeight
      + ' visible=' + visible
      + ' vp_px=' + vp2px(1));
    this.updateNativeTextInputState(inputId, (input: NativeTextInputState): void => {
      input.left = left;
      input.top = top;
      input.width = appliedWidth;
      input.height = appliedHeight;
      input.visible = visible;
    });
  }

  detachNativeTextInput(inputId: string): void {
    this.traceNTI('detach id=' + inputId);
    this.updateNativeTextInputState(inputId, (input: NativeTextInputState): void => {
      input.visible = false;
    });
  }

  setNativeTextInputText(inputId: string, text: string, programmatic: boolean): void {
    this.traceNTI('set_text id=' + inputId + ' programmatic=' + programmatic + ' len=' + text.length);
    this.updateNativeTextInputState(inputId, (input: NativeTextInputState): void => {
      if (programmatic && input.text !== text) {
        input.pendingProgrammaticEchoes += 1;
        input.lastProgrammaticText = text;
      }
      input.text = text;
      input.selectionStart = Math.min(input.selectionStart, text.length);
      input.selectionEnd = Math.min(input.selectionEnd, text.length);
    });
  }

  setNativeTextInputPlaceholder(inputId: string, placeholder: string): void {
    this.updateNativeTextInputState(inputId, (input: NativeTextInputState): void => {
      input.placeholder = placeholder;
    });
  }

  setNativeTextInputEditable(inputId: string, editable: boolean): void {
    this.updateNativeTextInputState(inputId, (input: NativeTextInputState): void => {
      input.editable = editable;
    });
  }

  setNativeTextInputSecure(inputId: string, secure: boolean): void {
    this.traceNTI('set_secure id=' + inputId + ' secure=' + secure);
    this.updateNativeTextInputState(inputId, (input: NativeTextInputState): void => {
      input.secure = secure;
    });
  }

  focusNativeTextInput(inputId: string): void {
    this.dispatchNativeTextInputCommand(inputId, 'focus');
  }

  blurNativeTextInput(inputId: string): void {
    this.dispatchNativeTextInputCommand(inputId, 'blur');
  }

  selectAllNativeTextInput(inputId: string): void {
    this.dispatchNativeTextInputCommand(inputId, 'select_all');
  }

  copyNativeTextInput(inputId: string): void {
    this.dispatchNativeTextInputCommand(inputId, 'copy');
  }

  cutNativeTextInput(inputId: string): void {
    this.dispatchNativeTextInputCommand(inputId, 'cut');
  }

  pasteNativeTextInput(inputId: string): void {
    this.dispatchNativeTextInputCommand(inputId, 'paste');
  }

  closeNativeTextInput(inputId: string): void {
    this.traceNTI('close id=' + inputId);
    this.pendingCommands.delete(inputId);
    this.nativeTextInputs = this.nativeTextInputs.filter((input: NativeTextInputState) => input.inputId !== inputId);
  }

  private nativeLabel(inputId: string): NativeLabelState | undefined {
    return this.nativeLabels.find((item: NativeLabelState) => item.inputId === inputId);
  }

  createNativeLabel(inputId: string, text: string): void {
    this.traceNTI('label_create id=' + inputId + ' len=' + text.length);
    let existing = this.nativeLabel(inputId);
    if (existing !== undefined) {
      existing.text = text;
      return;
    }
    let state = new NativeLabelState();
    state.inputId = inputId;
    state.text = text;
    let labels = this.nativeLabels.slice();
    labels.push(state);
    this.nativeLabels = labels;
  }

  updateNativeLabel(inputId: string, left: number, top: number, width: number, height: number, visible: boolean): void {
    let label = this.nativeLabel(inputId);
    if (label === undefined) {
      this.traceNTI('label_layout id=' + inputId + ' dropped=unknown_id');
      return;
    }
    this.traceNTI('label_layout id=' + inputId
      + ' makepad_rect=' + left + ',' + top + ',' + width + ',' + height
      + ' visible=' + visible + ' vp_px=' + vp2px(1));
    label.left = left;
    label.top = top;
    label.width = Math.max(1, width);
    label.height = Math.max(1, height);
    label.visible = visible;
  }

  setNativeLabelText(inputId: string, text: string): void {
    let label = this.nativeLabel(inputId);
    if (label === undefined) {
      this.traceNTI('label_set_text id=' + inputId + ' result=rejected_not_created');
      return;
    }
    this.traceNTI('label_set_text id=' + inputId + ' len=' + text.length);
    label.text = text;
  }

  detachNativeLabel(inputId: string): void {
    this.traceNTI('label_detach id=' + inputId);
    let label = this.nativeLabel(inputId);
    if (label !== undefined) {
      label.visible = false;
    }
  }

  closeNativeLabel(inputId: string): void {
    this.traceNTI('label_close id=' + inputId);
    this.nativeLabels = this.nativeLabels.filter((label: NativeLabelState) => label.inputId !== inputId);
  }

  build() {
    Stack() {
      Row() {
        Column() {
          XComponent(this.xComponentAttrs)
            .id('makepad_xcomponent')
            .focusable(true)
            .onLoad((xComponentContext) => {
              this.xComponentContext = xComponentContext as MakepadXComponentInterface;
              ArkGlue.getInstance().xComponentContext = this.xComponentContext;
              ArkGlue.getInstance().nativeTextInputHost = this;
              //this.xComponentContext.initMakepad(ArkGlue.getInstance());
            })
        }
        .width('100%')
      }
      .height('100%')

      ForEach(this.nativeTextInputs, (input: NativeTextInputState) => {
        NativeTextInputItem({
          state: input,
          traceCb: (message: string): void => {
            this.traceNTI(message);
          },
          changedCb: (inputId: string, value: string): void => {
            this.xComponentContext?.handleNativeTextInputChanged(inputId, value);
          },
          focusCb: (inputId: string, hasFocus: boolean): void => {
            this.xComponentContext?.handleNativeTextInputFocusChanged(inputId, hasFocus);
          },
          selectionCb: (inputId: string, start: number, end: number): void => {
            this.xComponentContext?.handleNativeTextInputSelectionChanged(inputId, start, end);
          },
          appearCb: (inputId: string): void => {
            this.flushPendingCommands(inputId);
          }
        })
      }, (input: NativeTextInputState) => input.inputId)

      ForEach(this.nativeLabels, (label: NativeLabelState) => {
        NativeLabelItem({ state: label })
      }, (label: NativeLabelState) => label.inputId)
    }
  }
}
