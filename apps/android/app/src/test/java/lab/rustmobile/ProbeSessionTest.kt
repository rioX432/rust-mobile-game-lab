package lab.rustmobile

import org.junit.Assert.*
import org.junit.Test

class ProbeSessionTest {
    private class RecordingRuntime : ProbeRuntime<String> {
        var nextHandle = 0L
        val live = mutableSetOf<Long>()
        val attached = mutableSetOf<Long>()
        val events = mutableListOf<String>()
        var detachResult = 0
        override fun create(seed: Long) = (++nextHandle).also { live += it; events += "create:$it" }
        override fun attach(handle: Long, surface: String, width: Int, height: Int, scale: Float): Int {
            check(handle in live)
            attached += handle
            events += "attach:$handle:$surface:$width:$height:$scale"
            return 0
        }
        override fun resize(handle: Long, width: Int, height: Int, scale: Float): Int {
            events += "resize:$handle:$width:$height:$scale"
            return 0
        }
        override fun pause(handle: Long, paused: Boolean): Int { events += "pause:$handle:$paused"; return 0 }
        override fun detach(handle: Long): Int {
            events += "detach:$handle"
            if (detachResult >= 0) attached -= handle
            return detachResult
        }
        override fun destroy(handle: Long): Int {
            events += "destroy:$handle"
            live -= handle
            attached -= handle
            return 0
        }
        override fun touch(handle: Long, x: Float, y: Float): Int { events += "touch:$handle:$x:$y"; return 0 }
        override fun reset(handle: Long): Int { events += "reset:$handle"; return 0 }
        override fun status(handle: Long): String { events += "status:$handle"; return "{\"live_instances\":${live.size}}" }
    }

    @Test fun repeatedOpenCloseRejectsCallbacksFromRetiredScreens() {
        val runtime = RecordingRuntime()
        var previous: ProbeSession<String>? = null
        repeat(10) {
            val current = ProbeSession(runtime, 42L) {}
            current.surfaceCreated()
            current.surfaceChanged("surface-$it", 640, 480, 2f)
            current.setPaused(false)
            val before = runtime.events.toList()
            previous?.apply {
                surfaceCreated()
                surfaceChanged("stale", 1, 1, 1f)
                surfaceDestroyed()
                touch(1f, 1f)
                reset()
                setPaused(false)
                assertNull(status(generation))
                close()
            }
            assertEquals(before, runtime.events)
            assertEquals(1, runtime.live.size)
            current.close()
            val handle = runtime.nextHandle
            assertTrue(runtime.events.indexOf("detach:$handle") < runtime.events.indexOf("destroy:$handle"))
            assertTrue(runtime.live.isEmpty())
            assertTrue(runtime.attached.isEmpty())
            previous = current
        }
    }

    @Test fun pauseAndSurfaceRecreationPreserveHandleAndRejectOldStatusGeneration() {
        val runtime = RecordingRuntime()
        val session = ProbeSession(runtime, 42L) {}
        repeat(5) { index ->
            session.surfaceCreated()
            session.surfaceChanged("surface-$index", 0, 0, 2f)
            assertTrue(runtime.attached.isEmpty())
            session.surfaceChanged("surface-$index", 640, 480, 2f)
            session.setPaused(false)
            session.touch(20f, 30f)
            session.reset()
            session.surfaceChanged("surface-$index", 320, 240, 3f)
            session.surfaceChanged("surface-$index", 0, 0, 3f)
            val generation = session.generation
            assertNotNull(session.status(generation))
            session.setPaused(true)
            val before = runtime.events.toList()
            session.touch(40f, 50f)
            assertEquals(before, runtime.events)
            session.surfaceDestroyed()
            assertNull(session.status(generation))
            assertTrue(runtime.attached.isEmpty())
        }
        session.close()
        assertEquals(1L, runtime.nextHandle)
        assertEquals(5, runtime.events.count { it.startsWith("touch:") })
        assertEquals(5, runtime.events.count { it.startsWith("reset:") })
        assertEquals(5, runtime.events.count { it.startsWith("detach:") })
    }

    @Test fun teardownFailureCannotBeReportedAsSafeSurfaceRelease() {
        val runtime = RecordingRuntime()
        val session = ProbeSession(runtime, 42L) {}
        session.surfaceCreated()
        session.surfaceChanged("surface", 640, 480, 2f)
        runtime.detachResult = -3
        assertThrows(IllegalStateException::class.java) { session.surfaceDestroyed() }
        assertThrows(IllegalStateException::class.java) { session.close() }
        assertTrue(runtime.live.isEmpty())
        assertEquals("destroy:1", runtime.events.last())
    }
}
