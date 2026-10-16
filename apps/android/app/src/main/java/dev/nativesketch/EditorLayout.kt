package dev.nativesketch

import android.graphics.*
import android.view.*
import android.widget.*
import com.google.android.material.appbar.MaterialToolbar
import kotlin.math.*

internal class EditorLayout(private val host: MainActivity) {
    fun install(retained: Retained?) =
        with(host) {
            val background =
                com.google.android.material.color.MaterialColors.getColor(
                    this,
                    android.R.attr.colorBackground,
                    Color.WHITE,
                )
            val root =
                LinearLayout(this).apply {
                    orientation = LinearLayout.VERTICAL
                    setBackgroundColor(background)
                }
            androidx.core.view.WindowCompat.getInsetsController(window, window.decorView).apply {
                val lightBackground =
                    androidx.core.graphics.ColorUtils.calculateLuminance(background) > .5
                isAppearanceLightStatusBars = lightBackground
                isAppearanceLightNavigationBars = lightBackground
            }
            root.setOnApplyWindowInsetsListener { v, i ->
                if (android.os.Build.VERSION.SDK_INT >= 30) {
                    val b = i.getInsets(WindowInsets.Type.systemBars())
                    v.setPadding(b.left, b.top, b.right, b.bottom)
                } else {
                    v.setPadding(
                        i.systemWindowInsetLeft,
                        i.systemWindowInsetTop,
                        i.systemWindowInsetRight,
                        i.systemWindowInsetBottom,
                    )
                }
                i
            }
            val toolbar =
                MaterialToolbar(this).apply {
                    title = "Native Sketch"
                    menu.add("Document").setShowAsAction(0)
                    setOnMenuItemClickListener {
                        documents()
                        true
                    }
                }
            root.addView(toolbar)
            EditorToolbar(host).install(root)
            canvas = SketchView(host)
            if (retained != null) {
                canvas.zoom = retained.zoom
                canvas.restoreCenterX = retained.ox
                canvas.restoreCenterY = retained.oy
            }
            val workspace = LinearLayout(this)
            workspace.addView(canvas, LinearLayout.LayoutParams(0, -1, 1f))
            sidePanel =
                LinearLayout(this).apply {
                    orientation = LinearLayout.VERTICAL
                    setPadding(dp(8), dp(8), dp(8), dp(8))
                    addView(TextView(host).apply { text = "Drawing controls" })
                    addView(button("Pen style") { style() })
                    addView(button("Selection…") { selection() })
                    addView(button("Fit drawing") { canvas.fit() })
                    addView(
                        button("Zoom in") {
                            canvas.scaleAt(1.25, canvas.width / 2.0, canvas.height / 2.0)
                        }
                    )
                    addView(
                        button("Zoom out") {
                            canvas.scaleAt(.8, canvas.width / 2.0, canvas.height / 2.0)
                        }
                    )
                }
            val inspectorScroll =
                ScrollView(this).apply {
                    isFillViewport = true
                    addView(sidePanel, ViewGroup.LayoutParams(-1, -2))
                }
            workspace.addView(inspectorScroll, LinearLayout.LayoutParams(dp(192), -1))
            root.addView(workspace, LinearLayout.LayoutParams(-1, 0, 1f))
            status =
                TextView(this).apply {
                    setPadding(dp(12), dp(4), dp(12), dp(4))
                    accessibilityLiveRegion = View.ACCESSIBILITY_LIVE_REGION_POLITE
                }
            root.addView(status)
            val bottom = LinearLayout(this)
            bottomBar = bottom
            bottom.addView(button("Style") { style() })
            bottom.addView(button("Selection") { selection() })
            bottom.addView(button("Zoom") { zoom() })
            root.addView(bottom)
            root.addOnLayoutChangeListener { _, left, _, right, _, _, _, _, _ ->
                val expanded = right - left >= dp(840)
                inspectorScroll.visibility = if (expanded) View.VISIBLE else View.GONE
                bottomBar.visibility = if (expanded) View.GONE else View.VISIBLE
            }
            canvas.setOnDragListener { _, event ->
                when (event.action) {
                    DragEvent.ACTION_DRAG_STARTED -> acceptsDrop(event.clipDescription)
                    DragEvent.ACTION_DROP -> {
                        val uri = event.clipData?.takeIf { it.itemCount == 1 }?.getItemAt(0)?.uri
                        if (uri?.scheme == "content" && !busy) {
                            // Keep the transient grant alive through a possible Save/Discard/Cancel
                            // dialog.
                            dropPermission?.release()
                            dropPermission = requestDragAndDropPermissions(event)
                            receiveUri(uri)
                            true
                        } else false
                    }
                    else -> true
                }
            }
            setContentView(root)
            refresh()
        }
}
