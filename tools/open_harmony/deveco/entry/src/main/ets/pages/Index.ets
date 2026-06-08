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
  xComponentAttrs: XComponentAttrs = {
    id: 'Entry',
    type: XComponentType.SURFACE,
    libraryname: 'entry',
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
      current.programmatic = false;
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

  createNativeTextInput(inputId: string, text: string, placeholder: string, editable: boolean): void {
    this.upsertNativeTextInput({
      inputId,
      text,
      placeholder,
      editable,
      left: 0,
      top: 0,
      width: 1,
      height: 1,
      visible: false,
      programmatic: false,
      selectionStart: 0,
      selectionEnd: 0,
      controller: new TextInputController()
    });
  }

  updateNativeTextInput(inputId: string, left: number, top: number, width: number, height: number, visible: boolean): void {
    this.updateNativeTextInputState(inputId, (input: NativeTextInputState): NativeTextInputState => {
      input.left = left;
      input.top = top;
      input.width = Math.max(1, width);
      input.height = Math.max(1, height);
      input.visible = visible;
      return input;
    });
  }

  detachNativeTextInput(inputId: string): void {
    this.updateNativeTextInputState(inputId, (input: NativeTextInputState): NativeTextInputState => {
      input.visible = false;
      return input;
    });
  }

  setNativeTextInputText(inputId: string, text: string, programmatic: boolean): void {
    this.updateNativeTextInputState(inputId, (input: NativeTextInputState): NativeTextInputState => {
      input.text = text;
      input.programmatic = programmatic;
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
    this.requestNativeFocus('native_text_input_' + inputId);
  }

  blurNativeTextInput(_inputId: string): void {
    this.requestNativeFocus('makepad_xcomponent');
  }

  selectAllNativeTextInput(inputId: string): void {
    let input = this.nativeTextInput(inputId);
    if (input === undefined) {
      return;
    }
    this.setNativeTextInputSelection(input, 0, input.text.length);
  }

  copyNativeTextInput(inputId: string): void {
    let input = this.nativeTextInput(inputId);
    if (input === undefined) {
      return;
    }
    this.requestNativeFocus('native_text_input_' + input.inputId);
    let selectedText = this.selectedNativeTextInputText(input);
    if (selectedText.length > 0) {
      this.setSystemClipboardText(selectedText);
    }
  }

  cutNativeTextInput(inputId: string): void {
    let input = this.nativeTextInput(inputId);
    if (input === undefined || !input.editable) {
      return;
    }
    this.requestNativeFocus('native_text_input_' + input.inputId);
    let selectedText = this.selectedNativeTextInputText(input);
    if (selectedText.length > 0 && this.setSystemClipboardText(selectedText)) {
      this.replaceNativeTextInputSelection(input, '');
    }
  }

  pasteNativeTextInput(inputId: string): void {
    let input = this.nativeTextInput(inputId);
    if (input === undefined || !input.editable) {
      return;
    }
    this.requestNativeFocus('native_text_input_' + input.inputId);
    let clipboardText = this.systemClipboardText();
    if (clipboardText !== undefined && clipboardText.length > 0) {
      this.replaceNativeTextInputSelection(input, clipboardText);
    }
  }

  closeNativeTextInput(inputId: string): void {
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
          .onChange((value: string) => {
            if (input.programmatic) {
              this.setNativeTextInputText(input.inputId, value, false);
              return;
            }
            this.setNativeTextInputText(input.inputId, value, false);
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
