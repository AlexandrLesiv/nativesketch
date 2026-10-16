package dev.nativesketch

import android.graphics.*
import android.view.*
import android.widget.*
import com.google.android.material.dialog.MaterialAlertDialogBuilder
import kotlin.math.*

internal class EditorZoomDialog(private val host: MainActivity) {
    fun zoom() {
        with(host) {
            MaterialAlertDialogBuilder(this)
                .setTitle("Canvas zoom")
                .setItems(arrayOf("Zoom in", "Zoom out", "Reset to 100%", "Fit drawing")) { _, n ->
                    if (n == 3) canvas.fit()
                    else if (n == 2) {
                        canvas.zoom = 1.0
                        canvas.ox = 0.0
                        canvas.oy = 0.0
                    } else
                        canvas.scaleAt(
                            if (n == 0) 1.25 else 0.8,
                            canvas.width / 2.0,
                            canvas.height / 2.0,
                        )
                    refresh()
                }
                .show()
        }
    }
}
