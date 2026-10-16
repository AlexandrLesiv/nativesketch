package dev.nativesketch

import android.graphics.*
import android.view.*
import android.widget.*
import kotlin.math.*

internal class DesktopReplay(private val host: MainActivity) {
    fun run(closeSnapshot: ByteArray, fixture: ReplayInputFixture): ByteArray =
        with(host) {
            check(replaceDialog == null) {
                "Back Cancel must dismiss its dialog before continuing"
            }
            check(!isFinishing && closeSnapshot.contentEquals(engine.export(handle))) {
                "Back Cancel must preserve the dirty drawing"
            }
            check(engine.state(handle)[4] == 1.0) {
                "Replay assertion: engine.state(handle)[4] == 1.0"
            }
            check(canvas.requestFocus() && canvas.hasFocus()) {
                "Replay assertion: canvas.requestFocus() && canvas.hasFocus()"
            }
            fixture.key(KeyEvent.KEYCODE_Z, KeyEvent.META_CTRL_ON)
            check(engine.state(handle)[0] == 1.0) {
                "Replay assertion: engine.state(handle)[0] == 1.0"
            }
            fixture.key(KeyEvent.KEYCODE_Z, KeyEvent.META_CTRL_ON or KeyEvent.META_SHIFT_ON)
            check(engine.state(handle)[0] == 2.0) {
                "Replay assertion: engine.state(handle)[0] == 2.0"
            }
            fixture.key(KeyEvent.KEYCODE_V)
            check(!pen) { "Replay assertion: !pen" }
            checked(act(3, 1.0))
            val desktopSnapshot = engine.export(handle)
            val historyBefore = engine.state(handle).copyOf()
            canvas.layout(canvas.left, canvas.top, canvas.right + 100, canvas.bottom + 50)
            check(desktopSnapshot.contentEquals(engine.export(handle))) {
                "Replay assertion: desktopSnapshot.contentEquals(engine.export(handle))"
            }
            check(
                engine.state(handle)[1] == historyBefore[1] &&
                    engine.state(handle)[2] == historyBefore[2]
            ) {
                "Replay assertion: engine.state(handle)[1] == historyBefore[1] && engine.state(handle)[2] == historyBefore[2]"
            }
            fixture.key(KeyEvent.KEYCODE_P)
            fixture.event(MotionEvent.ACTION_DOWN, 1, 2000)
            check(engine.state(handle)[5] == 1.0) {
                "Replay assertion: engine.state(handle)[5] == 1.0"
            }
            canvas.layout(canvas.left, canvas.top, canvas.right - 100, canvas.bottom - 50)
            check(engine.state(handle)[5] == 0.0) {
                "Replay assertion: engine.state(handle)[5] == 0.0"
            }
            check(desktopSnapshot.contentEquals(engine.export(handle))) {
                "Replay assertion: desktopSnapshot.contentEquals(engine.export(handle))"
            }
            check(canvas.requestFocus() && canvas.hasFocus()) {
                "Replay assertion: canvas.requestFocus() && canvas.hasFocus()"
            }
            fixture.key(KeyEvent.KEYCODE_SPACE)
            val oldOx = canvas.ox
            fixture.event(MotionEvent.ACTION_DOWN, 1, 2010)
            fixture.coords[0].x += 30f
            fixture.event(MotionEvent.ACTION_MOVE, 1, 2020)
            fixture.event(MotionEvent.ACTION_UP, 1, 2030)
            fixture.key(KeyEvent.KEYCODE_SPACE, action = KeyEvent.ACTION_UP)
            check(canvas.ox == oldOx + 30 && desktopSnapshot.contentEquals(engine.export(handle))) {
                "Replay assertion: canvas.ox == oldOx + 30 && desktopSnapshot.contentEquals(engine.export(handle))"
            }
            val wheel =
                MotionEvent.obtain(
                    1000,
                    2040,
                    MotionEvent.ACTION_SCROLL,
                    1,
                    fixture.props,
                    Array(1) {
                        MotionEvent.PointerCoords().apply {
                            x = 100f
                            y = 100f
                            setAxisValue(MotionEvent.AXIS_VSCROLL, 1f)
                        }
                    },
                    KeyEvent.META_CTRL_ON,
                    0,
                    1f,
                    1f,
                    0,
                    0,
                    InputDevice.SOURCE_MOUSE,
                    0,
                )
            val oldZoom = canvas.zoom
            canvas.onGenericMotionEvent(wheel)
            wheel.recycle()
            check(canvas.zoom > oldZoom && desktopSnapshot.contentEquals(engine.export(handle))) {
                "Replay assertion: canvas.zoom > oldZoom && desktopSnapshot.contentEquals(engine.export(handle))"
            }
            fixture.key(KeyEvent.KEYCODE_F)
            check(canvas.zoom.isFinite() && canvas.zoom > 0) {
                "Replay assertion: canvas.zoom.isFinite() && canvas.zoom > 0"
            }
            val secondary =
                MotionEvent.obtain(
                    1000,
                    2050,
                    MotionEvent.ACTION_DOWN,
                    1,
                    fixture.props,
                    fixture.coords,
                    0,
                    MotionEvent.BUTTON_SECONDARY,
                    1f,
                    1f,
                    0,
                    0,
                    InputDevice.SOURCE_MOUSE,
                    0,
                )
            canvas.onTouchEvent(secondary)
            secondary.recycle()
            check(
                engine.state(handle)[5] == 0.0 &&
                    desktopSnapshot.contentEquals(engine.export(handle))
            ) {
                "Replay assertion: engine.state(handle)[5] == 0.0 && desktopSnapshot.contentEquals(engine.export(handle))"
            }
            canvas.contextMenu?.dismiss()
            val field = EditText(this)
            (canvas.parent as LinearLayout).addView(field)
            field.requestFocus()
            val wasPen = pen
            fixture.key(KeyEvent.KEYCODE_V)
            check(pen == wasPen && desktopSnapshot.contentEquals(engine.export(handle))) {
                "Replay assertion: pen == wasPen && desktopSnapshot.contentEquals(engine.export(handle))"
            }
            (canvas.parent as LinearLayout).removeView(field)
            canvas.requestFocus()
            desktopSnapshot
        }
}
