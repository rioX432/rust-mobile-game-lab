import QuartzCore
import UIKit

struct SurfaceMetrics: Equatable {
    let width: UInt32
    let height: UInt32
    let scale: Float
}

final class MetalHostView: UIView {
    override class var layerClass: AnyClass { CAMetalLayer.self }
    var onSurfaceChange: ((CAMetalLayer?, SurfaceMetrics?) -> Void)?
    var onTouch: ((Float, Float) -> Void)?

    private var metalLayer: CAMetalLayer { layer as! CAMetalLayer }

    override init(frame: CGRect) {
        super.init(frame: frame)
        isMultipleTouchEnabled = false
        isAccessibilityElement = true
        accessibilityLabel = "Rust drawing area"
        metalLayer.isOpaque = true
    }

    required init?(coder: NSCoder) { fatalError("init(coder:) has not been implemented") }

    override func layoutSubviews() {
        super.layoutSubviews()
        updateSurface()
    }

    override func didMoveToWindow() {
        super.didMoveToWindow()
        updateSurface()
    }

    override func traitCollectionDidChange(_ previousTraitCollection: UITraitCollection?) {
        super.traitCollectionDidChange(previousTraitCollection)
        updateSurface()
    }

    private func updateSurface() {
        guard let window else {
            onSurfaceChange?(nil, nil)
            return
        }
        let scale = window.screen.scale
        contentScaleFactor = scale
        metalLayer.contentsScale = scale
        let width = max(0, (bounds.width * scale).rounded())
        let height = max(0, (bounds.height * scale).rounded())
        guard width.isFinite, height.isFinite,
              width <= CGFloat(UInt32.max), height <= CGFloat(UInt32.max) else { return }
        metalLayer.drawableSize = CGSize(width: width, height: height)
        let metrics = SurfaceMetrics(width: UInt32(width), height: UInt32(height), scale: Float(scale))
        onSurfaceChange?(metalLayer, metrics)
    }

    override func touchesBegan(_ touches: Set<UITouch>, with event: UIEvent?) {
        guard let touch = touches.first, window != nil else { return }
        let point = touch.location(in: self)
        guard bounds.contains(point) else { return }
        onTouch?(Float(point.x * contentScaleFactor), Float(point.y * contentScaleFactor))
    }
}
