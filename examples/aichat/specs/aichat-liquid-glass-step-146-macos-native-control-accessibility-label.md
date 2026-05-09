# Step 146 - macOS Native Control Accessibility Label

Date: 2026-05-10

## Change

The macOS native-control installer now copies
`NativeGlassControlDescriptor.label` into the installed `NSButton` mirror's
accessibility label with `setAccessibilityLabel:`.

When the selector exists, the platform logs:

```text
[liquid-glass] backend=apple-native-controls accessibility-label control=... label="Clear"
```

This gives explicit evidence that the platform control receives the same
semantic label Makepad uses for action dispatch and diagnostics.

## Verification

Passed:

```text
cargo test -p makepad-platform native_glass_control_accessibility --release
cargo check -p makepad-platform --release
git diff --check
```

## Verdict

This closes the first macOS native-control accessibility metadata slice: labels
are explicitly mirrored into AppKit controls.

It does not complete accessibility ownership. Focus traversal, VoiceOver
behavior, duplicate labels between native and Makepad-rendered controls, and
UIKit accessibility remain open.
