package dev.nativesketch

import android.content.Intent
import android.graphics.*
import android.view.*
import android.widget.*
import kotlin.math.*

internal class IncomingDocuments(private val host: MainActivity) {
    fun receiveIntent(incoming: Intent) {
        with(host) {
            if (incoming.action != Intent.ACTION_VIEW) return
            val uri = incoming.data ?: return
            if (uri.scheme != "content") return
            if (busy) {
                showError("Finish the current file operation before opening another drawing")
                return
            }
            // Only retain permissions actually offered by the sender.
            if (
                incoming.flags and Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION != 0 &&
                    incoming.flags and Intent.FLAG_GRANT_READ_URI_PERMISSION != 0
            ) {
                try {
                    contentResolver.takePersistableUriPermission(
                        uri,
                        Intent.FLAG_GRANT_READ_URI_PERMISSION,
                    )
                } catch (_: SecurityException) {
                    /* The transient incoming grant can still be used. */
                }
            }
            receiveUri(uri)
        }
    }

    fun receiveUri(uri: android.net.Uri) {
        with(host) {
            canvas.cancel()
            pendingUri = uri
            replace(4)
        }
    }

    fun importUri(uri: android.net.Uri) {
        with(host) {
            val bytes =
                contentResolver.openInputStream(uri)?.use { stream ->
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
    }

    fun completeIncoming() {
        with(host) {
            try {
                importUri(pendingUri ?: error("File request expired; open the drawing again"))
            } finally {
                pendingUri = null
                dropPermission?.release()
                dropPermission = null
            }
        }
    }
}
