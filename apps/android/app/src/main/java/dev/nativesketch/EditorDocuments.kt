package dev.nativesketch

import android.content.Intent
import android.graphics.*
import android.view.*
import android.widget.*
import com.google.android.material.dialog.MaterialAlertDialogBuilder
import kotlin.math.*

internal class EditorDocuments(private val host: MainActivity) {
    fun documents() {
        with(host) {
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
    }

    fun replace(kind: Int) {
        with(host) {
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
                    .setNeutralButton("Cancel") { _, _ ->
                        pendingUri = null
                        dropPermission?.release()
                        dropPermission = null
                    }
                    .setOnCancelListener {
                        pendingUri = null
                        dropPermission?.release()
                        dropPermission = null
                    }
                    .setOnDismissListener { replaceDialog = null }
                    .show()
        }
    }

    fun open() {
        with(host) {
            canvas.cancel()
            busy = true
            refresh()
            try {
                startActivityForResult(
                    Intent(Intent.ACTION_OPEN_DOCUMENT)
                        .addCategory(Intent.CATEGORY_OPENABLE)
                        .setType("*/*"),
                    1,
                )
            } catch (error: Exception) {
                busy = false
                throw error
            }
        }
    }

    fun save() {
        with(host) {
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
                    busy = false
                    savedBytes = null
                    pendingReplace = 0
                    throw error
                }
            }
        }
    }
}
