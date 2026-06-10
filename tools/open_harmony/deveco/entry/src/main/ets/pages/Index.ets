import { pasteboard } from '@kit.BasicServicesKit';
import {
  ArkGlue,
  MakepadXComponentInterface,
  NativeTextInputState,
  XComponentAttrs
} from '../makepad/makepad';

@Entry
@Component
struct Index {
  xComponentContext: MakepadXComponentInterface | undefined = undefined;
  @State nativeTextInputs: NativeTextInputState[] = [];
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

  private updateNativeTextInputState(inputId: string, updater: (input: NativeTextInputState) => NativeTextInputState): void {
    let inputs = this.nativeTextInputs.slice();
    let index = inputs.findIndex((item: NativeTextInputState) => item.inputId === inputId);
    if (index < 0) {
      return;
    }
    inputs[index] = updater(inputs[index]);
    this.nativeTextInputs = inputs;
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
    this.updateNativeTextInputState(input.inputId, (current: NativeTextInputState): NativeTextInputState => {
      current.selectionStart = nextStart;
      current.selectionEnd = nextEnd;
      return current;
    });
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
    this.updateNativeTextInputState(input.inputId, (current: NativeTextInputState): NativeTextInputState => {
      current.text = nextText;
      // The state-driven text change below re-renders the ArkUI TextInput and
      // may fire onChange; arm an echo so the explicit changed callback here
      // stays the single changed event for this edit.
      current.pendingProgrammaticEchoes += 1;
      current.lastProgrammaticText = nextText;
      current.selectionStart = nextSelection;
      current.selectionEnd = nextSelection;
      return current;
    });
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
      let reason = input === undefined ? 'not_created' : 'not_attached';
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

  createNativeTextInput(inputId: string, text: string, placeholder: string, editable: boolean): void {
    let existing = this.nativeTextInput(inputId);
    let attached = existing !== undefined ? existing.attached : false;
    this.traceNTI('create id=' + inputId + ' editable=' + editable + ' attached=' + attached);
    this.upsertNativeTextInput({
      inputId: inputId,
      text: text,
      placeholder: placeholder,
      editable: editable,
      left: 0,
      top: 0,
      width: 1,
      height: 1,
      visible: false,
      pendingProgrammaticEchoes: 0,
      lastProgrammaticText: '',
      attached: attached,
      selectionStart: 0,
      selectionEnd: 0,
      controller: existing !== undefined ? existing.controller : new TextInputController()
    });
  }

  updateNativeTextInput(inputId: string, left: number, top: number, width: number, height: number, visible: boolean): void {
    let appliedWidth = Math.max(1, width);
    let appliedHeight = Math.max(1, height);
    this.traceNTI('layout id=' + inputId
      + ' makepad_rect=' + left + ',' + top + ',' + width + ',' + height
      + ' applied_size=' + appliedWidth + 'x' + appliedHeight
      + ' visible=' + visible
      + ' vp_px=' + vp2px(1));
    this.updateNativeTextInputState(inputId, (input: NativeTextInputState): NativeTextInputState => {
      input.left = left;
      input.top = top;
      input.width = appliedWidth;
      input.height = appliedHeight;
      input.visible = visible;
      return input;
    });
  }

  detachNativeTextInput(inputId: string): void {
    this.traceNTI('detach id=' + inputId);
    this.updateNativeTextInputState(inputId, (input: NativeTextInputState): NativeTextInputState => {
      input.visible = false;
      return input;
    });
  }

  setNativeTextInputText(inputId: string, text: string, programmatic: boolean): void {
    this.traceNTI('set_text id=' + inputId + ' programmatic=' + programmatic + ' len=' + text.length);
    this.updateNativeTextInputState(inputId, (input: NativeTextInputState): NativeTextInputState => {
      if (programmatic && input.text !== text) {
        input.pendingProgrammaticEchoes += 1;
        input.lastProgrammaticText = text;
      }
      input.text = text;
      input.selectionStart = Math.min(input.selectionStart, text.length);
      input.selectionEnd = Math.min(input.selectionEnd, text.length);
      return input;
    });
  }

  setNativeTextInputPlaceholder(inputId: string, placeholder: string): void {
    this.updateNativeTextInputState(inputId, (input: NativeTextInputState): NativeTextInputState => {
      input.placeholder = placeholder;
      return input;
    });
  }

  setNativeTextInputEditable(inputId: string, editable: boolean): void {
    this.updateNativeTextInputState(inputId, (input: NativeTextInputState): NativeTextInputState => {
      input.editable = editable;
      return input;
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
        TextInput({ text: input.text, placeholder: input.placeholder, controller: input.controller })
          .id('native_text_input_' + input.inputId)
          .position({ x: input.left, y: input.top })
          .width(input.width)
          .height(input.height)
          .enabled(input.editable)
          .visibility(input.visible ? Visibility.Visible : Visibility.Hidden)
          .onAppear(() => {
            this.traceNTI('attached id=' + input.inputId);
            this.updateNativeTextInputState(input.inputId, (current: NativeTextInputState): NativeTextInputState => {
              current.attached = true;
              return current;
            });
            this.flushPendingCommands(input.inputId);
          })
          .onAreaChange((_oldValue: Area, newValue: Area) => {
            this.traceNTI('area id=' + input.inputId
              + ' arkui_global=' + JSON.stringify(newValue.globalPosition)
              + ' arkui_size=' + newValue.width + 'x' + newValue.height);
          })
          .onChange((value: string) => {
            // Re-resolve current state: the closure-captured `input` is the
            // render-time snapshot and its guard fields may be stale.
            let current = this.nativeTextInput(input.inputId);
            if (current === undefined) {
              return;
            }
            if (current.pendingProgrammaticEchoes > 0 && value === current.lastProgrammaticText) {
              this.updateNativeTextInputState(input.inputId, (state: NativeTextInputState): NativeTextInputState => {
                state.pendingProgrammaticEchoes -= 1;
                state.text = value;
                return state;
              });
              this.traceNTI('changed id=' + input.inputId + ' programmatic_echo=true len=' + value.length);
              return;
            }
            this.updateNativeTextInputState(input.inputId, (state: NativeTextInputState): NativeTextInputState => {
              // A real user edit invalidates any echoes that never fired.
              state.pendingProgrammaticEchoes = 0;
              state.text = value;
              state.selectionStart = Math.min(state.selectionStart, value.length);
              state.selectionEnd = Math.min(state.selectionEnd, value.length);
              return state;
            });
            this.traceNTI('changed id=' + input.inputId + ' programmatic_echo=false len=' + value.length);
            this.xComponentContext?.handleNativeTextInputChanged(input.inputId, value);
          })
          .onFocus(() => {
            this.xComponentContext?.handleNativeTextInputFocusChanged(input.inputId, true);
          })
          .onBlur(() => {
            this.xComponentContext?.handleNativeTextInputFocusChanged(input.inputId, false);
          })
          .onTextSelectionChange((start: number, end: number) => {
            this.updateNativeTextInputState(input.inputId, (current: NativeTextInputState): NativeTextInputState => {
              current.selectionStart = Math.min(start, end);
              current.selectionEnd = Math.max(start, end);
              return current;
            });
            this.xComponentContext?.handleNativeTextInputSelectionChanged(input.inputId, start, end);
          })
      }, (input: NativeTextInputState) => input.inputId)
    }
  }
}
