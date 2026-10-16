package dev.nativesketch

import android.graphics.*
import android.view.*
import android.widget.*
import com.google.android.material.dialog.MaterialAlertDialogBuilder
import kotlin.math.*

internal class EditorSelectionDialog(private val host: MainActivity) {
    fun selection() {
        with(host) {
            var update: () -> Unit = {}
            val s = engine.state(handle)
            val col =
                LinearLayout(this).apply {
                    orientation = LinearLayout.VERTICAL
                    setPadding(dp(24), 0, dp(24), 0)
                }
            col.addView(
                button("Previous stroke") {
                        tools.check(tools.getChildAt(1).id)
                        checked(act(3, -1.0))
                        update()
                    }
                    .apply { isEnabled = s[0] > 0 }
            )
            col.addView(
                button("Next stroke") {
                        tools.check(tools.getChildAt(1).id)
                        checked(act(3, 1.0))
                        update()
                    }
                    .apply { isEnabled = s[0] > 0 }
            )
            col.addView(
                button("Delete selected") {
                        checked(act(4))
                        update()
                    }
                    .apply { isEnabled = s[1] > 0 }
            )
            val field =
                EditText(this).apply {
                    hint = "Uniform scale (%)"
                    setText("100")
                    inputType =
                        android.text.InputType.TYPE_CLASS_NUMBER or
                            android.text.InputType.TYPE_NUMBER_FLAG_DECIMAL
                }
            col.addView(field)
            col.addView(
                button("Scale selected") {
                        val v = field.text.toString().toDoubleOrNull()
                        require(v != null && v.isFinite() && v > 0 && v <= 10000) {
                            "Enter a scale above 0 and at most 10000%"
                        }
                        val st = engine.state(handle)
                        checked(act(5))
                        try {
                            checked(act(6, v / 100, st[6], st[7]))
                            checked(act(7))
                        } catch (e: Exception) {
                            act(0)
                            throw e
                        }
                    }
                    .apply { isEnabled = s[1] > 0 }
            )
            update = {
                val selected = engine.state(handle)[1] != 0.0
                col.getChildAt(2).isEnabled = selected
                col.getChildAt(4).isEnabled = selected
            }
            MaterialAlertDialogBuilder(this)
                .setTitle("Selection")
                .setView(col)
                .setPositiveButton("Done", null)
                .show()
        }
    }
}
