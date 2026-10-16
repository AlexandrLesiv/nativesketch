package dev.nativesketch

import android.graphics.*
import android.view.*
import android.widget.*
import kotlin.math.*

/** Injected smoke replay; this does not establish native E2E acceptance. */
internal class EditorReplay(private val host: MainActivity) {
    fun replay() =
        with(host) {
            checked(act(10))
            checked(engine.begin(handle, 7, 1, ink, 4.0))
            checked(engine.sample(handle, 7, 80.0, 90.0, 1.0, .5, true, 1))
            checked(engine.sample(handle, 7, 220.0, 180.0, 2.0, 1.0, true, 2))
            checked(act(1, 7.0))
            check(engine.state(handle)[0] == 1.0) {
                "Replay assertion: engine.state(handle)[0] == 1.0"
            }
            checked(act(3, 1.0))
            checked(act(5))
            checked(act(6, 1.4, 80.0, 90.0))
            checked(act(0))
            val before = engine.export(handle)
            checked(act(8))
            check(engine.state(handle)[0] == 0.0) {
                "Replay assertion: engine.state(handle)[0] == 0.0"
            }
            checked(act(9))
            check(before.contentEquals(engine.export(handle))) {
                "Replay assertion: before.contentEquals(engine.export(handle))"
            }
            check(engine.importBytes(handle, byteArrayOf(1, 2, 3)) != 0) {
                "Replay assertion: engine.importBytes(handle, byteArrayOf(1, 2, 3)) != 0"
            }
            check(before.contentEquals(engine.export(handle))) {
                "Replay assertion: before.contentEquals(engine.export(handle))"
            }
            checked(engine.begin(handle, 9, 2, ink, 4.0))
            checked(engine.sample(handle, 9, 20.0, 20.0, 3.0, 0.0, false, 3))
            checked(act(0))
            check(engine.state(handle)[0] == 1.0) {
                "Replay assertion: engine.state(handle)[0] == 1.0"
            }
            val revision = engine.revision(handle)
            check(engine.state(handle)[4] == 1.0) {
                "Replay assertion: engine.state(handle)[4] == 1.0"
            }
            checked(engine.markSaved(handle, revision))
            check(engine.state(handle)[4] == 0.0) {
                "Replay assertion: engine.state(handle)[4] == 0.0"
            }
            checked(engine.importBytes(handle, before))
            check(engine.state(handle)[0] == 1.0) {
                "Replay assertion: engine.state(handle)[0] == 1.0"
            }
            check(engine.render(handle).isNotEmpty()) {
                "Replay assertion: engine.render(handle).isNotEmpty()"
            }
            val fixture = ReplayInputFixture(host)
            fixture.event(MotionEvent.ACTION_DOWN, 1, 1000)
            check(engine.state(handle)[5] == 1.0) {
                "Replay assertion: engine.state(handle)[5] == 1.0"
            }
            fixture.event(
                MotionEvent.ACTION_POINTER_DOWN or (1 shl MotionEvent.ACTION_POINTER_INDEX_SHIFT),
                2,
                1010,
            )
            check(engine.state(handle)[5] == 0.0) {
                "Replay assertion: engine.state(handle)[5] == 0.0"
            }
            check(engine.state(handle)[0] == 1.0) {
                "Replay assertion: engine.state(handle)[0] == 1.0"
            }
            fixture.event(MotionEvent.ACTION_UP, 1, 1020)
            check(engine.state(handle)[0] == 1.0) {
                "Replay assertion: engine.state(handle)[0] == 1.0"
            }
            checked(engine.begin(handle, 12, 0, ink, 4.0))
            checked(engine.sample(handle, 12, 40.0, 40.0, 4.0, 0.0, false, 4))
            checked(act(1, 12.0))
            val closeSnapshot = engine.export(handle)
            onBackPressedDispatcher.onBackPressed()
            check(replaceDialog?.isShowing == true && !isFinishing) {
                "Replay assertion: replaceDialog?.isShowing == true && !isFinishing"
            }
            replaceDialog!!.getButton(android.content.DialogInterface.BUTTON_NEUTRAL).performClick()
            ReplayQueue(host).after("Back Cancel completion and desktop input") {
                val desktopSnapshot = DesktopReplay(host).run(closeSnapshot, fixture)
                ProviderReplay(host).run(desktopSnapshot, fixture)
            }
        }
}
