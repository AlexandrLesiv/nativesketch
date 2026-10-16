package dev.nativesketch

import android.graphics.*
import android.view.*
import android.widget.*
import kotlin.math.*

internal class CanvasPainter(private val view: SketchView) {
    fun draw(c: Canvas) =
        with(view) {
            c.drawColor(Color.rgb(250, 250, 248))
            c.save()
            c.translate(ox.toFloat(), oy.toFloat())
            c.scale(zoom.toFloat(), zoom.toFloat())
            val commands = host.engine.render(host.handle)
            var stroke = -1.0
            var path = Path()
            fun flush() {
                if (!path.isEmpty) {
                    paint.style = Paint.Style.FILL
                    c.drawPath(path, paint)
                }
            }
            for (i in commands.indices step 12) {
                if (commands[i] != stroke) {
                    flush()
                    path = Path()
                    stroke = commands[i]
                    paint.color =
                        Color.argb(
                            (commands[i + 5] * 255).toInt(),
                            (commands[i + 2] * 255).toInt(),
                            (commands[i + 3] * 255).toInt(),
                            (commands[i + 4] * 255).toInt(),
                        )
                }
                when (commands[i + 1].toInt()) {
                    0 -> path.moveTo(commands[i + 6].toFloat(), commands[i + 7].toFloat())
                    1 -> path.lineTo(commands[i + 6].toFloat(), commands[i + 7].toFloat())
                    2 ->
                        path.cubicTo(
                            commands[i + 6].toFloat(),
                            commands[i + 7].toFloat(),
                            commands[i + 8].toFloat(),
                            commands[i + 9].toFloat(),
                            commands[i + 10].toFloat(),
                            commands[i + 11].toFloat(),
                        )
                    3 -> path.close()
                }
            }
            flush()
            val s = host.engine.state(host.handle)
            if (s[1] != 0.0) {
                paint.color = Color.rgb(55, 89, 168)
                paint.style = Paint.Style.STROKE
                paint.strokeWidth = (2 / zoom).toFloat()
                c.drawRect(
                    s[6].toFloat(),
                    s[7].toFloat(),
                    (s[6] + s[8]).toFloat(),
                    (s[7] + s[9]).toFloat(),
                    paint,
                )
                paint.style = Paint.Style.FILL
                c.drawCircle(
                    (s[6] + s[8]).toFloat(),
                    (s[7] + s[9]).toFloat(),
                    (6 / zoom).toFloat(),
                    paint,
                )
            }
            c.restore()
        }
}
