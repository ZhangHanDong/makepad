import display from '@ohos.display';
import deviceInfo from '@ohos.deviceInfo';
import inputMethod from '@ohos.inputMethod';
import { window } from '@kit.ArkUI';
import common from '@ohos.app.ability.common';
import resourceManager from '@ohos.resourceManager';

declare type Any = ESObject;

export interface MakepadXComponentInterface{
  onCreate(ark_ts: object):void;
  handleInsertTextEvent(text:String):void;
  handleDeleteLeftEvent(length: number):void;
  handleKeyboardStatus(isOpen:boolean, keyboardHeight:number):void;
  handleNativeTextInputChanged(inputId: string, text: string): void;
  handleNativeTextInputFocusChanged(inputId: string, hasFocus: boolean): void;
  handleNativeTextInputSelectionChanged(inputId: string, start: number, end: number): void;
}

export interface NativeTextInputHost {
  createNativeTextInput(inputId: string, text: string, placeholder: string, editable: boolean): void;
  updateNativeTextInput(inputId: string, left: number, top: number, width: number, height: number, visible: boolean): void;
  detachNativeTextInput(inputId: string): void;
  setNativeTextInputText(inputId: string, text: string, programmatic: boolean): void;
  setNativeTextInputPlaceholder(inputId: string, placeholder: string): void;
  setNativeTextInputEditable(inputId: string, editable: boolean): void;
  focusNativeTextInput(inputId: string): void;
  blurNativeTextInput(inputId: string): void;
  selectAllNativeTextInput(inputId: string): void;
  copyNativeTextInput(inputId: string): void;
  cutNativeTextInput(inputId: string): void;
  pasteNativeTextInput(inputId: string): void;
  closeNativeTextInput(inputId: string): void;
}

export interface NativeTextInputState {
  inputId: string;
  text: string;
  placeholder: string;
  editable: boolean;
  left: number;
  top: number;
  width: number;
  height: number;
  visible: boolean;
  // ArkUI may or may not fire onChange for a programmatic text update
  // depending on version, so a boolean flag cannot pair sets with echoes.
  // Each programmatic set increments the echo count; onChange only swallows
  // a callback when the count is positive AND the value matches the last
  // programmatic text, so stale echoes can never eat newer user input.
  pendingProgrammaticEchoes: number;
  lastProgrammaticText: string;
  // Commands arriving between state insertion and ArkUI component attach
  // (same Rust op batch) must be queued; onAppear flips this and flushes.
  attached: boolean;
  selectionStart: number;
  selectionEnd: number;
  controller: TextInputController;
}

function getDensity(): number {
  try {
    let displayClass = display.getDefaultDisplaySync();
    console.info('Test densityDPI:' + JSON.stringify(displayClass.densityDPI));
    return displayClass.densityDPI / 160;
  } catch (exception) {
    console.error('Failed to obtain the default display object. Code: ' + JSON.stringify(exception));
    return 3;
  }
}

function getDeviceType(): string {
  let device_type: string = deviceInfo.deviceType;
  if (device_type == "") {
    console.error("deviceInfo.deviceType is empty string!")
  } else {
    console.info("Device type is " + device_type)
  }
  return device_type;
}

export class ArkGlue{
  private static instance : ArkGlue;
  static initInstance(context: common.Context) : ArkGlue {
    if(ArkGlue.instance == null) {
      ArkGlue.instance = new ArkGlue(context);
    }
      return ArkGlue.instance;
  }

  static getInstance() : ArkGlue {
    return ArkGlue.instance;
  }

  context: common.Context;

  deviceType = getDeviceType();
  osFullName = deviceInfo.osFullName;
  displayDensity = getDensity();
  resMgr : resourceManager.ResourceManager;
  filesDir : string;
  cacheDir : string;
  tempDir : string;

  keyboardHeight : number = 1020;
  xComponentContext : MakepadXComponentInterface | undefined = undefined;
  nativeTextInputHost : NativeTextInputHost | undefined = undefined;

  private inputMethodController : inputMethod.InputMethodController;
  private textConfig: inputMethod.TextConfig;
  private imcFlag = false;

  constructor(context: common.Context) {
    this.context = context;
    this.resMgr = this.context.resourceManager;
    this.filesDir = this.context.filesDir;
    this.cacheDir = this.context.cacheDir;
    this.tempDir = this.context.tempDir;
    this.inputMethodController = inputMethod.getController();
    this.textConfig = {
      inputAttribute: {
        textInputType: 0,
        enterKeyType: 1
      }
    };
  }

  showKeyBoard(): void {
    this.showTextInput();
  }

  hideKeyBoard(): void {
    this.hideTextInput();
  }

  createNativeTextInput(inputId: string, text: string, placeholder: string, editable: boolean): void {
    this.nativeTextInputHost?.createNativeTextInput(inputId, text, placeholder, editable);
  }

  updateNativeTextInput(inputId: string, left: number, top: number, width: number, height: number, visible: boolean): void {
    this.nativeTextInputHost?.updateNativeTextInput(inputId, left, top, width, height, visible);
  }

  detachNativeTextInput(inputId: string): void {
    this.nativeTextInputHost?.detachNativeTextInput(inputId);
  }

