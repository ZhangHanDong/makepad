# Step 144 - macOS Native Button Glass Bezel

Date: 2026-05-10

## Change

The macOS native-control installer now configures installed `NativeGlassButton`
mirrors with the macOS 26 glass button bezel raw value:

```text
NSButton.BezelStyle.glass = 16
```

The local `MacOSX15.5.sdk` does not expose the typed Swift/AppKit enum case, so
the installer uses the raw value dynamically through `setBezelStyle:`. A probe
override is available for SDK/runtime validation:

```text
MAKEPAD_NATIVE_GLASS_BUTTON_BEZEL_RAW=<raw>
```

When a native button is installed and the selector exists, the platform logs:

```text
[liquid-glass] backend=apple-native-controls button-style control=... label="Clear" bezel=glass raw=16
```

This keeps native button glass separate from passive
`NativeGlassPanelDescriptor` surfaces. It applies only to explicit native
control descriptors.

## Verification

Passed:

```text
cargo test -p makepad-platform native_glass_button --release
cargo check -p makepad-platform --release
git diff --check
```

Studio release run:

```text
RunItem makepad-example-aichat-macos-native-clear-control-probe
build_id=[172]
```

Runtime evidence:

```text
[liquid-glass] backend=apple-native-controls button-style control=0000000000000043 label="Clear" bezel=glass raw=16
[liquid-glass] backend=apple-native-controls button-style control=0000000000000044 label="↑" bezel=glass raw=16
[liquid-glass] backend=apple-native-controls state=Installed reason=installed-appkit-buttons controls_total=2 controls_visible=2
```

## Verdict

This closes the first macOS native-button styling gap for the installer path:
AppKit button mirrors now request the glass bezel when runtime validation
enables native controls.

It does not prove physical/system click delivery, UIKit button configuration,
or accessibility behavior.
