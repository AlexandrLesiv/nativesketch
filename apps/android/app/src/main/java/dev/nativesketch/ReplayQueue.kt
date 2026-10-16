package dev.nativesketch

import android.graphics.*
import android.view.*
import android.widget.*
import kotlin.math.*

internal class ReplayQueue(private val host: MainActivity) {
    fun after(label: String, block: () -> Unit) {
        with(host) {
            // AlertDialog queues button and dismissal callbacks. Let both finish before assertions.
            canvas.post {
                canvas.post {
                    safe {
                        try {
                            block()
                        } catch (error: Exception) {
                            android.util.Log.e(
                                "NativeSketchReplay",
                                "FAIL: Android replay ($label)",
                                error,
                            )
                            throw error
                        }
                    }
                }
            }
        }
    }
}
