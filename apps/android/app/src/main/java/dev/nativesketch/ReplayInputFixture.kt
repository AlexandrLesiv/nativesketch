package dev.nativesketch

import android.graphics.*
import android.view.*
import android.widget.*
import kotlin.math.*

internal class ReplayInputFixture(private val host: MainActivity) {
    val props =
        Array(2) { i ->
            MotionEvent.PointerProperties().apply {
                id = i
                toolType = MotionEvent.TOOL_TYPE_FINGER
            }
        }
    val coords =
        Array(2) { i ->
            MotionEvent.PointerCoords().apply {
                x = 100f + i * 80
                y = 100f
                pressure = 1f
                size = 1f
            }
        }

    fun event(action: Int, count: Int, time: Long) {
        val e =
            MotionEvent.obtain(
                1000,
                time,
                action,
                count,
                props,
                coords,
                0,
                0,
                1f,
                1f,
                0,
                0,
                android.view.InputDevice.SOURCE_TOUCHSCREEN,
                0,
            )
        host.canvas.onTouchEvent(e)
        e.recycle()
    }

    fun key(code: Int, meta: Int = 0, action: Int = KeyEvent.ACTION_DOWN) {
        val event = KeyEvent(1000, 1000, action, code, 0, meta)
        if (action == KeyEvent.ACTION_UP) host.onKeyUp(code, event)
        else if (meta and KeyEvent.META_CTRL_ON != 0) host.onKeyShortcut(code, event)
        else host.onKeyDown(code, event)
    }
}
