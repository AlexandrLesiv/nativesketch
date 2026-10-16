package dev.nativesketch

import android.graphics.*
import android.view.*
import android.widget.*
import kotlin.math.*

internal class ProviderReplay(private val host: MainActivity) {
    fun run(desktopSnapshot: ByteArray, fixture: ReplayInputFixture) =
        with(host) {
            // The same incoming URI path used by intents and drops must guard dirty work.
            receiveUri(android.net.Uri.parse("content://dev.nativesketch.invalid/replay.nsketch"))
            check(replaceDialog?.isShowing == true) {
                "Replay assertion: replaceDialog?.isShowing == true"
            }
            replaceDialog!!.getButton(android.content.DialogInterface.BUTTON_NEUTRAL).performClick()
            ReplayQueue(host).after("Incoming file Cancel completion and provenance") {
                check(replaceDialog == null) { "Incoming-file Cancel must dismiss its dialog" }
                check(pendingUri == null && desktopSnapshot.contentEquals(engine.export(handle))) {
                    "Incoming-file Cancel must clear the request and retain drawing bytes"
                }
                check(
                    acceptsDrop(
                        android.content.ClipDescription(
                            "drawing",
                            arrayOf("application/x-native-sketch"),
                        )
                    )
                ) {
                    "Replay assertion: acceptsDrop(android.content.ClipDescription(\"drawing\", arrayOf(\"application/x-native-sketch\")))"
                }
                check(
                    acceptsDrop(
                        android.content.ClipDescription(
                            "drawing",
                            arrayOf("application/octet-stream"),
                        )
                    )
                ) {
                    "Replay assertion: acceptsDrop(android.content.ClipDescription(\"drawing\", arrayOf(\"application/octet-stream\")))"
                }
                check(
                    !acceptsDrop(android.content.ClipDescription("image", arrayOf("image/png")))
                ) {
                    "Replay assertion: !acceptsDrop(android.content.ClipDescription(\"image\", arrayOf(\"image/png\")))"
                }
                InputProvenanceReplay(host).run(desktopSnapshot, fixture)
                android.util.Log.i(
                    "NativeSketchReplay",
                    "PASS: JNI stroke, pressure, cancel, resize rollback, history, corrupt import, revision save, roundtrip, Back dispatcher cancel retains dirty drawing, desktop keyboard history/tools/Fit, resize cancellation/history, Space drag, Ctrl wheel, incoming URI dirty cancel, native MIME drops, exported mouse/stylus/touch provenance, large-fit zoom-out monotonicity",
                )
                refresh()
            }
        }
}
