package dev.nativesketch

/** Temporary native capture state; committed editing state remains in Rust. */
internal class PointerState {
    var pointer = -1
    var id = 0L
    var navigating = false
    var mousePan = false
    var resizing = false
    var anchorX = 0.0
    var anchorY = 0.0
    var initialDistance = 1.0
    var reseed = false
    var prevX = 0.0
    var prevY = 0.0
    var prevSpan = 1.0
}
