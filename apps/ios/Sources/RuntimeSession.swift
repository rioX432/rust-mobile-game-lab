import QuartzCore
import Foundation

// UIKit and the polling timer serialize every call on the main thread. Rust owns GPU work.
final class RuntimeSession {
    let generation = UUID()
    var onStatus: ((UUID, String) -> Void)?
    private var handle: UInt64
    private var timer: Timer?
    private var closing = false
    private var closeDetached = false
    private var active = false
    private var desiredLayer: CAMetalLayer?
    private var desiredMetrics: SurfaceMetrics?
    private var attachedLayer: CAMetalLayer?
    private var attachedMetrics: SurfaceMetrics?
    private var paused: Bool?
    private var lastError: String?

    init() {
        precondition(Thread.isMainThread)
        handle = mg_create(42)
        NSLog("[iOS runtime %@] create handle=%llu", generation.uuidString, handle)
        if handle == 0 { lastError = "mg_create returned zero" }
    }

    func start() {
        guard handle != 0 else {
            onStatus?(generation, "mg_create returned zero")
            return
        }
        guard timer == nil else { return }
        // A pending close retains this session until detach/destroy accept the bounded work.
        let timer = Timer(timeInterval: 0.1, repeats: true) { [self] _ in reconcile() }
        self.timer = timer
        RunLoop.main.add(timer, forMode: .common)
        reconcile()
    }

    func setSurface(_ layer: CAMetalLayer?, metrics: SurfaceMetrics?) {
        guard !closing else { return }
        desiredLayer = layer
        desiredMetrics = metrics
        reconcile()
    }

    func setActive(_ active: Bool) {
        guard !closing else { return }
        self.active = active
        reconcile()
    }

    func touch(x: Float, y: Float) {
        guard !closing, active, attachedLayer != nil, attachedLayer === desiredLayer,
              attachedMetrics == desiredMetrics, handle != 0 else { return }
        _ = accept(mg_touch(handle, x, y), operation: "touch")
    }

    func reset() {
        action(0)
    }

    func selectMode(race: Bool) { action(1, argument: race ? 1 : 0) }
    func startRound() { action(2) }
    func closeRound() { action(4) }

    func nativeChoose(_ option: UInt32) {
        guard active, attachedLayer != nil, attachedLayer === desiredLayer,
              attachedMetrics == desiredMetrics else { return }
        action(3, argument: option)
    }

    private func action(_ action: UInt32, argument: UInt32 = 0) {
        guard !closing, handle != 0 else { return }
        _ = accept(mg_action(handle, action, argument), operation: "action:\(action)")
        publishStatus()
    }

    func close() {
        precondition(Thread.isMainThread)
        guard !closing else { return }
        closing = true
        onStatus = nil
        desiredLayer = nil
        desiredMetrics = nil
        reconcile()
    }

    private func reconcile() {
        precondition(Thread.isMainThread)
        guard handle != 0 else { return }
        if closing {
            if !closeDetached {
                guard accept(mg_detach(handle), operation: "detach(close)") else { return }
                closeDetached = true
                attachedLayer = nil
                attachedMetrics = nil
                NSLog("[iOS runtime %@] detach accepted", generation.uuidString)
            }
            guard accept(mg_destroy(handle), operation: "destroy") else { return }
            NSLog("[iOS runtime %@] destroy accepted handle=%llu", generation.uuidString, handle)
            handle = 0
            timer?.invalidate()
            timer = nil
            return
        }

        let wantedLayer = active ? desiredLayer : nil
        let usable = wantedLayer != nil && (desiredMetrics?.width ?? 0) > 0
            && (desiredMetrics?.height ?? 0) > 0
        if attachedLayer != nil && (!usable || attachedLayer !== wantedLayer) {
            guard accept(mg_detach(handle), operation: "detach") else { publishStatus(); return }
            attachedLayer = nil
            attachedMetrics = nil
            NSLog("[iOS runtime %@] detach accepted", generation.uuidString)
        }
        if usable, let layer = wantedLayer, let metrics = desiredMetrics {
            if attachedLayer == nil {
                let pointer = Unmanaged.passUnretained(layer).toOpaque()
                guard accept(mg_attach_metal(handle, pointer, metrics.width, metrics.height, metrics.scale),
                             operation: "attach") else { publishStatus(); return }
                attachedLayer = layer
                attachedMetrics = metrics
                NSLog("[iOS runtime %@] attach accepted %ux%u scale=%.2f", generation.uuidString,
                      metrics.width, metrics.height, metrics.scale)
            } else if attachedMetrics != metrics {
                guard accept(mg_resize(handle, metrics.width, metrics.height, metrics.scale),
                             operation: "resize") else { publishStatus(); return }
                attachedMetrics = metrics
                NSLog("[iOS runtime %@] resize accepted %ux%u scale=%.2f", generation.uuidString,
                      metrics.width, metrics.height, metrics.scale)
            }
        }
        if paused != !active {
            if accept(mg_pause(handle, active ? 0 : 1), operation: "pause") { paused = !active }
        }
        publishStatus()
    }

    private func accept(_ result: Int32, operation: String) -> Bool {
        guard result >= 0 else {
            let error = "\(operation): \(result)"
            if lastError != error { NSLog("[iOS runtime %@] %@", generation.uuidString, error) }
            lastError = error
            return false
        }
        lastError = nil
        return true
    }

    private func publishStatus() {
        guard !closing, handle != 0 else { return }
        var buffer = [CChar](repeating: 0, count: 16 * 1024)
        let count = buffer.withUnsafeMutableBufferPointer {
            mg_status(handle, $0.baseAddress, UInt32($0.count))
        }
        let status: String
        if count >= 0, Int(count) < buffer.count, buffer[Int(count)] == 0 {
            status = String(decoding: buffer.prefix(Int(count)).map { UInt8(bitPattern: $0) }, as: UTF8.self)
        } else {
            status = "mg_status failed or invalid length: \(count)"
        }
        onStatus?(generation, RuntimeStatus.readable(status)
                  + (lastError.map { "\nHost ABI error: \($0)" } ?? ""))
    }
}
