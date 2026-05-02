import UIKit

func makeGlassHierarchy(containerRect: CGRect, panelRects: [CGRect]) -> UIView {
    let containerEffect = UIGlassContainerEffect()
    let container = UIVisualEffectView(effect: containerEffect)
    container.frame = containerRect
    container.isUserInteractionEnabled = false

    for rect in panelRects {
        let panelEffect = UIGlassEffect(style: .regular)
        let panel = UIVisualEffectView(effect: panelEffect)
        panel.frame = rect
        panel.layer.cornerRadius = min(rect.width, rect.height) / 2
        panel.layer.masksToBounds = true
        panel.isUserInteractionEnabled = false
        container.contentView.addSubview(panel)
    }

    return container
}

_ = makeGlassHierarchy(
    containerRect: CGRect(x: 0, y: 0, width: 640, height: 420),
    panelRects: [
        CGRect(x: 24, y: 24, width: 180, height: 360),
        CGRect(x: 228, y: 24, width: 388, height: 260),
        CGRect(x: 228, y: 304, width: 388, height: 92),
    ]
)
