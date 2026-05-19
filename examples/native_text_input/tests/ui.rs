use makepad_test::{makepad_test, Selector, TestApp};

#[makepad_test]
fn native_text_input_smoke_controls_are_visible(app: TestApp) {
    app.locator(Selector::id("native_input")).wait_visible();
    app.locator(Selector::id("native_label")).wait_visible();
    app.locator(Selector::id("clipped_native_input"))
        .wait_visible();
    app.locator(Selector::id("focus_button")).wait_visible();
    app.locator(Selector::id("set_button")).wait_visible();
    app.locator(Selector::id("blur_button")).wait_visible();
    app.locator(Selector::id("select_button")).wait_visible();
    app.locator(Selector::id("copy_button")).wait_visible();
    app.locator(Selector::id("cut_button")).wait_visible();
    app.locator(Selector::id("paste_button")).wait_visible();
}

#[makepad_test]
fn native_text_input_buttons_update_status_labels(app: TestApp) {
    app.locator(Selector::id("set_button")).click();
    app.locator(Selector::id("status_label"))
        .wait_text("Status: text set");
    app.locator(Selector::id("native_input"))
        .wait_value("hello native");
    app.locator(Selector::id("value_label"))
        .wait_text("Value: hello native");

    app.locator(Selector::id("focus_button")).click();
    app.locator(Selector::id("status_label"))
        .wait_text("Status: focus requested");

    app.locator(Selector::id("blur_button")).click();
    app.locator(Selector::id("status_label"))
        .wait_text("Status: blur requested");
}

#[makepad_test]
fn native_text_input_command_buttons_update_status_labels(app: TestApp) {
    app.locator(Selector::id("select_button")).click();
    app.locator(Selector::id("status_label"))
        .wait_text("Status: select all requested");

    app.locator(Selector::id("copy_button")).click();
    app.locator(Selector::id("status_label"))
        .wait_text("Status: copy requested");

    app.locator(Selector::id("cut_button")).click();
    app.locator(Selector::id("status_label"))
        .wait_text("Status: cut requested");

    app.locator(Selector::id("paste_button")).click();
    app.locator(Selector::id("status_label"))
        .wait_text("Status: paste requested");
}

#[makepad_test]
fn native_label_update_path_is_drivable(app: TestApp) {
    app.locator(Selector::id("label_button")).click();
    app.locator(Selector::id("native_label"))
        .wait_text("NativeLabel updated through shared host registry");
    app.locator(Selector::id("status_label"))
        .wait_text("Status: native label updated");
}

#[makepad_test]
fn native_text_input_same_frame_update_path_is_drivable(app: TestApp) {
    app.locator(Selector::id("set_both_button")).click();
    app.locator(Selector::id("status_label"))
        .wait_text("Status: same-frame native updates requested");
    app.locator(Selector::id("native_input"))
        .wait_value("same frame native input");
    app.locator(Selector::id("native_label"))
        .wait_text("NativeLabel updated in same frame");
    app.locator(Selector::id("value_label"))
        .wait_text("Value: same frame native input");
}
