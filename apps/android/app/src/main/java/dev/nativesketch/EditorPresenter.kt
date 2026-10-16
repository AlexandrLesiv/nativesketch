package dev.nativesketch

import android.view.*
import android.widget.*
import com.google.android.material.button.MaterialButton
import com.google.android.material.dialog.MaterialAlertDialogBuilder
import kotlin.math.*

internal class EditorPresenter(private val host: MainActivity) {
    fun safe(block: () -> Unit) {
        with(host) {
            try {
                block()
            } catch (e: Exception) {
                showError(e.message ?: "Operation failed")
            }
            refresh()
        }
    }

    fun showError(message: String) {
        with(host) {
            MaterialAlertDialogBuilder(this)
                .setTitle("Could not complete action")
                .setMessage(message)
                .setPositiveButton("OK", null)
                .show()
        }
    }

    fun button(label: String, action: () -> Unit): MaterialButton =
        with(host) {
            MaterialButton(this).apply {
                text = label
                minHeight = dp(48)
                setPadding(dp(10), 0, dp(10), 0)
                setOnClickListener {
                    if (!busy)
                        safe {
                            canvas.cancel()
                            action()
                        }
                }
            }
        }

    fun refresh() {
        with(host) {
            if (!canvasReady) return
            val s = engine.state(handle)
            for (i in 0 until tools.childCount) tools.getChildAt(i).isEnabled = !busy
            undo.isEnabled = !busy && s[2] != 0.0
            redo.isEnabled = !busy && s[3] != 0.0
            status.text =
                "${if(pen) "Pen" else "Select"} · ${s[0].toInt()} strokes · ${(canvas.zoom*100).roundToInt()}%${if(s[4]!=0.0) " · Unsaved" else ""}"
            canvas.contentDescription =
                "Drawing canvas. ${status.text}. ${if(s[1]!=0.0) "Stroke selected" else "No selection"}"
            canvas.invalidate()
        }
    }
}
