package dev.nativesketch

/** JNI transports ABI 2 values only. Rust owns every document and edit operation. */
class Engine {
    companion object {
        init {
            System.loadLibrary("sketch_jni")
        }
    }

    external fun create(): Long

    external fun free(h: Long)

    external fun state(h: Long): DoubleArray

    external fun action(h: Long, op: Int, a: Double, b: Double, c: Double): Int

    external fun begin(h: Long, pointer: Int, device: Int, color: Int, width: Double): Int

    external fun sample(
        h: Long,
        pointer: Int,
        x: Double,
        y: Double,
        time: Double,
        pressure: Double,
        hasPressure: Boolean,
        id: Long,
    ): Int

    external fun render(h: Long): DoubleArray

    external fun export(h: Long): ByteArray

    external fun revision(h: Long): Long

    external fun importBytes(h: Long, bytes: ByteArray): Int

    external fun markSaved(h: Long, revision: Long): Int
}
