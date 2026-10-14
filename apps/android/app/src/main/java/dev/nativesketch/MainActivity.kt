package dev.nativesketch

import android.app.Activity
import android.content.Intent
import android.graphics.*
import android.os.Bundle
import android.view.*
import android.widget.*
import com.google.android.material.appbar.MaterialToolbar
import com.google.android.material.button.MaterialButton
import com.google.android.material.dialog.MaterialAlertDialogBuilder
import kotlin.math.*

class MainActivity : Activity() {
    private val engine = Engine()
    private var handle = 0L
    private lateinit var canvas: SketchView
    private lateinit var status: TextView
    private lateinit var undo: MaterialButton
    private lateinit var redo: MaterialButton
    private var busy = false
    private var pendingReplace = 0 // 1 new, 2 open
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
        val retained = lastNonConfigurationInstance as? Retained
        handle = retained?.handle ?: engine.create()
        if (retained != null) {
            busy = retained.busy
            savedBytes = retained.bytes
            savedRevision = retained.revision
            pendingReplace = retained.replace
            pen = retained.pen
            ink = retained.ink
            width = retained.width
        }
        val root = LinearLayout(this).apply { orientation = LinearLayout.VERTICAL }
        root.setOnApplyWindowInsetsListener { v, i ->
            val b = i.getInsets(WindowInsets.Type.systemBars())
            v.setPadding(b.left, b.top, b.right, b.bottom)
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
        row.addView(button("Pen") { pen = true })
        row.addView(button("Select") { pen = false })
        undo = button("Undo") { checked(act(8)) }
        redo = button("Redo") { checked(act(9)) }
        row.addView(undo)
        row.addView(redo)
        for (i in 0 until row.childCount) {
            row.getChildAt(i).layoutParams = LinearLayout.LayoutParams(0, dp(52), 1f)
            row.getChildAt(i).minimumWidth = 0
        }
        root.addView(row)
        canvas = SketchView()
        if (retained != null) {
            canvas.zoom = retained.zoom
            canvas.restoreCenterX = retained.ox
            canvas.restoreCenterY = retained.oy
        }
        root.addView(canvas, LinearLayout.LayoutParams(-1, 0, 1f))
        status =
            TextView(this).apply {
                setPadding(dp(12), dp(4), dp(12), dp(4))
                accessibilityLiveRegion = View.ACCESSIBILITY_LIVE_REGION_POLITE
            }
        root.addView(status)
        val bottom = LinearLayout(this)
        bottom.addView(button("Style") { style() })
        bottom.addView(button("Selection") { selection() })
        bottom.addView(button("Zoom") { zoom() })
        root.addView(bottom)
        setContentView(root)
        refresh()
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
        if (!hasFocus && ::canvas.isInitialized) canvas.cancel()
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
    )

    override fun onRetainNonConfigurationInstance(): Any {
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
        )
    }

    override fun onDestroy() {
        if (handle != 0L && !isChangingConfigurations) engine.free(handle)
        handle = 0
        super.onDestroy()
    }

    override fun onKeyDown(keyCode: Int, event: android.view.KeyEvent): Boolean {
        if (keyCode == android.view.KeyEvent.KEYCODE_ESCAPE) {
            canvas.cancel()
            return true
        }
        return super.onKeyDown(keyCode, event)
    }

    private fun refresh() {
        if (!::canvas.isInitialized) return
        val s = engine.state(handle)
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
            if (kind == 1) safe { checked(act(10)) } else open()
        }
        if (engine.state(handle)[4] == 0.0) {
            perform()
            return
        }
        MaterialAlertDialogBuilder(this)
            .setTitle("Unsaved drawing")
            .setMessage("Save your drawing before replacing it?")
            .setPositiveButton("Save") { _, _ ->
                pendingReplace = kind
                save()
            }
            .setNegativeButton("Discard") { _, _ -> perform() }
            .setNeutralButton("Cancel", null)
            .show()
    }

    private fun open() {
        canvas.cancel()
        busy = true
        refresh()
        startActivityForResult(
            Intent(Intent.ACTION_OPEN_DOCUMENT)
                .addCategory(Intent.CATEGORY_OPENABLE)
                .setType("*/*"),
            1,
        )
    }

    private fun save() {
        safe {
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
                    val bytes =
                        contentResolver.openInputStream(uri)?.use { stream ->
                            val out = java.io.ByteArrayOutputStream()
                            val chunk = ByteArray(8192)
                            while (out.size() <= 64 * 1024 * 1024) {
                                val n =
                                    stream.read(
                                        chunk,
                                        0,
                                        minOf(chunk.size, 64 * 1024 * 1024 + 1 - out.size()),
                                    )
                                if (n < 0) break
                                out.write(chunk, 0, n)
                            }
                            out.toByteArray()
                        } ?: error("Provider did not open this document")
                    require(bytes.size <= 64 * 1024 * 1024) { "Document exceeds 64 MiB" }
                    checked(engine.importBytes(handle, bytes))
                }
                if (requestCode == 2) {
                    val snapshot = savedBytes ?: error("Save was interrupted; please save again")
                    contentResolver.openOutputStream(uri, "w")?.use {
                        it.write(snapshot)
                        it.flush()
                    } ?: error("Provider did not accept this document")
                    checked(engine.markSaved(handle, savedRevision))
                    if (pendingReplace == 1) checked(act(10)) else if (pendingReplace == 2) open()
                }
            }
        savedBytes = null
        pendingReplace = 0
        refresh()
    }

    private fun style() {
        val col =
            LinearLayout(this).apply {
                orientation = 1
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
                colors.addView(button(listOf("Ink", "Red", "Green", "Violet")[i]) { ink = c })
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
                orientation = 1
                setPadding(dp(24), 0, dp(24), 0)
            }
        col.addView(
            button("Previous stroke") {
                    checked(act(3, -1.0))
                    update()
                }
                .apply { isEnabled = s[0] > 0 }
        )
        col.addView(
            button("Next stroke") {
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
            .setItems(arrayOf("Zoom in", "Zoom out", "Reset to 100%")) { _, n ->
                if (n == 2) {
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
            invalidate()
        }

        fun scaleAt(factor: Double, x: Double, y: Double) {
            val next = (zoom * factor).coerceIn(.1, 8.0)
            ox = x - (x - ox) * next / zoom
            oy = y - (y - oy) * next / zoom
            zoom = next
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
                        val x = (e.x - ox) / zoom
                        val y = (e.y - oy) / zoom
                        if (pen) {
                            checked(
                                engine.begin(
                                    handle,
                                    pointer,
                                    if (e.getToolType(0) == MotionEvent.TOOL_TYPE_STYLUS) 1 else 2,
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
                        if (navigating && e.pointerCount >= 2) {
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
        android.util.Log.i(
            "NativeSketchReplay",
            "PASS: JNI stroke, pressure, cancel, resize rollback, history, corrupt import, revision save, roundtrip",
        )
        refresh()
    }
}
