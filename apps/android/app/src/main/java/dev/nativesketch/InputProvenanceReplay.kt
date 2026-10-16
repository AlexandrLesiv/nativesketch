package dev.nativesketch

import android.graphics.*
import android.view.*
import android.widget.*
import kotlin.math.*

internal class InputProvenanceReplay(private val host: MainActivity) {
    fun run(desktopSnapshot: ByteArray, fixture: ReplayInputFixture) =
        with(host) {
            // Export the actual native input result, checking the NSKETCH1 device field.
            for ((tool, source, device) in
                listOf(
                    Triple(MotionEvent.TOOL_TYPE_MOUSE, InputDevice.SOURCE_MOUSE, 0L),
                    Triple(MotionEvent.TOOL_TYPE_STYLUS, InputDevice.SOURCE_STYLUS, 1L),
                    Triple(
                        MotionEvent.TOOL_TYPE_FINGER,
                        InputDevice.SOURCE_TOUCHSCREEN,
                        2L,
                    ),
                )) {
                checked(act(10))
                fixture.props[0].toolType = tool
                for (action in listOf(MotionEvent.ACTION_DOWN, MotionEvent.ACTION_UP)) {
                    val input =
                        MotionEvent.obtain(
                            3000,
                            3010,
                            action,
                            1,
                            fixture.props,
                            fixture.coords,
                            0,
                            0,
                            1f,
                            1f,
                            0,
                            0,
                            source,
                            0,
                        )
                    canvas.onTouchEvent(input)
                    input.recycle()
                }
                val exported = engine.export(handle)
                check(
                    java.nio.ByteBuffer.wrap(exported)
                        .order(java.nio.ByteOrder.LITTLE_ENDIAN)
                        .getLong(24) == device
                ) {
                    "Replay assertion: java.nio.ByteBuffer.wrap(exported).order(java.nio.ByteOrder.LITTLE_ENDIAN).getLong(24) == device"
                }
            }
            checked(act(10))
            checked(engine.begin(handle, 22, 0, ink, 4.0))
            checked(engine.sample(handle, 22, -1e7, -1e7, 10.0, 0.0, false, 1))
            checked(engine.sample(handle, 22, 1e7, 1e7, 11.0, 0.0, false, 2))
            checked(act(1, 22.0))
            canvas.fit()
            val fitted = canvas.zoom
            check(fitted < .001) { "Replay assertion: fitted < .001" }
            canvas.scaleAt(.8, canvas.width / 2.0, canvas.height / 2.0)
            check(canvas.zoom <= fitted) { "Replay assertion: canvas.zoom <= fitted" }
            checked(engine.importBytes(handle, desktopSnapshot))
            canvas.fit()
        }
}
