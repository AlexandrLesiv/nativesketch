package dev.nativesketch

import android.graphics.*
import android.view.*
import android.widget.*
import kotlin.math.*

internal class SketchView(internal val host: MainActivity) : View(host) {
    internal val pointerState = PointerState()
    var zoom = 1.0
    var ox = 0.0
    var oy = 0.0
    var restoreCenterX: Double? = null
    var restoreCenterY: Double? = null
    var contextMenu: PopupMenu? = null
    internal val paint = Paint(Paint.ANTI_ALIAS_FLAG)

    init {
        isFocusable = true
        isFocusableInTouchMode = true
        importantForAccessibility = IMPORTANT_FOR_ACCESSIBILITY_YES
    }

    fun cancel() {
        if (host.handle != 0L) host.act(0)
        pointerState.pointer = -1
        pointerState.resizing = false
        pointerState.navigating = false
        pointerState.mousePan = false
        invalidate()
    }

    fun scaleAt(factor: Double, x: Double, y: Double) {
        if (!factor.isFinite() || factor <= 0.0) return
        // Fit may legitimately show very large drawings below the manual zoom floor.
        // Zoom out must never jump inward from that fitted viewport.
        val next = (zoom * factor).coerceIn(minOf(zoom, .001), 8.0)
        ox = x - (x - ox) * next / zoom
        oy = y - (y - oy) * next / zoom
        zoom = next
    }

    fun fit() {
        val commands = host.engine.render(host.handle)
        if (commands.isEmpty()) {
            zoom = 1.0
            ox = 0.0
            oy = 0.0
            return
        }
        var minX = Double.POSITIVE_INFINITY
        var minY = minX
        var maxX = Double.NEGATIVE_INFINITY
        var maxY = maxX
        for (i in commands.indices step 12) {
            val count =
                when (commands[i + 1].toInt()) {
                    0,
                    1 -> 1
                    2 -> 3
                    else -> 0
                }
            for (j in 0 until count) {
                val x = commands[i + 6 + j * 2]
                val y = commands[i + 7 + j * 2]
                minX = min(minX, x)
                maxX = max(maxX, x)
                minY = min(minY, y)
                maxY = max(maxY, y)
            }
        }
        if (!minX.isFinite()) return
        zoom =
            min(
                    (width - host.dp(48)).coerceAtLeast(1) / (maxX - minX).coerceAtLeast(1.0),
                    (height - host.dp(48)).coerceAtLeast(1) / (maxY - minY).coerceAtLeast(1.0),
                )
                .coerceAtMost(8.0)
        ox = width / 2.0 - (minX + maxX) / 2 * zoom
        oy = height / 2.0 - (minY + maxY) / 2 * zoom
    }

    override fun onResolvePointerIcon(event: MotionEvent, pointerIndex: Int): PointerIcon =
        PointerIcon.getSystemIcon(
            context,
            if (host.spaceHeld) PointerIcon.TYPE_GRAB
            else if (host.pen) PointerIcon.TYPE_CROSSHAIR else PointerIcon.TYPE_ARROW,
        )

    override fun onGenericMotionEvent(e: MotionEvent): Boolean {
        if (host.busy) return true
        if (e.actionMasked == MotionEvent.ACTION_SCROLL) {
            cancel()
            val dy = e.getAxisValue(MotionEvent.AXIS_VSCROLL).toDouble()
            val dx = e.getAxisValue(MotionEvent.AXIS_HSCROLL).toDouble()
            if (e.metaState and KeyEvent.META_CTRL_ON != 0)
                scaleAt(exp(dy * .12), e.x.toDouble(), e.y.toDouble())
            else {
                ox += dx * host.dp(48)
                oy += dy * host.dp(48)
            }
            host.refresh()
            return true
        }
        return super.onGenericMotionEvent(e)
    }

    override fun onSizeChanged(w: Int, h: Int, oldw: Int, oldh: Int) {
        cancel()
        if (restoreCenterX != null) {
            ox = w / 2.0 - restoreCenterX!! * zoom
            oy = h / 2.0 - restoreCenterY!! * zoom
            restoreCenterX = null
            restoreCenterY = null
        } else if (oldw > 0) {
            ox += (w - oldw) / 2.0
            oy += (h - oldh) / 2.0
        }
    }

    override fun onDraw(c: Canvas) {
        super.onDraw(c)
        painter.draw(c)
    }

    override fun onTouchEvent(e: MotionEvent): Boolean = input.touch(e)

    private val painter = CanvasPainter(this)
    private val input = CanvasInput(this)
}
