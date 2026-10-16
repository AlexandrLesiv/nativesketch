package dev.nativesketch

import android.graphics.*
import android.view.*
import android.widget.*
import com.google.android.material.button.MaterialButton
import kotlin.math.*

internal class EditorToolbar(private val host: MainActivity) {
    fun install(root: LinearLayout) =
        with(host) {
            val row = LinearLayout(this)
            tools =
                com.google.android.material.button.MaterialButtonToggleGroup(this).apply {
                    isSingleSelection = true
                    isSelectionRequired = true
                }
            val penButton =
                MaterialButton(this).apply {
                    id = View.generateViewId()
                    text = "Pen"
                    setPadding(dp(6), 0, dp(6), 0)
                    minWidth = 0
                    minHeight = dp(48)
                }
            val selectButton =
                MaterialButton(this).apply {
                    id = View.generateViewId()
                    text = "Select"
                    setPadding(dp(6), 0, dp(6), 0)
                    minWidth = 0
                    minHeight = dp(48)
                }
            val checkedState = arrayOf(intArrayOf(android.R.attr.state_checked), intArrayOf())
            fun themedColor(attribute: Int) =
                com.google.android.material.color.MaterialColors.getColor(
                    this,
                    attribute,
                    Color.BLACK,
                )
            val selectedBackground =
                themedColor(com.google.android.material.R.attr.colorSecondaryContainer)
            val selectedForeground =
                themedColor(com.google.android.material.R.attr.colorOnSecondaryContainer)
            val normalBackground = themedColor(com.google.android.material.R.attr.colorSurface)
            val normalForeground = themedColor(com.google.android.material.R.attr.colorOnSurface)
            val outline = themedColor(com.google.android.material.R.attr.colorOutline)
            for (toolButton in listOf(penButton, selectButton)) {
                toolButton.backgroundTintList =
                    android.content.res.ColorStateList(
                        checkedState,
                        intArrayOf(selectedBackground, normalBackground),
                    )
                toolButton.setTextColor(
                    android.content.res.ColorStateList(
                        checkedState,
                        intArrayOf(selectedForeground, normalForeground),
                    )
                )
                toolButton.strokeColor =
                    android.content.res.ColorStateList(
                        checkedState,
                        intArrayOf(selectedBackground, outline),
                    )
                toolButton.strokeWidth = dp(1)
            }
            tools.addView(penButton, LinearLayout.LayoutParams(0, dp(52), 1f))
            tools.addView(selectButton, LinearLayout.LayoutParams(0, dp(52), 1f))
            tools.check(if (pen) penButton.id else selectButton.id)
            tools.addOnButtonCheckedListener { _, checkedId, isChecked ->
                if (isChecked && !busy)
                    safe {
                        canvas.cancel()
                        pen = checkedId == penButton.id
                        if (pen) checked(act(11))
                    }
            }
            row.addView(tools)
            undo = button("Undo") { checked(act(8)) }
            redo = button("Redo") { checked(act(9)) }
            row.addView(undo)
            row.addView(redo)
            for (i in 0 until row.childCount) {
                row.getChildAt(i).layoutParams = LinearLayout.LayoutParams(0, dp(52), 1f)
                row.getChildAt(i).minimumWidth = 0
            }
            tools.layoutParams = LinearLayout.LayoutParams(0, dp(52), 2f)
            root.addView(row)
        }
}
