package lab.rustmobile

import android.view.Surface

object NativeBridge {
    init { System.loadLibrary("game_runtime") }

    @JvmStatic external fun create(seed: Long): Long
    @JvmStatic external fun destroy(handle: Long): Int
    @JvmStatic external fun attach(handle: Long, surface: Surface, width: Int, height: Int, scale: Float): Int
    @JvmStatic external fun resize(handle: Long, width: Int, height: Int, scale: Float): Int
    @JvmStatic external fun touch(handle: Long, xPixels: Float, yPixels: Float): Int
    @JvmStatic external fun pause(handle: Long, paused: Boolean): Int
    @JvmStatic external fun detach(handle: Long): Int
    @JvmStatic external fun action(handle: Long, action: Int, argument: Int): Int
    @JvmStatic external fun status(handle: Long): String
}
