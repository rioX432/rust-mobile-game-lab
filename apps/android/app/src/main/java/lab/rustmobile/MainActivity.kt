package lab.rustmobile

import android.app.Activity
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.util.Log
import android.view.Surface
import android.view.SurfaceHolder
import android.view.SurfaceView
import android.widget.Button
import android.widget.FrameLayout
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextView

class MainActivity : Activity() {
    private val handler = Handler(Looper.getMainLooper())
    private lateinit var container: FrameLayout
    private lateinit var status: TextView
    private lateinit var counters: TextView
    private lateinit var toggle: Button
    private var session: ProbeSession<Surface>? = null
    private var surfaceView: SurfaceView? = null
    private var callback: SurfaceHolder.Callback? = null
    private var polling: Runnable? = null
    private var resumed = false
    private var opens = 0
    private var returns = 0
    private var backgrounded = false
    private var expanded = true

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val root = LinearLayout(this).apply { orientation = LinearLayout.VERTICAL }
        root.setOnApplyWindowInsetsListener { view, insets ->
            view.setPadding(insets.systemWindowInsetLeft, insets.systemWindowInsetTop,
                insets.systemWindowInsetRight, insets.systemWindowInsetBottom)
            insets
        }
        root.addView(TextView(this).apply { setText(R.string.header); textSize = 20f })
        counters = TextView(this)
        root.addView(counters)
        val controls = LinearLayout(this)
        toggle = Button(this).apply {
            setText(R.string.open)
            setOnClickListener { if (session == null) open() else close() }
        }
        controls.addView(toggle, LinearLayout.LayoutParams(0, -2, 1f))
        controls.addView(Button(this).apply { setText(R.string.reset); setOnClickListener { session?.reset() } },
            LinearLayout.LayoutParams(0, -2, 1f))
        controls.addView(Button(this).apply { setText(R.string.resize); setOnClickListener { resizeProbe() } },
            LinearLayout.LayoutParams(0, -2, 1f))
        root.addView(controls)
        container = FrameLayout(this)
        root.addView(container, LinearLayout.LayoutParams(-1, 0, 1f))
        status = TextView(this).apply { setText(R.string.closed); setTextIsSelectable(true) }
        root.addView(ScrollView(this).apply { addView(status) }, LinearLayout.LayoutParams(-1, dp(160)))
        root.addView(TextView(this).apply { setText(R.string.footer) })
        setContentView(root)
        updateCounters()
    }

    private fun open() {
        if (session != null) return
        val current = ProbeSession(AndroidRuntime, 42L) { Log.i("RustMobileHost", it) }
        val view = ProbeSurfaceView(this) { x, y ->
            if (session === current) current.touch(x, y)
        }
        session = current
        surfaceView = view
        val listener = object : SurfaceHolder.Callback {
            override fun surfaceCreated(holder: SurfaceHolder) {
                if (session === current && surfaceView === view) current.surfaceCreated()
            }
            override fun surfaceChanged(holder: SurfaceHolder, format: Int, width: Int, height: Int) {
                if (session === current && surfaceView === view) {
                    current.surfaceChanged(holder.surface, width, height, resources.displayMetrics.density)
                }
            }
            override fun surfaceDestroyed(holder: SurfaceHolder) {
                if (session === current && surfaceView === view) current.surfaceDestroyed()
            }
        }
        callback = listener
        view.holder.addCallback(listener)
        current.setPaused(!resumed)
        container.addView(view, FrameLayout.LayoutParams(-1, if (expanded) -1 else dp(180)))
        opens++
        toggle.setText(R.string.close)
        updateCounters()
        if (resumed) startPolling(current)
    }

    private fun close() {
        val current = session ?: return
        stopPolling()
        current.close()
        surfaceView?.let { view -> callback?.let { view.holder.removeCallback(it) } }
        session = null
        surfaceView = null
        callback = null
        container.removeAllViews()
        toggle.setText(R.string.open)
        status.setText(R.string.closed_after_teardown)
    }

    private fun resizeProbe() {
        expanded = !expanded
        surfaceView?.let { view ->
            view.layoutParams = FrameLayout.LayoutParams(-1, if (expanded) -1 else dp(180))
        }
    }

    private fun startPolling(current: ProbeSession<Surface>) {
        stopPolling()
        val task = object : Runnable {
            override fun run() {
                if (!resumed || session !== current) return
                val generation = current.generation
                current.status(generation)?.let { json ->
                    if (session === current && generation == current.generation) status.text = json
                }
                handler.postDelayed(this, 250)
            }
        }
        polling = task
        handler.post(task)
    }

    private fun stopPolling() {
        polling?.let { handler.removeCallbacks(it) }
        polling = null
    }

    override fun onResume() {
        super.onResume()
        resumed = true
        if (backgrounded && session != null) returns++
        backgrounded = false
        session?.let { it.setPaused(false); startPolling(it) }
        updateCounters()
    }

    override fun onPause() {
        resumed = false
        backgrounded = session != null
        stopPolling()
        session?.setPaused(true)
        super.onPause()
    }

    @Deprecated("The probe consumes Back before leaving the host")
    override fun onBackPressed() {
        if (session != null) close() else super.onBackPressed()
    }

    override fun onDestroy() {
        close()
        super.onDestroy()
    }

    private fun updateCounters() {
        counters.text = getString(R.string.counters, opens, returns)
    }

    private fun dp(value: Int) = (value * resources.displayMetrics.density).toInt()
}
