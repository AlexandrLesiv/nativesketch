package dev.nativesketch.portable

import java.nio.ByteBuffer

/** Internal boundary; only malformed-buffer tests bypass the exclusive-buffer wrapper. */
internal interface NativeCalls {
    fun apiVersion(): Int

    fun appNew(packet: ByteBuffer?, length: Long, receipt: ByteBuffer?): Int

    fun appCall(app: Long, packet: ByteBuffer?, length: Long, receipt: ByteBuffer?): Int

    fun appTakeWork(app: Long, packet: ByteBuffer?, length: Long, receipt: ByteBuffer?): Int

    fun workCopyInput(work: Long, source: ByteBuffer?, length: Long, receipt: ByteBuffer?): Int

    fun workCall(
        work: Long,
        packet: ByteBuffer?,
        length: Long,
        input: Long,
        receipt: ByteBuffer?,
    ): Int

    fun appComplete(
        app: Long,
        packet: ByteBuffer?,
        length: Long,
        result: Long,
        receipt: ByteBuffer?,
    ): Int

    fun bufferLength(buffer: Long, receipt: ByteBuffer?): Int

    fun bufferCopy(
        buffer: Long,
        offset: Long,
        destination: ByteBuffer?,
        capacity: Long,
        receipt: ByteBuffer?,
    ): Int

    fun bufferFree(buffer: Long, receipt: ByteBuffer?): Int

    fun inputFree(input: Long, receipt: ByteBuffer?): Int

    fun appFree(app: Long, receipt: ByteBuffer?): Int

    fun resultRetireStopped(result: Long, receipt: ByteBuffer?): Int
}
