package dev.nativesketch

import android.content.Intent
import android.graphics.*
import android.os.Bundle
import android.view.*
import android.widget.*
import com.google.android.material.button.MaterialButton
import kotlin.math.*

class MainActivity : androidx.activity.ComponentActivity() {
    private val presenter = EditorPresenter(this)
    internal val canvasReady
        get() = ::canvas.isInitialized

    internal val engine = Engine()
    internal var handle = 0L
    internal lateinit var canvas: SketchView
    internal lateinit var status: TextView
    internal lateinit var undo: MaterialButton
    internal lateinit var redo: MaterialButton
    internal lateinit var tools: com.google.android.material.button.MaterialButtonToggleGroup
    internal var replaceDialog: androidx.appcompat.app.AlertDialog? = null
    internal var busy = false
    internal var pendingUri: android.net.Uri? = null
    internal var spaceHeld = false
    internal lateinit var sidePanel: LinearLayout
    internal lateinit var bottomBar: LinearLayout
    internal var pendingReplace = 0 // 1 new, 2 open, 3 finish
    internal var savedBytes: ByteArray? = null
    internal var savedRevision = 0L
    internal var pen = true
    internal var ink = Color.rgb(32, 49, 75)
    internal var width = 4.0

    internal fun dp(n: Int) = (n * resources.displayMetrics.density).roundToInt()

    internal fun act(op: Int, a: Double = 0.0, b: Double = 0.0, c: Double = 0.0) =
        engine.action(handle, op, a, b, c)

    internal fun checked(code: Int) {
        if (code != 0) throw IllegalStateException("Operation rejected ($code)")
    }

    internal fun safe(block: () -> Unit) = presenter.safe(block)

    internal fun showError(message: String) = presenter.showError(message)

    internal fun button(label: String, action: () -> Unit) = presenter.button(label, action)

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        onBackPressedDispatcher.addCallback(
            this,
            object : androidx.activity.OnBackPressedCallback(true) {
                override fun handleOnBackPressed() {
                    requestClose()
                }
            },
        )
        val retained = lastCustomNonConfigurationInstance as? Retained
        handle = retained?.handle ?: engine.create()
        if (retained != null) {
            busy = retained.busy
            savedBytes = retained.bytes
            savedRevision = retained.revision
            pendingReplace = retained.replace
            pen = retained.pen
            ink = retained.ink
            width = retained.width
            pendingUri = retained.uri
            dropPermission = retained.permission
        }
        EditorLayout(this).install(retained)
        if (retained == null) receiveIntent(intent)
        else if (pendingUri != null && !busy) canvas.post { replace(4) }
        if (
            intent.getBooleanExtra("replay", false) &&
                applicationInfo.flags and android.content.pm.ApplicationInfo.FLAG_DEBUGGABLE != 0
        ) {
            intent.removeExtra("replay")
            canvas.post {
                safe {
                    try {
                        replay()
                    } catch (error: Exception) {
                        android.util.Log.e("NativeSketchReplay", "FAIL: Android replay", error)
                        throw error
                    }
                }
            }
        }
    }

    override fun onPause() {
        if (handle != 0L) canvas.cancel()
        super.onPause()
    }

    override fun onWindowFocusChanged(hasFocus: Boolean) {
        super.onWindowFocusChanged(hasFocus)
        if (!hasFocus && ::canvas.isInitialized) {
            spaceHeld = false
            canvas.cancel()
        }
    }

    override fun onRetainCustomNonConfigurationInstance(): Any {
        return EditorRetention(this).capture()
    }

    override fun onDestroy() {
        if (!isChangingConfigurations) {
            if (handle != 0L) engine.free(handle)
            dropPermission?.release()
        }
        handle = 0
        super.onDestroy()
    }

    internal fun requestClose() {
        if (!busy) {
            canvas.cancel()
            replace(3)
        }
    }

    override fun onKeyUp(keyCode: Int, event: KeyEvent): Boolean {
        if (keyCode == KeyEvent.KEYCODE_SPACE && spaceHeld) {
            spaceHeld = false
            return true
        }
        return super.onKeyUp(keyCode, event)
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        receiveIntent(intent)
    }

    internal fun refresh() = presenter.refresh()

    override fun onKeyDown(keyCode: Int, event: KeyEvent) =
        keyboard.handleEditorKey(event) || super.onKeyDown(keyCode, event)

    override fun onKeyShortcut(keyCode: Int, event: KeyEvent) =
        keyboard.handleEditorKey(event) || super.onKeyShortcut(keyCode, event)

    internal var dropPermission: android.view.DragAndDropPermissions? = null
    private val keyboard = EditorKeyboard(this)
    private val documents = EditorDocuments(this)
    private val incoming = IncomingDocuments(this)

    internal fun documents() = documents.documents()

    internal fun replace(kind: Int) = documents.replace(kind)

    internal fun open() = documents.open()

    internal fun save() = documents.save()

    internal fun receiveIntent(intent: Intent) = incoming.receiveIntent(intent)

    internal fun receiveUri(uri: android.net.Uri) = incoming.receiveUri(uri)

    internal fun importUri(uri: android.net.Uri) = incoming.importUri(uri)

    internal fun completeIncoming() = incoming.completeIncoming()

    internal fun style() = EditorStyleDialog(this).style()

    internal fun selection() = EditorSelectionDialog(this).selection()

    internal fun zoom() = EditorZoomDialog(this).zoom()

    internal fun replay() = EditorReplay(this).replay()

    internal fun acceptsDrop(description: android.content.ClipDescription?) =
        acceptsDrawingDrop(description)

    @Deprecated("Native document result bridge")
    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        DocumentResults(this).receive(requestCode, resultCode, data)
    }
}
