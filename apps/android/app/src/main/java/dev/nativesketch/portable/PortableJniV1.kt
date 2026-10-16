package dev.nativesketch.portable

import java.nio.ByteBuffer

private class PortableJniV1 : NativeCalls {
    external override fun apiVersion(): Int

    external override fun appNew(packet: ByteBuffer?, length: Long, receipt: ByteBuffer?): Int

    external override fun appCall(
        app: Long,
        packet: ByteBuffer?,
        length: Long,
        receipt: ByteBuffer?,
    ): Int

    external override fun appTakeWork(
        app: Long,
        packet: ByteBuffer?,
        length: Long,
        receipt: ByteBuffer?,
    ): Int

    external override fun workCopyInput(
        work: Long,
        source: ByteBuffer?,
        length: Long,
        receipt: ByteBuffer?,
    ): Int

    external override fun workCall(
        work: Long,
        packet: ByteBuffer?,
        length: Long,
        input: Long,
        receipt: ByteBuffer?,
    ): Int

    external override fun appComplete(
        app: Long,
        packet: ByteBuffer?,
        length: Long,
        result: Long,
        receipt: ByteBuffer?,
    ): Int

    external override fun bufferLength(buffer: Long, receipt: ByteBuffer?): Int

    external override fun bufferCopy(
        buffer: Long,
        offset: Long,
        destination: ByteBuffer?,
        capacity: Long,
        receipt: ByteBuffer?,
    ): Int

    external override fun bufferFree(buffer: Long, receipt: ByteBuffer?): Int

    external override fun inputFree(input: Long, receipt: ByteBuffer?): Int

    external override fun appFree(app: Long, receipt: ByteBuffer?): Int

    external override fun resultRetireStopped(result: Long, receipt: ByteBuffer?): Int
}

private object PortableLibrary {
    init {
        val explicit = System.getProperty("nativesketch.jni.path")
        if (explicit == null) System.loadLibrary("sketch_app_jni") else System.load(explicit)
    }

    fun calls(): NativeCalls = PortableJniV1()
}

internal fun nativeCalls(): NativeCalls {
    val calls = PortableLibrary.calls()
    check(calls.apiVersion() == 1) { "Unsupported portable JNI transport version" }
    return calls
}
