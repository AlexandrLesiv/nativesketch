package dev.nativesketch

import android.content.Intent
import android.graphics.*
import android.os.Bundle
import android.view.*
import android.widget.*
import com.google.android.material.appbar.MaterialToolbar
import com.google.android.material.button.MaterialButton
import com.google.android.material.dialog.MaterialAlertDialogBuilder
import kotlin.math.*

class MainActivity : androidx.activity.ComponentActivity() {
    private val engine = Engine()
    private var handle = 0L
    private lateinit var canvas: SketchView
    private lateinit var status: TextView
    private lateinit var undo: MaterialButton
    private lateinit var redo: MaterialButton
    private lateinit var tools: com.google.android.material.button.MaterialButtonToggleGroup
    private var replaceDialog: androidx.appcompat.app.AlertDialog? = null
    private var busy = false
    private var pendingUri: android.net.Uri? = null
    private var spaceHeld = false
    private lateinit var sidePanel: LinearLayout
    private lateinit var bottomBar: LinearLayout
    private var pendingReplace = 0 // 1 new, 2 open, 3 finish
    private var savedBytes: ByteArray? = null
    private var savedRevision = 0L
    private var pen = true
    private var ink = Color.rgb(32, 49, 75)
    private var width = 4.0

    private fun dp(n: Int) = (n * resources.displayMetrics.density).roundToInt()

    private fun act(op: Int, a: Double = 0.0, b: Double = 0.0, c: Double = 0.0) =
        engine.action(handle, op, a, b, c)

    private fun checked(code: Int) {
        if (code != 0) throw IllegalStateException("Operation rejected ($code)")
    }

    private fun safe(block: () -> Unit) {
        try {
            block()
        } catch (e: Exception) {
            showError(e.message ?: "Operation failed")
        }
        refresh()
    }

    private fun showError(message: String) {
        MaterialAlertDialogBuilder(this)
            .setTitle("Could not complete action")
            .setMessage(message)
            .setPositiveButton("OK", null)
            .show()
    }

