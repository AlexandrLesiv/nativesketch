package dev.nativesketch

/**
 * Carries native configuration across an Activity recreation without duplicating the Rust owner.
 */
internal class EditorRetention(private val host: MainActivity) {
    fun capture(): Retained =
        with(host) {
            canvas.cancel()
            Retained(
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
}
