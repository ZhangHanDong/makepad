# macOS NativeTextInput Manual Runtime Checklist

Use Studio RunItem `makepad-example-native-text-input-macos-standalone` for
manual AppKit validation. This checklist covers visual and human input behavior
that static tests and bridge log queries cannot prove.

## Required Before Closing P0B

- [ ] Click the primary native input and type ASCII text; the field text changes
      in place.
- [ ] The `Value:` label updates through `NativeTextInputAction::Changed`.
- [ ] Programmatic `Primary Set` updates the primary field without adding an
      extra changed-event count.
- [ ] `Primary Focus` gives keyboard focus to the primary field.
- [ ] `Primary Blur` removes first-responder focus; it does not clear text.
- [ ] `Primary Select`, `Primary Copy`, `Primary Cut`, and `Primary Paste`
      operate on the primary field.
- [ ] Repeat focus, set, select, copy, cut, paste, and blur on the secondary
      field.
- [ ] Close/relaunch the RunItem; no stale `NSTextField` remains from the
      previous run.

## Required Before Closing P1/P1.5/P4

- [ ] `Set Label` changes the native label, proving `NativeLabel` uses the
      shared host registry.
- [ ] `Set Both` changes primary input, secondary input, and native label in one
      interaction.
- [ ] The clipped native input stays within the composition/clipping panel.
- [ ] Scrolling the panel does not leave a native field floating outside its
      Makepad container.
- [ ] Multiple native views keep the expected z-order and do not cover the
      regular Makepad buttons or labels.
