Status: PASS
RunItem: makepad-example-native-text-input
Example: examples/native_text_input
Verified: focus blur set_text selection clipboard native_label shared_host clipped_widget
NeedsManualValidation: changed_event_from_native_typing

Studio: 127.0.0.1:8001
BuildId: 8
Screenshot: /var/folders/q9/w05jvn_11tdblq6rzg5p_0t00000gn/T/makepad_studio_hub/build-8-kind-0-req-65-1779141801150.png
ClippedScreenshot: /var/folders/q9/w05jvn_11tdblq6rzg5p_0t00000gn/T/makepad_studio_hub/build-8-kind-0-req-497-1779141802512.png
Transcript: docs/native-textinput-evidence/macos-studio-runtime.log
Command: tools/native_textinput_macos_studio_smoke.sh --clear-build-id 7 --timeout 180

Smoke sequence:
- WidgetQuery id:native_input
- Click focus_button
- QueryLogs focus requested
- Click native_input
- Click set_button
- QueryLogs text set
- Click select_button
- QueryLogs select all requested
- Click copy_button
- Click cut_button
- Click paste_button
- QueryLogs paste requested
- Click label_button
- QueryLogs native label updated
- Click set_both_button
- QueryLogs same-frame native updates requested
- WidgetQuery id:clipped_native_input
- Screenshot clipped host state
- Click blur_button
- QueryLogs blur requested