  setNativeTextInputText(inputId: string, text: string, programmatic: boolean): void {
    this.nativeTextInputHost?.setNativeTextInputText(inputId, text, programmatic);
  }

  setNativeTextInputPlaceholder(inputId: string, placeholder: string): void {
    this.nativeTextInputHost?.setNativeTextInputPlaceholder(inputId, placeholder);
  }

  setNativeTextInputEditable(inputId: string, editable: boolean): void {
    this.nativeTextInputHost?.setNativeTextInputEditable(inputId, editable);
  }

  focusNativeTextInput(inputId: string): void {
    this.nativeTextInputHost?.focusNativeTextInput(inputId);
  }

  blurNativeTextInput(inputId: string): void {
    this.nativeTextInputHost?.blurNativeTextInput(inputId);
  }

  selectAllNativeTextInput(inputId: string): void {
    this.nativeTextInputHost?.selectAllNativeTextInput(inputId);
  }

  copyNativeTextInput(inputId: string): void {
    this.nativeTextInputHost?.copyNativeTextInput(inputId);
  }

  cutNativeTextInput(inputId: string): void {
    this.nativeTextInputHost?.cutNativeTextInput(inputId);
  }

  pasteNativeTextInput(inputId: string): void {
    this.nativeTextInputHost?.pasteNativeTextInput(inputId);
  }

  closeNativeTextInput(inputId: string): void {
    this.nativeTextInputHost?.closeNativeTextInput(inputId);
  }

  private async showTextInput(): Promise<void> {
    this.listenKeyboardHeightChange();
    await this.attach(true);
      if (this.imcFlag != true) {
      this.listenKeyBoardEvent();
    }
    this.inputMethodController.showTextInput().then(() => {
      console.debug("Succeeded in showing softKeyboard");
    }).catch((err: Any) => {
      console.error("Failed to show softKeyboard:" + JSON.stringify(err));
    });
  }

  private async hideTextInput(): Promise<void> {
    this.inputMethodController.hideTextInput().then(() => {
      console.debug("Succeeded in hide softKeyboard");
    }).catch((err: Any) => {
      console.error("Failed to hide softKeyboard:" + JSON.stringify(err));
    })
    this.cancelKeyboardHeightChange();
  }

  private async attach(showKeyboard: boolean): Promise<void> {
    try {
      await this.inputMethodController.attach(showKeyboard, this.textConfig);
    } catch (err) {
      console.error("Failed to attach:" + JSON.stringify(err));
    }
  }

  private listenKeyBoardEvent(): void {
    try {
      this.inputMethodController.on('insertText', (text) => {
      this.xComponentContext?.handleInsertTextEvent(text);
      console.debug("insertText: " + text);
    });
    } catch (err) {
      console.error("Failed to subscribe insertText:" + JSON.stringify(err));
      this.cancelListenKeyBoardEvent();
      return;
    }

    try {
      this.inputMethodController.on('deleteLeft', (length) => {
        this.xComponentContext?.handleDeleteLeftEvent(length);
        console.debug("deleteLeft, length:" + JSON.stringify(length));
      })
    } catch (err) {
      console.error("Failed to subscribe deleteLeft:" + JSON.stringify(err));
      this.cancelListenKeyBoardEvent();
      return;
    }

    try {
      this.inputMethodController.on('sendKeyboardStatus', (state) => {
        if (state == inputMethod.KeyboardStatus.HIDE) {
          this.xComponentContext?.handleKeyboardStatus(false, 0);
          this.inputMethodController.detach();
          this.cancelListenKeyBoardEvent();
        }
        console.debug("sendKeyboardStatus, state:" + JSON.stringify(state));
      })
    } catch (err) {
      console.error("Failed to subscribe sendKeyboardStatus:" + JSON.stringify(err));
      this.cancelListenKeyBoardEvent();
      return;
    }

    console.debug("listenKeyBoardEvent success");
    this.imcFlag = true;
  }

  private cancelListenKeyBoardEvent(): void {
    this.inputMethodController.off('insertText');
    this.inputMethodController.off('deleteLeft');
    this.inputMethodController.off('sendKeyboardStatus');
    this.imcFlag = false;
  }

  private listenKeyboardHeightChange() {
    window.getLastWindow(getContext(this)).then(currentWindow => {
      try {
        currentWindow.on('keyboardHeightChange', (data) => {
          console.info('Succeeded in enabling the listener for keyboard height changes. Data: ' + JSON.stringify(data));
          if (data != 0) {
            this.xComponentContext?.handleKeyboardStatus(true,data);
          }
        });
      } catch (exception) {
        console.error(`Failed to enable the listener for keyboard height changes. Cause code: ${exception.code}, message: ${exception.message}`);
      }
    })
  }

  private cancelKeyboardHeightChange() {
    window.getLastWindow(getContext(this)).then(currentWindow => {
      try {
        currentWindow.off('keyboardHeightChange');
      } catch (exception) {
        console.error(`Failed to enable the listener for keyboard height changes. Cause code: ${exception.code}, message: ${exception.message}`);
      }
    })
  }

}

export interface XComponentAttrs {
  id: string;
  type: number;
  libraryname: string;
}
