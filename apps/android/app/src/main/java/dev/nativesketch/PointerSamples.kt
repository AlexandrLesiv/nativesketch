package dev.nativesketch

import android.graphics.*
import android.view.*
import android.widget.*
import kotlin.math.*

internal class PointerSamples(private val view: SketchView) {
    fun append(e: MotionEvent, index: Int) =
        with(view) {
            val range = e.device?.getMotionRange(MotionEvent.AXIS_PRESSURE, e.source)
            val stylus =
                e.getToolType(index) == MotionEvent.TOOL_TYPE_STYLUS &&
                    range != null &&
                    range.min.isFinite() &&
                    range.max.isFinite() &&
                    range.range > 0
            fun pressure(p: Float): Double {
                return if (stylus && p.isFinite())
                    ((p - range!!.min) / range.range).toDouble().coerceIn(0.0, 1.0)
                else 0.0
            }
            for (h in 0 until e.historySize) host.checked(
                host.engine.sample(
                    host.handle,
                    pointerState.pointer,
                    (e.getHistoricalX(index, h) - ox) / zoom,
                    (e.getHistoricalY(index, h) - oy) / zoom,
                    e.getHistoricalEventTime(h) / 1000.0,
                    pressure(e.getHistoricalPressure(index, h)),
                    stylus && e.getHistoricalPressure(index, h).isFinite(),
                    ++pointerState.id,
                )
            )
            host.checked(
                host.engine.sample(
                    host.handle,
                    pointerState.pointer,
                    (e.getX(index) - ox) / zoom,
                    (e.getY(index) - oy) / zoom,
                    e.eventTime / 1000.0,
                    pressure(e.getPressure(index)),
                    stylus && e.getPressure(index).isFinite(),
                    ++pointerState.id,
                )
            )
        }
}
