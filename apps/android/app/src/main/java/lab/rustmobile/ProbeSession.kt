package lab.rustmobile

internal interface ProbeRuntime<S> {
    fun create(seed: Long): Long
    fun attach(handle: Long, surface: S, width: Int, height: Int, scale: Float): Int
    fun resize(handle: Long, width: Int, height: Int, scale: Float): Int
    fun pause(handle: Long, paused: Boolean): Int
    fun detach(handle: Long): Int
    fun destroy(handle: Long): Int
    fun touch(handle: Long, x: Float, y: Float): Int
    fun reset(handle: Long): Int
    fun action(handle: Long, action: Int, argument: Int): Int
    fun status(handle: Long): String
}

// Each screen owns one session. All calls, including Surface callbacks, run on the UI thread.
internal class ProbeSession<S>(
    private val runtime: ProbeRuntime<S>,
    seed: Long,
    private val report: (String) -> Unit,
) {
    private val handle = runtime.create(seed).also { check(it != 0L) { "create returned an invalid handle" } }
    private var closed = false
    private var attached = false
    private var paused = true
    private var surfacePresent = false
    var generation: Long = 0
        private set

    init { checked("pause", runtime.pause(handle, true)) }

    fun surfaceCreated() {
        if (closed) return
        check(!surfacePresent)
        surfacePresent = true
        generation++
        report("surfaceCreated handle=$handle generation=$generation")
    }

    fun surfaceChanged(surface: S, width: Int, height: Int, scale: Float) {
        if (closed || !surfacePresent) return
        if (attached) {
            checked("resize", runtime.resize(handle, width, height, scale))
        } else if (width > 0 && height > 0) {
            val result = runtime.attach(handle, surface, width, height, scale)
            checked("attach", result)
            attached = result >= 0
            if (attached) checked("pause", runtime.pause(handle, paused))
        }
    }

    fun surfaceDestroyed() {
        if (closed || !surfacePresent) return
        surfacePresent = false
        generation++
        detach()
    }

    fun setPaused(value: Boolean) {
        if (closed) return
        paused = value
        checked("pause", runtime.pause(handle, value))
    }

    fun touch(x: Float, y: Float) {
        if (!closed && attached && !paused) checked("touch", runtime.touch(handle, x, y))
    }

    fun reset() {
        if (!closed) checked("reset", runtime.reset(handle))
    }

    fun selectMode(race: Boolean) = action(1, if (race) 1 else 0)
    fun startRound() = action(2, 0)
    fun nativeChoose(option: Int) {
        if (attached && !paused) action(3, option)
    }
    fun closeRound() = action(4, 0)

    private fun action(action: Int, argument: Int) {
        if (!closed) checked("action:$action", runtime.action(handle, action, argument))
    }

    fun status(expectedGeneration: Long): String? =
        if (closed || generation != expectedGeneration) null else runtime.status(handle)

    fun close() {
        if (closed) return
        closed = true
        generation++
        try {
            detach()
        } finally {
            val result = runtime.destroy(handle)
            report("destroy handle=$handle result=$result")
            check(result >= 0) { "destroy failed: $result" }
        }
    }

    private fun detach() {
        if (!attached) return
        val result = runtime.detach(handle)
        report("detach handle=$handle generation=$generation result=$result")
        // Returning while Rust can still draw would violate SurfaceHolder.Callback's lifetime.
        check(result >= 0) { "detach failed: $result; Rust must synchronously invalidate drawing" }
        attached = false
    }

    private fun checked(operation: String, result: Int) {
        if (result < 0) report("$operation handle=$handle result=$result")
    }
}
