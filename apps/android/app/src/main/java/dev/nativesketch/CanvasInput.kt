package dev.nativesketch

import android.graphics.*
import android.view.*
import android.widget.*
import kotlin.math.*

internal class CanvasInput(private val view: SketchView) {
    private val samples = PointerSamples(view)

    fun touch(e: MotionEvent): Boolean {
        with(view) {
            if (host.busy) return true
            try {
                when (e.actionMasked) {
                    MotionEvent.ACTION_DOWN -> {
                        requestFocus()
                        pointerState.pointer = e.getPointerId(0)
                        pointerState.navigating = false
                        if (e.buttonState and MotionEvent.BUTTON_SECONDARY != 0) {
                            cancel()
                            contextMenu =
                                PopupMenu(host, view).apply {
                                    menu.add("Fit drawing").setOnMenuItemClickListener {
                                        fit()
                                        host.refresh()
                                        true
                                    }
                                    menu.add("Selection…").setOnMenuItemClickListener {
                                        host.selection()
                                        true
                                    }
                                    show()
                                }
                            return true
                        }
                        if (host.spaceHeld || e.buttonState and MotionEvent.BUTTON_TERTIARY != 0) {
                            pointerState.navigating = true
                            pointerState.mousePan = true
                            pointerState.prevX = e.x.toDouble()
                            pointerState.prevY = e.y.toDouble()
                            return true
                        }
                        val x = (e.x - ox) / zoom
                        val y = (e.y - oy) / zoom
                        if (host.pen) {
                            host.checked(
                                host.engine.begin(
                                    host.handle,
                                    pointerState.pointer,
                                    when (e.getToolType(0)) {
                                        MotionEvent.TOOL_TYPE_STYLUS,
                                        MotionEvent.TOOL_TYPE_ERASER -> 1
                                        MotionEvent.TOOL_TYPE_MOUSE -> 0
                                        else ->
                                            if (e.isFromSource(InputDevice.SOURCE_MOUSE)) 0 else 2
                                    },
                                    host.ink,
                                    host.width,
                                )
                            )
                            samples.append(e, 0)
                        } else {
                            val s = host.engine.state(host.handle)
                            if (
                                s[1] != 0.0 &&
                                    hypot(x - s[6] - s[8], y - s[7] - s[9]) * zoom < host.dp(24)
                            ) {
                                host.checked(host.act(5))
                                pointerState.resizing = true
                                pointerState.anchorX = s[6]
                                pointerState.anchorY = s[7]
                                pointerState.initialDistance =
                                    hypot(x - pointerState.anchorX, y - pointerState.anchorY)
                                        .coerceAtLeast(.001)
                            } else host.checked(host.act(2, x, y, host.dp(16) / zoom))
                        }
                    }
                    MotionEvent.ACTION_POINTER_DOWN -> {
                        host.act(0)
                        pointerState.pointer = -1
                        pointerState.resizing = false
                        pointerState.navigating = true
                        if (e.pointerCount >= 2) {
                            pointerState.prevX = (e.getX(0) + e.getX(1)) / 2.0
                            pointerState.prevY = (e.getY(0) + e.getY(1)) / 2.0
                            pointerState.prevSpan =
                                hypot(
                                        (e.getX(1) - e.getX(0)).toDouble(),
                                        (e.getY(1) - e.getY(0)).toDouble(),
                                    )
                                    .coerceAtLeast(1.0)
                        }
                    }
                    MotionEvent.ACTION_MOVE -> {
                        if (pointerState.mousePan) {
                            ox += e.x - pointerState.prevX
                            oy += e.y - pointerState.prevY
                            pointerState.prevX = e.x.toDouble()
                            pointerState.prevY = e.y.toDouble()
                        } else if (pointerState.navigating && e.pointerCount >= 2) {
                            val x = (e.getX(0) + e.getX(1)) / 2.0
                            val y = (e.getY(0) + e.getY(1)) / 2.0
                            val span =
                                hypot(
                                        (e.getX(1) - e.getX(0)).toDouble(),
                                        (e.getY(1) - e.getY(0)).toDouble(),
                                    )
                                    .coerceAtLeast(1.0)
                            if (!pointerState.reseed) {
                                scaleAt(
                                    span / pointerState.prevSpan,
                                    pointerState.prevX,
                                    pointerState.prevY,
                                )
                                ox += x - pointerState.prevX
                                oy += y - pointerState.prevY
                            }
                            pointerState.reseed = false
                            pointerState.prevX = x
                            pointerState.prevY = y
                            pointerState.prevSpan = span
                        } else if (!pointerState.navigating && pointerState.pointer >= 0) {
                            val idx = e.findPointerIndex(pointerState.pointer)
                            if (idx < 0) cancel()
                            else if (pointerState.resizing)
                                host.checked(
                                    host.act(
                                        6,
                                        hypot(
                                            (e.getX(idx) - ox) / zoom - pointerState.anchorX,
                                            (e.getY(idx) - oy) / zoom - pointerState.anchorY,
                                        ) / pointerState.initialDistance,
                                        pointerState.anchorX,
                                        pointerState.anchorY,
                                    )
                                )
                            else if (host.pen) samples.append(e, idx)
                        }
                    }
                    MotionEvent.ACTION_UP -> {
                        if (!pointerState.navigating && pointerState.pointer >= 0) {
                            if (pointerState.resizing) host.checked(host.act(7))
                            else if (host.pen) {
                                samples.append(e, 0)
                                host.checked(host.act(1, pointerState.pointer.toDouble()))
                            }
                        }
                        cancel()
                    }
                    MotionEvent.ACTION_POINTER_UP -> {
                        pointerState.reseed = true
                    }
                    MotionEvent.ACTION_CANCEL -> cancel()
                }
            } catch (ex: Exception) {
                cancel()
                host.showError(ex.message ?: "Input failed")
            }
            host.refresh()
            return true
        }
    }
}
