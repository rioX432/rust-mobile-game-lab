package lab.rustmobile

import android.view.Surface

internal object AndroidRuntime : ProbeRuntime<Surface> {
    override fun create(seed: Long) = NativeBridge.create(seed)
    override fun attach(handle: Long, surface: Surface, width: Int, height: Int, scale: Float) =
        NativeBridge.attach(handle, surface, width, height, scale)
    override fun resize(handle: Long, width: Int, height: Int, scale: Float) =
        NativeBridge.resize(handle, width, height, scale)
    override fun pause(handle: Long, paused: Boolean) = NativeBridge.pause(handle, paused)
    override fun detach(handle: Long) = NativeBridge.detach(handle)
    override fun destroy(handle: Long) = NativeBridge.destroy(handle)
    override fun touch(handle: Long, x: Float, y: Float) = NativeBridge.touch(handle, x, y)
    override fun reset(handle: Long) = NativeBridge.action(handle, 0, 0)
    override fun status(handle: Long) = NativeBridge.status(handle)
}
