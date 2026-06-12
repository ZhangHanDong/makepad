import display from '@ohos.display';
import http from '@ohos.net.http';
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
  handleHttpHeaders(requestId: string, statusCode: number, headersFlat: string): void;
  handleHttpChunk(requestId: string, data: ArrayBuffer): void;
  handleHttpComplete(requestId: string): void;
  handleHttpError(requestId: string, message: string): void;
}

@Observed
export class NativeLabelState {
  inputId: string = '';
  text: string = '';
  left: number = 0;
  top: number = 0;
  width: number = 1;
  height: number = 1;
  visible: boolean = false;
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
  createNativeLabel(inputId: string, text: string): void;
  updateNativeLabel(inputId: string, left: number, top: number, width: number, height: number, visible: boolean): void;
  setNativeLabelText(inputId: string, text: string): void;
  detachNativeLabel(inputId: string): void;
  closeNativeLabel(inputId: string): void;
}

// @Observed + @ObjectLink is required: the overlay list renders through
// ForEach keyed by inputId, and ArkUI reuses (does not rebuild) an item whose
// key is unchanged. With a plain interface, programmatic state changes such
// as set_text update the array but never reach the on-screen TextInput.
@Observed
export class NativeTextInputState {
  inputId: string = '';
  text: string = '';
  placeholder: string = '';
  editable: boolean = true;
  left: number = 0;
  top: number = 0;
  width: number = 1;
  height: number = 1;
  visible: boolean = false;
  // ArkUI may or may not fire onChange for a programmatic text update
  // depending on version, so a boolean flag cannot pair sets with echoes.
  // Each programmatic set increments the echo count; onChange only swallows
  // a callback when the count is positive AND the value matches the last
  // programmatic text, so stale echoes can never eat newer user input.
  pendingProgrammaticEchoes: number = 0;
  lastProgrammaticText: string = '';
  // Commands arriving between state insertion and ArkUI component attach
  // (same Rust op batch) must be queued; onAppear flips this and flushes.
  attached: boolean = false;
  selectionStart: number = 0;
  selectionEnd: number = 0;
  controller: TextInputController = new TextInputController();
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

  createNativeLabel(inputId: string, text: string): void {
    this.nativeTextInputHost?.createNativeLabel(inputId, text);
  }

  updateNativeLabel(inputId: string, left: number, top: number, width: number, height: number, visible: boolean): void {
    this.nativeTextInputHost?.updateNativeLabel(inputId, left, top, width, height, visible);
  }

  setNativeLabelText(inputId: string, text: string): void {
    this.nativeTextInputHost?.setNativeLabelText(inputId, text);
  }

  detachNativeLabel(inputId: string): void {
    this.nativeTextInputHost?.detachNativeLabel(inputId);
  }

  closeNativeLabel(inputId: string): void {
    this.nativeTextInputHost?.closeNativeLabel(inputId);
  }

  private httpRequests: Map<string, http.HttpRequest> = new Map<string, http.HttpRequest>();

  private httpMethod(method: string): http.RequestMethod {
    if (method === 'POST') {
      return http.RequestMethod.POST;
    }
    if (method === 'PUT') {
      return http.RequestMethod.PUT;
    }
    if (method === 'DELETE') {
      return http.RequestMethod.DELETE;
    }
    if (method === 'HEAD') {
      return http.RequestMethod.HEAD;
    }
    if (method === 'OPTIONS') {
      return http.RequestMethod.OPTIONS;
    }
    if (method === 'TRACE') {
      return http.RequestMethod.TRACE;
    }
    if (method === 'CONNECT') {
      return http.RequestMethod.CONNECT;
    }
    return http.RequestMethod.GET;
  }

  // Headers cross the bridge as newline-delimited key/value pairs to avoid a
  // JSON dependency on the Rust side.
  private parseFlatHeaders(flat: string): Record<string, string> {
    let header: Record<string, string> = {};
    let lines = flat.split('\n');
    for (let i = 0; i + 1 < lines.length; i += 2) {
      if (lines[i].length > 0) {
        header[lines[i]] = lines[i + 1];
      }
    }
    return header;
  }

  private flattenHeaders(headers: Object): string {
    let flat = '';
    let record = headers as Record<string, Object>;
    for (let key of Object.keys(record)) {
      flat += key + '\n' + String(record[key]) + '\n';
    }
    return flat;
  }

  private finishHttpRequest(requestId: string): void {
    let request = this.httpRequests.get(requestId);
    if (request !== undefined) {
      this.httpRequests.delete(requestId);
      request.destroy();
    }
  }

  httpStart(requestId: string, method: string, url: string, headersFlat: string, body: ArrayBuffer): void {
    console.info('[MakepadNet] httpStart id=' + requestId + ' method=' + method + ' url=' + url + ' body_len=' + body.byteLength);
    let request = http.createHttp();
    this.httpRequests.set(requestId, request);
    let headersStash = '';
    // Observed @ohos.net.http ordering: headersReceive -> dataReceive ->
    // dataEnd -> response-code callback (last). Completion must wait for
    // both dataEnd and the status code, or the Rust side would finish the
    // request with status 0 and drop the late headers.
    let dataEnded = false;
    let gotCode = false;
    let maybeComplete = (): void => {
      if (dataEnded && gotCode) {
        console.info('[MakepadNet] complete id=' + requestId);
        this.xComponentContext?.handleHttpComplete(requestId);
        this.finishHttpRequest(requestId);
      }
    };
    request.on('headersReceive', (header: Object) => {
      headersStash = this.flattenHeaders(header);
      console.info('[MakepadNet] headersReceive id=' + requestId);
    });
    request.on('dataReceive', (data: ArrayBuffer) => {
      console.info('[MakepadNet] dataReceive id=' + requestId + ' len=' + data.byteLength);
      this.xComponentContext?.handleHttpChunk(requestId, data);
    });
    request.on('dataEnd', () => {
      console.info('[MakepadNet] dataEnd id=' + requestId);
      dataEnded = true;
      maybeComplete();
    });
    let options: http.HttpRequestOptions = {
      method: this.httpMethod(method),
      header: this.parseFlatHeaders(headersFlat),
    };
    if (body.byteLength > 0) {
      options.extraData = body;
    }
    try {
      request.requestInStream(url, options, (err: Error, responseCode: number) => {
        if (err) {
          console.error('[MakepadNet] request error id=' + requestId + ' err=' + JSON.stringify(err));
          this.xComponentContext?.handleHttpError(requestId, JSON.stringify(err));
          this.finishHttpRequest(requestId);
          return;
        }
        console.info('[MakepadNet] response id=' + requestId + ' code=' + responseCode);
        this.xComponentContext?.handleHttpHeaders(requestId, responseCode, headersStash);
        gotCode = true;
        maybeComplete();
      });
    } catch (err) {
      console.error('[MakepadNet] requestInStream threw id=' + requestId + ' err=' + JSON.stringify(err));
      this.xComponentContext?.handleHttpError(requestId, JSON.stringify(err));
      this.finishHttpRequest(requestId);
    }
  }

  httpCancel(requestId: string): void {
    this.finishHttpRequest(requestId);
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
