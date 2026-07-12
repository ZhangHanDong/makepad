Status: PASS
RunItem: makepad-example-native-text-input-macos-standalone-diag
Example: examples/native_text_input
Verified: ClearBuild RunItem detach leak

Studio: 127.0.0.1:8001
BuildId: 8
Transcript: docs/native-textinput-evidence/macos-leak-runtime.log
Command: Studio bridge RunItem makepad-example-native-text-input-macos-standalone-diag

Evidence:
- Studio `RunItem` launched the standalone macOS diagnostic release build.
- QueryLogs recorded `NativeTextInput live count: 1`, `2`, and `3` after startup.
- The diagnostic path requested close for primary, secondary, and clipped native inputs plus the native label.
- QueryLogs recorded `NativeTextInput live count: 2`, `1`, and `0` before process exit.
- The build stopped with exit code 0 after the diagnostic quit request.
