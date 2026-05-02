import AppKit
import ObjectiveC.runtime

func probeClass(_ name: String) -> AnyClass? {
    let cls: AnyClass? = NSClassFromString(name)
    print("[liquid-glass-probe] class \(name) exists=\(cls != nil)")
    return cls
}

func probeSelector(_ cls: AnyClass?, _ selectorName: String) -> Bool {
    guard let cls else {
        print("[liquid-glass-probe] selector \(selectorName) exists=false reason=class-missing")
        return false
    }
    let sel = NSSelectorFromString(selectorName)
    let exists = class_getInstanceMethod(cls, sel) != nil
    print("[liquid-glass-probe] selector \(selectorName) exists=\(exists)")
    return exists
}

func makeView(_ cls: AnyClass?, frame: NSRect, name: String) -> NSView? {
    guard let cls = cls as? NSView.Type else {
        print("[liquid-glass-probe] construct \(name) ok=false reason=class-not-nsview")
        return nil
    }
    let view = cls.init(frame: frame)
    print("[liquid-glass-probe] construct \(name) ok=true frame=\(frame)")
    return view
}

func sendInt(_ object: AnyObject, selectorName: String, value: Int) {
    let selector = NSSelectorFromString(selectorName)
    guard object.responds(to: selector) else {
        print("[liquid-glass-probe] send \(selectorName) ok=false reason=missing-selector")
        return
    }
    typealias Fn = @convention(c) (AnyObject, Selector, Int) -> Void
    let imp = object.method(for: selector)
    let fn = unsafeBitCast(imp, to: Fn.self)
    fn(object, selector, value)
    print("[liquid-glass-probe] send \(selectorName) ok=true value=\(value)")
}

func sendDouble(_ object: AnyObject, selectorName: String, value: Double) {
    let selector = NSSelectorFromString(selectorName)
    guard object.responds(to: selector) else {
        print("[liquid-glass-probe] send \(selectorName) ok=false reason=missing-selector")
        return
    }
    typealias Fn = @convention(c) (AnyObject, Selector, Double) -> Void
    let imp = object.method(for: selector)
    let fn = unsafeBitCast(imp, to: Fn.self)
    fn(object, selector, value)
    print("[liquid-glass-probe] send \(selectorName) ok=true value=\(value)")
}

func sendObject(_ object: AnyObject, selectorName: String, value: AnyObject) {
    let selector = NSSelectorFromString(selectorName)
    guard object.responds(to: selector) else {
        print("[liquid-glass-probe] send \(selectorName) ok=false reason=missing-selector")
        return
    }
    typealias Fn = @convention(c) (AnyObject, Selector, AnyObject) -> Void
    let imp = object.method(for: selector)
    let fn = unsafeBitCast(imp, to: Fn.self)
    fn(object, selector, value)
    print("[liquid-glass-probe] send \(selectorName) ok=true value=\(type(of: value))")
}

let glassClass: AnyClass? = probeClass("NSGlassEffectView")
let containerClass: AnyClass? = probeClass("NSGlassEffectContainerView")

for selector in ["initWithFrame:", "setStyle:", "setTintColor:", "setCornerRadius:", "setContentView:"] {
    _ = probeSelector(glassClass, selector)
}

for selector in ["initWithFrame:", "setSpacing:"] {
    _ = probeSelector(containerClass, selector)
}

print("[liquid-glass-probe] style regular raw=0 expected-from-v3")
print("[liquid-glass-probe] style clear raw=1 expected-from-v3")

let rootFrame = NSRect(x: 0, y: 0, width: 640, height: 420)
let panelFrames = [
    NSRect(x: 24, y: 24, width: 180, height: 360),
    NSRect(x: 228, y: 24, width: 388, height: 260),
    NSRect(x: 228, y: 304, width: 388, height: 92),
]

if let container = makeView(containerClass, frame: rootFrame, name: "NSGlassEffectContainerView") {
    sendDouble(container, selectorName: "setSpacing:", value: 20.0)
    container.wantsLayer = true
    container.layer?.masksToBounds = false

    var installedPanels = 0
    for (index, frame) in panelFrames.enumerated() {
        guard let panel = makeView(glassClass, frame: frame, name: "NSGlassEffectView[\(index)]") else {
            continue
        }
        panel.wantsLayer = true
        panel.layer?.masksToBounds = true
        let radius = min(frame.width, frame.height) / 2.0
        panel.layer?.cornerRadius = radius
        sendDouble(panel, selectorName: "setCornerRadius:", value: radius)
        sendInt(panel, selectorName: "setStyle:", value: index == 0 ? 0 : 1)
        sendObject(
            panel,
            selectorName: "setTintColor:",
            value: NSColor(srgbRed: 0.90, green: 0.95, blue: 1.0, alpha: 0.16)
        )
        container.addSubview(panel)
        installedPanels += 1
    }

    print("[liquid-glass-probe] native-container construct ok=true panels=\(installedPanels)")
} else {
    print("[liquid-glass-probe] native-container construct ok=false panels=0")
}
