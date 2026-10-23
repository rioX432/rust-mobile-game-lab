package lab.rustmobile

import android.content.Context
import android.view.MotionEvent
import android.view.SurfaceView

internal class ProbeSurfaceView(context: Context, private val onTap: (Float, Float) -> Unit) : SurfaceView(context) {
    private var tapPosition: Pair<Float, Float>? = null

    init {
        isClickable = true
        contentDescription = context.getString(R.string.probe_surface)
    }

    override fun onTouchEvent(event: MotionEvent): Boolean {
        when (event.actionMasked) {
            MotionEvent.ACTION_UP -> {
                tapPosition = event.x to event.y
                performClick()
            }
            MotionEvent.ACTION_CANCEL -> tapPosition = null
        }
        return true
    }

    override fun performClick(): Boolean {
        super.performClick()
        val position = tapPosition ?: (width / 2f to height / 2f)
        tapPosition = null
        onTap(position.first, position.second)
        return true
    }
}