    private fun button(label: String, action: () -> Unit): MaterialButton =
        MaterialButton(this).apply {
            text = label
            minHeight = dp(48)
            setPadding(dp(10), 0, dp(10), 0)
            setOnClickListener {
                if (!busy)
                    safe {
                        canvas.cancel()
                        action()
                    }
            }
        }

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
        val root = LinearLayout(this).apply { orientation = LinearLayout.VERTICAL }
        root.setOnApplyWindowInsetsListener { v, i ->
            if (android.os.Build.VERSION.SDK_INT >= 30) {
                val b = i.getInsets(WindowInsets.Type.systemBars())
                v.setPadding(b.left, b.top, b.right, b.bottom)
            } else {
                v.setPadding(
                    i.systemWindowInsetLeft,
                    i.systemWindowInsetTop,
                    i.systemWindowInsetRight,
                    i.systemWindowInsetBottom,
                )
            }
            i
        }
        val toolbar =
            MaterialToolbar(this).apply {
                title = "Native Sketch"
                menu.add("Document").setShowAsAction(0)
                setOnMenuItemClickListener {
                    documents()
                    true
                }
            }
        root.addView(toolbar)
        val row = LinearLayout(this)
        tools =
            com.google.android.material.button.MaterialButtonToggleGroup(this).apply {
                isSingleSelection = true
                isSelectionRequired = true
            }
        val penButton =
            MaterialButton(this).apply {
                id = View.generateViewId()
                text = "Pen"
                minWidth = 0
                minHeight = dp(48)
            }
        val selectButton =
            MaterialButton(this).apply {
                id = View.generateViewId()
                text = "Select"
                minWidth = 0
                minHeight = dp(48)
            }
        tools.addView(penButton, LinearLayout.LayoutParams(0, dp(52), 1f))
        tools.addView(selectButton, LinearLayout.LayoutParams(0, dp(52), 1f))
        tools.check(if (pen) penButton.id else selectButton.id)
        tools.addOnButtonCheckedListener { _, checkedId, isChecked ->
            if (isChecked && !busy)
                safe {
                    canvas.cancel()
                    pen = checkedId == penButton.id
                    if (pen) checked(act(11))
                }
        }
        row.addView(tools)
        undo = button("Undo") { checked(act(8)) }
        redo = button("Redo") { checked(act(9)) }
        row.addView(undo)
        row.addView(redo)
        for (i in 0 until row.childCount) {
            row.getChildAt(i).layoutParams = LinearLayout.LayoutParams(0, dp(52), 1f)
            row.getChildAt(i).minimumWidth = 0
        }
        tools.layoutParams = LinearLayout.LayoutParams(0, dp(52), 2f)
        root.addView(row)
        canvas = SketchView()
        if (retained != null) {
            canvas.zoom = retained.zoom
            canvas.restoreCenterX = retained.ox
            canvas.restoreCenterY = retained.oy
        }
        val workspace = LinearLayout(this)
        workspace.addView(canvas, LinearLayout.LayoutParams(0, -1, 1f))
        sidePanel = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(dp(8), dp(8), dp(8), dp(8))
            addView(TextView(this@MainActivity).apply { text = "Drawing controls" })
            addView(button("Pen style") { style() })
            addView(button("Selection…") { selection() })
            addView(button("Fit drawing") { canvas.fit() })
            addView(button("Zoom in") { canvas.scaleAt(1.25, canvas.width/2.0, canvas.height/2.0) })
            addView(button("Zoom out") { canvas.scaleAt(.8, canvas.width/2.0, canvas.height/2.0) })
        }
        val inspectorScroll = ScrollView(this).apply {
            isFillViewport = true
            addView(sidePanel, ViewGroup.LayoutParams(-1, -2))
        }
        workspace.addView(inspectorScroll, LinearLayout.LayoutParams(dp(192), -1))
        root.addView(workspace, LinearLayout.LayoutParams(-1, 0, 1f))
        status =
            TextView(this).apply {
                setPadding(dp(12), dp(4), dp(12), dp(4))
                accessibilityLiveRegion = View.ACCESSIBILITY_LIVE_REGION_POLITE
            }
        root.addView(status)
        val bottom = LinearLayout(this)
        bottomBar = bottom
        bottom.addView(button("Style") { style() })
        bottom.addView(button("Selection") { selection() })
        bottom.addView(button("Zoom") { zoom() })
        root.addView(bottom)
        root.addOnLayoutChangeListener { _, left, _, right, _, _, _, _, _ ->
            val expanded = right - left >= dp(840)
            inspectorScroll.visibility = if (expanded) View.VISIBLE else View.GONE
            bottomBar.visibility = if (expanded) View.GONE else View.VISIBLE
        }
        canvas.setOnDragListener { _, event ->
            when (event.action) {
                DragEvent.ACTION_DRAG_STARTED -> acceptsDrop(event.clipDescription)
                DragEvent.ACTION_DROP -> {
                    val uri = event.clipData?.takeIf { it.itemCount == 1 }?.getItemAt(0)?.uri
                    if (uri?.scheme == "content" && !busy) {
                        // Keep the transient grant alive through a possible Save/Discard/Cancel dialog.
                        dropPermission?.release()
                        dropPermission = requestDragAndDropPermissions(event)
                        receiveUri(uri)
                        true
                    } else false
                }
                else -> true
            }
        }
        setContentView(root)
        refresh()
        if (retained == null) receiveIntent(intent)
        else if (pendingUri != null && !busy) canvas.post { replace(4) }
        if (
            intent.getBooleanExtra("replay", false) &&
                applicationInfo.flags and android.content.pm.ApplicationInfo.FLAG_DEBUGGABLE != 0
        ) {
            intent.removeExtra("replay")
            canvas.post { safe { replay() } }
        }
    }

    override fun onPause() {
        if (handle != 0L) canvas.cancel()
        super.onPause()
    }

    override fun onWindowFocusChanged(hasFocus: Boolean) {
        super.onWindowFocusChanged(hasFocus)
        if (!hasFocus && ::canvas.isInitialized) { spaceHeld = false; canvas.cancel() }
    }

    private data class Retained(
        val handle: Long,
        val busy: Boolean,
        val bytes: ByteArray?,
        val revision: Long,
        val replace: Int,
        val pen: Boolean,
        val ink: Int,
        val width: Double,
        val zoom: Double,
        val ox: Double,
        val oy: Double,
        val uri: android.net.Uri?,
        val permission: android.view.DragAndDropPermissions?,
    )

    override fun onRetainCustomNonConfigurationInstance(): Any {
        canvas.cancel()
        return Retained(
            handle,
            busy,
            savedBytes,
            savedRevision,
            pendingReplace,
            pen,
            ink,
            width,
            canvas.zoom,
            (canvas.width / 2.0 - canvas.ox) / canvas.zoom,
            (canvas.height / 2.0 - canvas.oy) / canvas.zoom,
            pendingUri,
            dropPermission,
        )
    }

    override fun onDestroy() {
        if (!isChangingConfigurations) {
            if (handle != 0L) engine.free(handle)
            dropPermission?.release()
        }
        handle = 0
        super.onDestroy()
    }

    private fun requestClose() {
        if (!busy) {
            canvas.cancel()
            replace(3)
        }
    }

    override fun onKeyDown(keyCode: Int, event: KeyEvent): Boolean =
        handleEditorKey(event) || super.onKeyDown(keyCode, event)

    override fun onKeyShortcut(keyCode: Int, event: KeyEvent): Boolean =
        handleEditorKey(event) || super.onKeyShortcut(keyCode, event)

    override fun onKeyUp(keyCode: Int, event: KeyEvent): Boolean {
        if (keyCode == KeyEvent.KEYCODE_SPACE && spaceHeld) { spaceHeld = false; return true }
        return super.onKeyUp(keyCode, event)
    }

    private fun handleEditorKey(event: KeyEvent): Boolean {
        if (currentFocus is EditText || busy) return false
        if (event.keyCode == KeyEvent.KEYCODE_SPACE && canvas.hasFocus()) {
            if (event.action == KeyEvent.ACTION_DOWN && !spaceHeld) canvas.cancel()
            spaceHeld = event.action == KeyEvent.ACTION_DOWN
            return true
        }
        if (event.action != KeyEvent.ACTION_DOWN || event.repeatCount != 0)
            return false
        val ctrl = event.isCtrlPressed
        val action: (() -> Unit)? = when {
            ctrl && event.keyCode == KeyEvent.KEYCODE_Z -> ({ checked(act(if(event.isShiftPressed) 9 else 8)) })
            ctrl && event.keyCode == KeyEvent.KEYCODE_Y -> ({ checked(act(9)) })
            ctrl && event.keyCode == KeyEvent.KEYCODE_S -> ({ save() })
            ctrl && event.keyCode == KeyEvent.KEYCODE_O -> ({ replace(2) })
            ctrl && event.keyCode == KeyEvent.KEYCODE_N -> ({ replace(1) })
            !ctrl && event.keyCode == KeyEvent.KEYCODE_P -> ({ tools.check(tools.getChildAt(0).id) })
            !ctrl && event.keyCode == KeyEvent.KEYCODE_V -> ({ tools.check(tools.getChildAt(1).id) })
            event.keyCode == KeyEvent.KEYCODE_ESCAPE -> ({ canvas.cancel() })
            event.keyCode == KeyEvent.KEYCODE_FORWARD_DEL || event.keyCode == KeyEvent.KEYCODE_DEL -> ({
                if(engine.state(handle)[1] != 0.0) checked(act(4))
            })
            !ctrl && event.keyCode == KeyEvent.KEYCODE_F -> ({ canvas.fit() })
            event.keyCode == KeyEvent.KEYCODE_EQUALS || event.keyCode == KeyEvent.KEYCODE_PLUS -> ({ canvas.scaleAt(1.25,canvas.width/2.0,canvas.height/2.0) })
            event.keyCode == KeyEvent.KEYCODE_MINUS -> ({ canvas.scaleAt(.8,canvas.width/2.0,canvas.height/2.0) })
            else -> null
        }
        if (action == null) return false
        safe { canvas.cancel(); action() }
        return true
    }

    private fun acceptsDrop(description: android.content.ClipDescription?): Boolean =
        description?.let { it.hasMimeType("text/uri-list") ||
            it.hasMimeType("application/octet-stream") ||
            it.hasMimeType("application/x-native-sketch") } == true

    private var dropPermission: android.view.DragAndDropPermissions? = null

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        receiveIntent(intent)
    }

    private fun receiveIntent(incoming: Intent) {
        if (incoming.action != Intent.ACTION_VIEW) return
        val uri = incoming.data ?: return
        if (uri.scheme != "content") return
        if (busy) { showError("Finish the current file operation before opening another drawing"); return }
        // Only retain permissions actually offered by the sender.
        if (incoming.flags and Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION != 0 &&
            incoming.flags and Intent.FLAG_GRANT_READ_URI_PERMISSION != 0) {
            try { contentResolver.takePersistableUriPermission(uri, Intent.FLAG_GRANT_READ_URI_PERMISSION) }
            catch (_: SecurityException) { /* The transient incoming grant can still be used. */ }
        }
        receiveUri(uri)
    }

    private fun receiveUri(uri: android.net.Uri) {
        canvas.cancel()
        pendingUri = uri
        replace(4)
    }

    private fun importUri(uri: android.net.Uri) {
        val bytes = contentResolver.openInputStream(uri)?.use { stream ->
            val out = java.io.ByteArrayOutputStream()
            val chunk = ByteArray(8192)
            val limit = 64 * 1024 * 1024
            while (out.size() <= limit) {
                val n = stream.read(chunk, 0, minOf(chunk.size, limit + 1 - out.size()))
                if (n < 0) break
                out.write(chunk, 0, n)
            }
            out.toByteArray()
        } ?: error("Provider did not open this document")
        require(bytes.size <= 64 * 1024 * 1024) { "Document exceeds 64 MiB" }
        checked(engine.importBytes(handle, bytes))
    }

    private fun completeIncoming() {
        try { importUri(pendingUri ?: error("File request expired; open the drawing again")) }
        finally { pendingUri = null; dropPermission?.release(); dropPermission = null }
    }

    private fun refresh() {
        if (!::canvas.isInitialized) return
        val s = engine.state(handle)
        for (i in 0 until tools.childCount) tools.getChildAt(i).isEnabled = !busy
        undo.isEnabled = !busy && s[2] != 0.0
        redo.isEnabled = !busy && s[3] != 0.0
        status.text =
            "${if(pen) "Pen" else "Select"} · ${s[0].toInt()} strokes · ${(canvas.zoom*100).roundToInt()}%${if(s[4]!=0.0) " · Unsaved" else ""}"
        canvas.contentDescription =
            "Drawing canvas. ${status.text}. ${if(s[1]!=0.0) "Stroke selected" else "No selection"}"
        canvas.invalidate()
    }

    private fun documents() {
        if (busy) return
        canvas.cancel()
        MaterialAlertDialogBuilder(this)
            .setTitle("Document")
            .setItems(arrayOf("New", "Open…", "Save a copy…")) { _, n ->
                when (n) {
                    0 -> replace(1)
                    1 -> replace(2)
                    2 -> save()
                }
            }
            .show()
    }

    private fun replace(kind: Int) {
        fun perform() {
            when (kind) {
                1 -> safe { checked(act(10)) }
                2 -> open()
                3 -> finish()
                4 -> safe { completeIncoming() }
            }
        }
        if (engine.state(handle)[4] == 0.0) {
            perform()
            return
        }
        replaceDialog?.dismiss()
        replaceDialog =
            MaterialAlertDialogBuilder(this)
                .setTitle("Unsaved drawing")
                .setMessage(
                    if (kind == 3) "Save your drawing before closing it?"
                    else "Save your drawing before replacing it?"
                )
                .setPositiveButton("Save") { _, _ ->
                    pendingReplace = kind
                    save()
                }
                .setNegativeButton("Discard") { _, _ -> perform() }
                .setNeutralButton("Cancel") { _, _ -> pendingUri = null; dropPermission?.release(); dropPermission = null }
                .setOnCancelListener { pendingUri = null; dropPermission?.release(); dropPermission = null }
                .setOnDismissListener { replaceDialog = null }
                .show()
    }

    private fun open() {
        canvas.cancel()
        busy = true
        refresh()
        try { startActivityForResult(
            Intent(Intent.ACTION_OPEN_DOCUMENT)
                .addCategory(Intent.CATEGORY_OPENABLE)
                .setType("*/*"),
            1,
        ) } catch (error: Exception) { busy = false; throw error }
    }

    private fun save() {
        safe {
            try {
            canvas.cancel()
            savedBytes = engine.export(handle)
            savedRevision = engine.revision(handle)
            busy = true
            refresh()
            startActivityForResult(
                Intent(Intent.ACTION_CREATE_DOCUMENT)
                    .addCategory(Intent.CATEGORY_OPENABLE)
                    .setType("application/octet-stream")
                    .putExtra(Intent.EXTRA_TITLE, "Drawing.nsketch"),
                2,
            )
            } catch (error: Exception) {
                busy = false; savedBytes = null; pendingReplace = 0
                throw error
            }
        }
    }

    @Deprecated("Native document result bridge")
    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        busy = false
        val uri = data?.data
        if (resultCode == RESULT_OK && uri != null)
            safe {
                if (requestCode == 1) {
                    importUri(uri)
                }
                if (requestCode == 2) {
                    val snapshot = savedBytes ?: error("Save was interrupted; please save again")
                    contentResolver.openOutputStream(uri, "w")?.use {
                        it.write(snapshot)
                        it.flush()
                    } ?: error("Provider did not accept this document")
                    checked(engine.markSaved(handle, savedRevision))
                    when (pendingReplace) {
                        1 -> checked(act(10))
                        2 -> open()
                        3 -> finish()
                        4 -> completeIncoming()
                    }
                }
            }
        savedBytes = null
        pendingReplace = 0
        pendingUri = null
        dropPermission?.release(); dropPermission = null
        refresh()
    }

    private fun style() {
        var pendingInk = ink
        val col =
            LinearLayout(this).apply {
                orientation = LinearLayout.VERTICAL
                setPadding(dp(24), 0, dp(24), 0)
            }
        val field =
            EditText(this).apply {
                hint = "Stroke width (document units)"
                inputType =
                    android.text.InputType.TYPE_CLASS_NUMBER or
                        android.text.InputType.TYPE_NUMBER_FLAG_DECIMAL
                setText(width.toString())
            }
        col.addView(field)
        val colors = LinearLayout(this)
        listOf(
                Color.rgb(32, 49, 75),
                Color.rgb(189, 45, 64),
                Color.rgb(21, 126, 94),
                Color.rgb(86, 72, 190),
            )
            .forEachIndexed { i, c ->
                colors.addView(
                    button(listOf("Ink", "Red", "Green", "Violet")[i]) { pendingInk = c }
                )
            }
        val scroll = HorizontalScrollView(this)
        scroll.addView(colors)
        col.addView(scroll)
        val dialog =
            MaterialAlertDialogBuilder(this)
                .setTitle("Pen style")
                .setView(col)
                .setPositiveButton("Apply", null)
                .setNegativeButton("Cancel", null)
                .create()
        dialog.setOnShowListener {
            dialog.getButton(-1).setOnClickListener {
                val w = field.text.toString().toDoubleOrNull()
                if (w == null || !w.isFinite() || w !in 0.1..100.0) field.error = "Enter 0.1–100"
                else {
                    width = w
                    ink = pendingInk
                    dialog.dismiss()
                }
            }
        }
        dialog.show()
    }

    private fun selection() {
        var update: () -> Unit = {}
        val s = engine.state(handle)
        val col =
            LinearLayout(this).apply {
                orientation = LinearLayout.VERTICAL
                setPadding(dp(24), 0, dp(24), 0)
            }
        col.addView(
            button("Previous stroke") {
                    tools.check(tools.getChildAt(1).id)
                    checked(act(3, -1.0))
                    update()
                }
                .apply { isEnabled = s[0] > 0 }
        )
        col.addView(
            button("Next stroke") {
                    tools.check(tools.getChildAt(1).id)
                    checked(act(3, 1.0))
                    update()
                }
                .apply { isEnabled = s[0] > 0 }
        )
        col.addView(
            button("Delete selected") {
                    checked(act(4))
                    update()
                }
                .apply { isEnabled = s[1] > 0 }
        )
        val field =
            EditText(this).apply {
                hint = "Uniform scale (%)"
                setText("100")
                inputType =
                    android.text.InputType.TYPE_CLASS_NUMBER or
                        android.text.InputType.TYPE_NUMBER_FLAG_DECIMAL
            }
        col.addView(field)
        col.addView(
            button("Scale selected") {
                    val v = field.text.toString().toDoubleOrNull()
                    require(v != null && v.isFinite() && v > 0 && v <= 10000) {
                        "Enter a scale above 0 and at most 10000%"
                    }
                    val st = engine.state(handle)
                    checked(act(5))
                    try {
                        checked(act(6, v / 100, st[6], st[7]))
                        checked(act(7))
                    } catch (e: Exception) {
                        act(0)
                        throw e
                    }
                }
                .apply { isEnabled = s[1] > 0 }
        )
        update = {
            val selected = engine.state(handle)[1] != 0.0
            col.getChildAt(2).isEnabled = selected
            col.getChildAt(4).isEnabled = selected
        }
        MaterialAlertDialogBuilder(this)
            .setTitle("Selection")
            .setView(col)
            .setPositiveButton("Done", null)
            .show()
    }

    private fun zoom() {
        MaterialAlertDialogBuilder(this)
            .setTitle("Canvas zoom")
            .setItems(arrayOf("Zoom in", "Zoom out", "Reset to 100%", "Fit drawing")) { _, n ->
                if (n == 3) canvas.fit()
                else if (n == 2) {
                    canvas.zoom = 1.0
                    canvas.ox = 0.0
                    canvas.oy = 0.0
                } else
                    canvas.scaleAt(
                        if (n == 0) 1.25 else 0.8,
                        canvas.width / 2.0,
                        canvas.height / 2.0,
                    )
                refresh()
            }
            .show()
    }

    inner class SketchView : View(this@MainActivity) {
        var zoom = 1.0
        var ox = 0.0
        var oy = 0.0
        var restoreCenterX: Double? = null
        var restoreCenterY: Double? = null
        private var pointer = -1
        private var id = 0L
        private var navigating = false
        private var mousePan = false
        var contextMenu: PopupMenu? = null
        private var resizing = false
        private var anchorX = 0.0
        private var anchorY = 0.0
        private var initialDistance = 1.0
        private var reseed = false
        private var prevX = 0.0
        private var prevY = 0.0
        private var prevSpan = 1.0
        private val paint = Paint(Paint.ANTI_ALIAS_FLAG)

        init {
            isFocusable = true
            importantForAccessibility = IMPORTANT_FOR_ACCESSIBILITY_YES
        }

        fun cancel() {
            if (handle != 0L) act(0)
            pointer = -1
            resizing = false
            navigating = false
            mousePan = false
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
            val commands = engine.render(handle)
            if (commands.isEmpty()) { zoom = 1.0; ox = 0.0; oy = 0.0; return }
            var minX = Double.POSITIVE_INFINITY; var minY = minX
            var maxX = Double.NEGATIVE_INFINITY; var maxY = maxX
            for (i in commands.indices step 12) {
                val count = when(commands[i+1].toInt()) { 0,1 -> 1; 2 -> 3; else -> 0 }
                for(j in 0 until count) {
                    val x = commands[i+6+j*2]; val y = commands[i+7+j*2]
                    minX = min(minX,x); maxX = max(maxX,x)
                    minY = min(minY,y); maxY = max(maxY,y)
                }
            }
            if (!minX.isFinite()) return
            zoom = min((width-dp(48)).coerceAtLeast(1)/(maxX-minX).coerceAtLeast(1.0),
                (height-dp(48)).coerceAtLeast(1)/(maxY-minY).coerceAtLeast(1.0)).coerceAtMost(8.0)
            ox = width/2.0 - (minX+maxX)/2*zoom
            oy = height/2.0 - (minY+maxY)/2*zoom
        }

        override fun onResolvePointerIcon(event: MotionEvent, pointerIndex: Int): PointerIcon =
            PointerIcon.getSystemIcon(context, if(spaceHeld) PointerIcon.TYPE_GRAB else if(pen) PointerIcon.TYPE_CROSSHAIR else PointerIcon.TYPE_ARROW)

        override fun onGenericMotionEvent(e: MotionEvent): Boolean {
            if (busy) return true
            if (e.actionMasked == MotionEvent.ACTION_SCROLL) {
                cancel()
                val dy = e.getAxisValue(MotionEvent.AXIS_VSCROLL).toDouble()
                val dx = e.getAxisValue(MotionEvent.AXIS_HSCROLL).toDouble()
                if (e.metaState and KeyEvent.META_CTRL_ON != 0) scaleAt(exp(dy * .12), e.x.toDouble(), e.y.toDouble())
                else { ox += dx * dp(48); oy += dy * dp(48) }
                refresh(); return true
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
            c.drawColor(Color.rgb(250, 250, 248))
            c.save()
            c.translate(ox.toFloat(), oy.toFloat())
            c.scale(zoom.toFloat(), zoom.toFloat())
            val commands = engine.render(handle)
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
            val s = engine.state(handle)
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

        override fun onTouchEvent(e: MotionEvent): Boolean {
            if (busy) return true
            try {
                when (e.actionMasked) {
                    MotionEvent.ACTION_DOWN -> {
                        requestFocus()
                        pointer = e.getPointerId(0)
                        navigating = false
                        if (e.buttonState and MotionEvent.BUTTON_SECONDARY != 0) {
                            cancel()
                            contextMenu = PopupMenu(this@MainActivity, this).apply {
                                menu.add("Fit drawing").setOnMenuItemClickListener { fit(); refresh(); true }
                                menu.add("Selection…").setOnMenuItemClickListener { selection(); true }
                                show()
                            }
                            return true
                        }
                        if (spaceHeld || e.buttonState and MotionEvent.BUTTON_TERTIARY != 0) {
                            navigating = true; mousePan = true
                            prevX = e.x.toDouble(); prevY = e.y.toDouble()
                            return true
                        }
                        val x = (e.x - ox) / zoom
                        val y = (e.y - oy) / zoom
                        if (pen) {
                            checked(
                                engine.begin(
                                    handle,
                                    pointer,
                                    when (e.getToolType(0)) {
                                        MotionEvent.TOOL_TYPE_STYLUS, MotionEvent.TOOL_TYPE_ERASER -> 1
                                        MotionEvent.TOOL_TYPE_MOUSE -> 0
                                        else -> if(e.isFromSource(InputDevice.SOURCE_MOUSE)) 0 else 2
                                    },
                                    ink,
                                    this@MainActivity.width,
                                )
                            )
                            append(e, 0)
                        } else {
                            val s = engine.state(handle)
                            if (
                                s[1] != 0.0 &&
                                    hypot(x - s[6] - s[8], y - s[7] - s[9]) * zoom < dp(24)
                            ) {
                                checked(act(5))
                                resizing = true
                                anchorX = s[6]
                                anchorY = s[7]
                                initialDistance =
                                    hypot(x - anchorX, y - anchorY).coerceAtLeast(.001)
                            } else checked(act(2, x, y, dp(16) / zoom))
                        }
                    }
                    MotionEvent.ACTION_POINTER_DOWN -> {
                        act(0)
                        pointer = -1
                        resizing = false
                        navigating = true
                        if (e.pointerCount >= 2) {
                            prevX = (e.getX(0) + e.getX(1)) / 2.0
                            prevY = (e.getY(0) + e.getY(1)) / 2.0
                            prevSpan =
                                hypot(
                                        (e.getX(1) - e.getX(0)).toDouble(),
                                        (e.getY(1) - e.getY(0)).toDouble(),
                                    )
                                    .coerceAtLeast(1.0)
                        }
                    }
                    MotionEvent.ACTION_MOVE -> {
                        if (mousePan) {
                            ox += e.x - prevX; oy += e.y - prevY
                            prevX = e.x.toDouble(); prevY = e.y.toDouble()
                        } else if (navigating && e.pointerCount >= 2) {
                            val x = (e.getX(0) + e.getX(1)) / 2.0
                            val y = (e.getY(0) + e.getY(1)) / 2.0
                            val span =
                                hypot(
                                        (e.getX(1) - e.getX(0)).toDouble(),
                                        (e.getY(1) - e.getY(0)).toDouble(),
                                    )
                                    .coerceAtLeast(1.0)
                            if (!reseed) {
                                scaleAt(span / prevSpan, prevX, prevY)
                                ox += x - prevX
                                oy += y - prevY
                            }
                            reseed = false
                            prevX = x
                            prevY = y
                            prevSpan = span
                        } else if (!navigating && pointer >= 0) {
                            val idx = e.findPointerIndex(pointer)
                            if (idx < 0) cancel()
                            else if (resizing)
                                checked(
                                    act(
                                        6,
                                        hypot(
                                            (e.getX(idx) - ox) / zoom - anchorX,
                                            (e.getY(idx) - oy) / zoom - anchorY,
                                        ) / initialDistance,
                                        anchorX,
                                        anchorY,
                                    )
                                )
                            else if (pen) append(e, idx)
                        }
                    }
                    MotionEvent.ACTION_UP -> {
                        if (!navigating && pointer >= 0) {
                            if (resizing) checked(act(7))
                            else if (pen) {
                                append(e, 0)
                                checked(act(1, pointer.toDouble()))
                            }
                        }
                        cancel()
                    }
                    MotionEvent.ACTION_POINTER_UP -> {
                        reseed = true
                    }
                    MotionEvent.ACTION_CANCEL -> cancel()
                }
            } catch (ex: Exception) {
                cancel()
                showError(ex.message ?: "Input failed")
            }
            refresh()
            return true
        }

        private fun append(e: MotionEvent, index: Int) {
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
            for (h in 0 until e.historySize) checked(
                engine.sample(
                    handle,
                    pointer,
                    (e.getHistoricalX(index, h) - ox) / zoom,
                    (e.getHistoricalY(index, h) - oy) / zoom,
                    e.getHistoricalEventTime(h) / 1000.0,
                    pressure(e.getHistoricalPressure(index, h)),
                    stylus && e.getHistoricalPressure(index, h).isFinite(),
                    ++id,
                )
            )
            checked(
                engine.sample(
                    handle,
                    pointer,
                    (e.getX(index) - ox) / zoom,
                    (e.getY(index) - oy) / zoom,
                    e.eventTime / 1000.0,
                    pressure(e.getPressure(index)),
                    stylus && e.getPressure(index).isFinite(),
                    ++id,
                )
            )
        }
    }

    private fun replay() {
        checked(act(10))
        checked(engine.begin(handle, 7, 1, ink, 4.0))
        checked(engine.sample(handle, 7, 80.0, 90.0, 1.0, .5, true, 1))
        checked(engine.sample(handle, 7, 220.0, 180.0, 2.0, 1.0, true, 2))
        checked(act(1, 7.0))
        check(engine.state(handle)[0] == 1.0)
        checked(act(3, 1.0))
        checked(act(5))
        checked(act(6, 1.4, 80.0, 90.0))
        checked(act(0))
        val before = engine.export(handle)
        checked(act(8))
        check(engine.state(handle)[0] == 0.0)
        checked(act(9))
        check(before.contentEquals(engine.export(handle)))
        check(engine.importBytes(handle, byteArrayOf(1, 2, 3)) != 0)
        check(before.contentEquals(engine.export(handle)))
        checked(engine.begin(handle, 9, 2, ink, 4.0))
        checked(engine.sample(handle, 9, 20.0, 20.0, 3.0, 0.0, false, 3))
        checked(act(0))
        check(engine.state(handle)[0] == 1.0)
        val revision = engine.revision(handle)
        check(engine.state(handle)[4] == 1.0)
        checked(engine.markSaved(handle, revision))
        check(engine.state(handle)[4] == 0.0)
        checked(engine.importBytes(handle, before))
        check(engine.state(handle)[0] == 1.0)
        check(engine.render(handle).isNotEmpty())
        val props =
            Array(2) { i ->
                MotionEvent.PointerProperties().apply {
                    id = i
                    toolType = MotionEvent.TOOL_TYPE_FINGER
                }
            }
        val coords =
            Array(2) { i ->
                MotionEvent.PointerCoords().apply {
                    x = 100f + i * 80
                    y = 100f
                    pressure = 1f
                    size = 1f
                }
            }
        fun event(action: Int, count: Int, time: Long) {
            val e =
                MotionEvent.obtain(
                    1000,
                    time,
                    action,
                    count,
                    props,
                    coords,
                    0,
                    0,
                    1f,
                    1f,
                    0,
                    0,
                    android.view.InputDevice.SOURCE_TOUCHSCREEN,
                    0,
                )
            canvas.onTouchEvent(e)
            e.recycle()
        }
        event(MotionEvent.ACTION_DOWN, 1, 1000)
        check(engine.state(handle)[5] == 1.0)
        event(
            MotionEvent.ACTION_POINTER_DOWN or (1 shl MotionEvent.ACTION_POINTER_INDEX_SHIFT),
            2,
            1010,
        )
        check(engine.state(handle)[5] == 0.0)
        check(engine.state(handle)[0] == 1.0)
        event(MotionEvent.ACTION_UP, 1, 1020)
        check(engine.state(handle)[0] == 1.0)
        checked(engine.begin(handle, 12, 0, ink, 4.0))
        checked(engine.sample(handle, 12, 40.0, 40.0, 4.0, 0.0, false, 4))
        checked(act(1, 12.0))
        val closeSnapshot = engine.export(handle)
        onBackPressedDispatcher.onBackPressed()
        check(replaceDialog?.isShowing == true && !isFinishing)
        replaceDialog!!.getButton(android.content.DialogInterface.BUTTON_NEUTRAL).performClick()
        check(!isFinishing && closeSnapshot.contentEquals(engine.export(handle)))
        check(engine.state(handle)[4] == 1.0)
        fun key(code: Int, meta: Int = 0, action: Int = KeyEvent.ACTION_DOWN) {
            val event = KeyEvent(1000, 1000, action, code, 0, meta)
            if(action == KeyEvent.ACTION_UP) onKeyUp(code, event)
            else if(meta and KeyEvent.META_CTRL_ON != 0) onKeyShortcut(code, event)
            else onKeyDown(code, event)
        }
        canvas.requestFocus()
        key(KeyEvent.KEYCODE_Z, KeyEvent.META_CTRL_ON)
        check(engine.state(handle)[0] == 1.0)
        key(KeyEvent.KEYCODE_Z, KeyEvent.META_CTRL_ON or KeyEvent.META_SHIFT_ON)
        check(engine.state(handle)[0] == 2.0)
        key(KeyEvent.KEYCODE_V)
        check(!pen)
        checked(act(3, 1.0))
        val desktopSnapshot = engine.export(handle)
        val historyBefore = engine.state(handle).copyOf()
        canvas.layout(canvas.left, canvas.top, canvas.right+100, canvas.bottom+50)
        check(desktopSnapshot.contentEquals(engine.export(handle)))
        check(engine.state(handle)[1] == historyBefore[1] && engine.state(handle)[2] == historyBefore[2])
        key(KeyEvent.KEYCODE_P)
        event(MotionEvent.ACTION_DOWN, 1, 2000)
        check(engine.state(handle)[5] == 1.0)
        canvas.layout(canvas.left, canvas.top, canvas.right-100, canvas.bottom-50)
        check(engine.state(handle)[5] == 0.0)
        check(desktopSnapshot.contentEquals(engine.export(handle)))
        key(KeyEvent.KEYCODE_SPACE)
        val oldOx = canvas.ox
        event(MotionEvent.ACTION_DOWN, 1, 2010)
        coords[0].x += 30f
        event(MotionEvent.ACTION_MOVE, 1, 2020)
        event(MotionEvent.ACTION_UP, 1, 2030)
        key(KeyEvent.KEYCODE_SPACE, action = KeyEvent.ACTION_UP)
        check(canvas.ox == oldOx + 30 && desktopSnapshot.contentEquals(engine.export(handle)))
        val wheel = MotionEvent.obtain(1000, 2040, MotionEvent.ACTION_SCROLL, 1,
            props, Array(1) { MotionEvent.PointerCoords().apply {
                x=100f; y=100f; setAxisValue(MotionEvent.AXIS_VSCROLL, 1f)
            } }, KeyEvent.META_CTRL_ON, 0, 1f, 1f, 0, 0, InputDevice.SOURCE_MOUSE, 0)
        val oldZoom = canvas.zoom
        canvas.onGenericMotionEvent(wheel); wheel.recycle()
        check(canvas.zoom > oldZoom && desktopSnapshot.contentEquals(engine.export(handle)))
        key(KeyEvent.KEYCODE_F)
        check(canvas.zoom.isFinite() && canvas.zoom > 0)
        val secondary = MotionEvent.obtain(1000, 2050, MotionEvent.ACTION_DOWN, 1,
            props, coords, 0, MotionEvent.BUTTON_SECONDARY, 1f, 1f, 0, 0, InputDevice.SOURCE_MOUSE, 0)
        canvas.onTouchEvent(secondary); secondary.recycle()
        check(engine.state(handle)[5] == 0.0 && desktopSnapshot.contentEquals(engine.export(handle)))
        canvas.contextMenu?.dismiss()
        val field = EditText(this)
        (canvas.parent as LinearLayout).addView(field)
        field.requestFocus()
        val wasPen = pen
        key(KeyEvent.KEYCODE_V)
        check(pen == wasPen && desktopSnapshot.contentEquals(engine.export(handle)))
        (canvas.parent as LinearLayout).removeView(field)
        canvas.requestFocus()
        // The same incoming URI path used by intents and drops must guard dirty work.
        receiveUri(android.net.Uri.parse("content://dev.nativesketch.invalid/replay.nsketch"))
        check(replaceDialog?.isShowing == true)
        replaceDialog!!.getButton(android.content.DialogInterface.BUTTON_NEUTRAL).performClick()
        check(pendingUri == null && desktopSnapshot.contentEquals(engine.export(handle)))
        check(acceptsDrop(android.content.ClipDescription("drawing", arrayOf("application/x-native-sketch"))))
        check(acceptsDrop(android.content.ClipDescription("drawing", arrayOf("application/octet-stream"))))
        check(!acceptsDrop(android.content.ClipDescription("image", arrayOf("image/png"))))
        // Export the actual native input result, checking the NSKETCH1 device field.
        for ((tool, source, device) in listOf(
            Triple(MotionEvent.TOOL_TYPE_MOUSE, InputDevice.SOURCE_MOUSE, 0L),
            Triple(MotionEvent.TOOL_TYPE_STYLUS, InputDevice.SOURCE_STYLUS, 1L),
            Triple(MotionEvent.TOOL_TYPE_FINGER, InputDevice.SOURCE_TOUCHSCREEN, 2L))) {
            checked(act(10))
            props[0].toolType = tool
            for(action in listOf(MotionEvent.ACTION_DOWN, MotionEvent.ACTION_UP)) {
                val input = MotionEvent.obtain(3000, 3010, action, 1, props, coords,
                    0, 0, 1f, 1f, 0, 0, source, 0)
                canvas.onTouchEvent(input); input.recycle()
            }
            val exported = engine.export(handle)
            check(java.nio.ByteBuffer.wrap(exported).order(java.nio.ByteOrder.LITTLE_ENDIAN).getLong(24) == device)
        }
        checked(act(10))
        checked(engine.begin(handle, 22, 0, ink, 4.0))
        checked(engine.sample(handle, 22, -1e7, -1e7, 10.0, 0.0, false, 1))
        checked(engine.sample(handle, 22, 1e7, 1e7, 11.0, 0.0, false, 2))
        checked(act(1, 22.0))
        canvas.fit()
        val fitted = canvas.zoom
        check(fitted < .001)
        canvas.scaleAt(.8, canvas.width/2.0, canvas.height/2.0)
        check(canvas.zoom <= fitted)
        checked(engine.importBytes(handle, desktopSnapshot))
        canvas.fit()
        android.util.Log.i(
            "NativeSketchReplay",
            "PASS: JNI stroke, pressure, cancel, resize rollback, history, corrupt import, revision save, roundtrip, Back dispatcher cancel retains dirty drawing, desktop keyboard history/tools/Fit, resize cancellation/history, Space drag, Ctrl wheel, incoming URI dirty cancel, native MIME drops, exported mouse/stylus/touch provenance, large-fit zoom-out monotonicity",
        )
        refresh()
    }
}
