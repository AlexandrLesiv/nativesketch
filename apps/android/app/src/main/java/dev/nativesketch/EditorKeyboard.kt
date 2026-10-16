package dev.nativesketch

import android.graphics.*
import android.view.*
import android.widget.*
import kotlin.math.*

internal class EditorKeyboard(private val host: MainActivity) {
    fun handleEditorKey(event: KeyEvent): Boolean {
        with(host) {
            if (currentFocus is EditText || busy) return false
            if (event.keyCode == KeyEvent.KEYCODE_SPACE && canvas.hasFocus()) {
                if (event.action == KeyEvent.ACTION_DOWN && !spaceHeld) canvas.cancel()
                spaceHeld = event.action == KeyEvent.ACTION_DOWN
                return true
            }
            if (event.action != KeyEvent.ACTION_DOWN || event.repeatCount != 0) return false
            val ctrl = event.isCtrlPressed
            val action: (() -> Unit)? =
                when {
                    ctrl && event.keyCode == KeyEvent.KEYCODE_Z -> ({
                            checked(act(if (event.isShiftPressed) 9 else 8))
                        })
                    ctrl && event.keyCode == KeyEvent.KEYCODE_Y -> ({ checked(act(9)) })
                    ctrl && event.keyCode == KeyEvent.KEYCODE_S -> ({ save() })
                    ctrl && event.keyCode == KeyEvent.KEYCODE_O -> ({ replace(2) })
                    ctrl && event.keyCode == KeyEvent.KEYCODE_N -> ({ replace(1) })
                    !ctrl && event.keyCode == KeyEvent.KEYCODE_P -> ({
                            tools.check(tools.getChildAt(0).id)
                        })
                    !ctrl && event.keyCode == KeyEvent.KEYCODE_V -> ({
                            tools.check(tools.getChildAt(1).id)
                        })
                    event.keyCode == KeyEvent.KEYCODE_ESCAPE -> ({ canvas.cancel() })
                    event.keyCode == KeyEvent.KEYCODE_FORWARD_DEL ||
                        event.keyCode == KeyEvent.KEYCODE_DEL -> ({
                            if (engine.state(handle)[1] != 0.0) checked(act(4))
                        })
                    !ctrl && event.keyCode == KeyEvent.KEYCODE_F -> ({ canvas.fit() })
                    event.keyCode == KeyEvent.KEYCODE_EQUALS ||
                        event.keyCode == KeyEvent.KEYCODE_PLUS -> ({
                            canvas.scaleAt(1.25, canvas.width / 2.0, canvas.height / 2.0)
                        })
                    event.keyCode == KeyEvent.KEYCODE_MINUS -> ({
                            canvas.scaleAt(.8, canvas.width / 2.0, canvas.height / 2.0)
                        })
                    else -> null
                }
            if (action == null) return false
            safe {
                canvas.cancel()
                action()
            }
            return true
        }
    }
}
