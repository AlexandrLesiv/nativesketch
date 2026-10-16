package dev.nativesketch

import android.graphics.*
import android.view.*
import android.widget.*
import com.google.android.material.dialog.MaterialAlertDialogBuilder
import kotlin.math.*

internal class EditorStyleDialog(private val host: MainActivity) {
    fun style() {
        with(host) {
            var pendingInk = ink
            val col =
                LinearLayout(this).apply {
                    orientation = LinearLayout.VERTICAL
                    setPadding(dp(24), 0, dp(24), 0)
                }
            val field =
                EditText(this).apply {
                    hint = "Stroke width (document units)"
                    inputType =
                        android.text.InputType.TYPE_CLASS_NUMBER or
                            android.text.InputType.TYPE_NUMBER_FLAG_DECIMAL
                    setText(width.toString())
                }
            col.addView(field)
            val colors = LinearLayout(this)
            listOf(
                    Color.rgb(32, 49, 75),
                    Color.rgb(189, 45, 64),
                    Color.rgb(21, 126, 94),
                    Color.rgb(86, 72, 190),
                )
                .forEachIndexed { i, c ->
                    colors.addView(
                        button(listOf("Ink", "Red", "Green", "Violet")[i]) { pendingInk = c }
                    )
                }
            val scroll = HorizontalScrollView(this)
            scroll.addView(colors)
            col.addView(scroll)
            val dialog =
                MaterialAlertDialogBuilder(this)
                    .setTitle("Pen style")
                    .setView(col)
                    .setPositiveButton("Apply", null)
                    .setNegativeButton("Cancel", null)
                    .create()
            dialog.setOnShowListener {
                dialog.getButton(-1).setOnClickListener {
                    val w = field.text.toString().toDoubleOrNull()
                    if (w == null || !w.isFinite() || w !in 0.1..100.0)
                        field.error = "Enter 0.1–100"
                    else {
                        width = w
                        ink = pendingInk
                        dialog.dismiss()
                    }
                }
            }
            dialog.show()
        }
    }
}
