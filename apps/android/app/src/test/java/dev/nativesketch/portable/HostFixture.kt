package dev.nativesketch.portable

import java.io.File
import java.nio.ByteBuffer
import java.nio.ByteOrder

internal fun fixture(name: String): ByteArray {
    val line =
        File(requireNotNull(System.getProperty("nativesketch.wire.fixtures"))).readLines().first {
            it.trim().startsWith("\"$name\":")
        }
    val hex = line.substringAfter(": \"").substringBefore('"')
    return ByteArray(hex.length / 2) { hex.substring(it * 2, it * 2 + 2).toInt(16).toByte() }
}

internal fun decoded(bytes: ByteArray): ByteBuffer =
    ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)

internal fun words(vararg values: Long): ByteArray {
    val bytes = WirePacket.body(values.size * 8)
    for ((index, value) in values.withIndex()) bytes.putLong(index * 8, value)
    return bytes.array()
}

internal fun direct(bytes: ByteArray): ByteBuffer {
    val buffer = PortableBuffers.direct(bytes.size)
    for (index in bytes.indices) buffer.put(index, bytes[index])
    return buffer
}

internal class HostFixture(documentLimit: Long? = null) : AutoCloseable {
    val channel = TransportChannel()
    private var detached = false
    val session: Long

    init {
        val bootstrap = fixture("bootstrap")
        if (documentLimit != null) decoded(bootstrap).putLong(64, documentLimit)
        channel.create(bootstrap)
        channel.call(fixture("create.cap3"))
        session = decoded(channel.replyBytes()).getLong(72)
        channel.releaseReply()
    }

    fun call(opcode: Int, body: ByteArray = byteArrayOf()): ByteBuffer {
        channel.call(WirePacket.frame(opcode, body))
        try {
            return decoded(channel.replyBytes())
        } finally {
            channel.releaseReply()
        }
    }

    fun context(): ByteArray {
        val projection = call(3, words(session))
        return words(session, projection.getLong(80), projection.getLong(104))
    }

    fun file(tag: Int): ByteBuffer = call(20, context() + WirePacket.body(8).putInt(0, tag).array())

    fun decision(effect: Long, tag: Int): ByteBuffer =
        call(21, context() + words(effect) + WirePacket.body(8).putInt(0, tag).array())

    fun queue(tag: Int = 2): Long {
        val effect = file(tag).getLong(240)
        val resource =
            WirePacket.body(32)
                .putInt(0, 1)
                .putInt(4, if (tag == 5) 1 else 7)
                .putLong(8, Long.MIN_VALUE + 3)
                .putLong(16, 1)
                .putLong(24, 1)
        return call(22, context() + words(effect) + resource.array()).getLong(248)
    }

    fun draw() {
        call(10, context() + WirePacket.body(24).putInt(0, 6).putLong(8, 7).putInt(16, 2).array())
        call(12, context() + fixture("sample.0.1").copyOfRange(48, fixture("sample.0.1").size))
        call(10, context() + WirePacket.body(16).putInt(0, 7).putLong(8, 7).array())
    }

    fun take() {
        channel.takeWork(WirePacket.frame(30))
        channel.releaseReply()
        check(channel.work != 0L)
    }

    fun work(opcode: Int, body: ByteArray = byteArrayOf()): ByteBuffer {
        channel.workCall(WirePacket.frame(opcode, body))
        try {
            return decoded(channel.replyBytes())
        } finally {
            channel.releaseReply()
        }
    }

    fun complete() {
        channel.complete(WirePacket.frame(31))
        channel.releaseReply()
    }

    fun detachAndDrain() {
        check(!detached)
        var outcome = file(6)
        if (outcome.getInt(232) == 3) outcome = decision(outcome.getLong(240), 3)
        check(outcome.getInt(232) == 6)
        check(decision(outcome.getLong(240), 5).getInt(232) == 7)
        call(2, words(session))
        while (true) {
            channel.takeWork(WirePacket.frame(30))
            channel.releaseReply()
            if (channel.work == 0L) break
            work(106)
            complete()
        }
        while (call(34).getInt(72) != 0) {
            /* Each issued release is consumed once. */
        }
        detached = true
    }

    override fun close() {
        if (!detached) detachAndDrain()
        channel.freeApp()
        check(channel.app == 0L)
    }
}
