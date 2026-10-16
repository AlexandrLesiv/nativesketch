package dev.nativesketch

import android.content.Intent
import android.graphics.*
import android.view.*
import android.widget.*
import kotlin.math.*

internal class DocumentResults(private val host: MainActivity) {
    fun receive(requestCode: Int, resultCode: Int, data: Intent?) {
        with(host) {
            busy = false
            val uri = data?.data
            if (resultCode == android.app.Activity.RESULT_OK && uri != null)
                safe {
                    if (requestCode == 1) {
                        importUri(uri)
                    }
                    if (requestCode == 2) {
                        val snapshot =
                            savedBytes ?: error("Save was interrupted; please save again")
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
            dropPermission?.release()
            dropPermission = null
            refresh()
        }
    }
}
