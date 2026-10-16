package dev.nativesketch.portable

import java.nio.ByteBuffer
import java.nio.ByteOrder

/** Literal NSAP v2 frame construction; no native editor policies are implemented here. */
internal object WirePacket {
    fun frame(
        opcode: Int,
        body: ByteArray = byteArrayOf(),
        request: Long = -81985529216486896L,
    ): ByteArray {
        require(body.size <= 1_048_552)
        val packet = ByteBuffer.allocate(24 + body.size).order(ByteOrder.LITTLE_ENDIAN)
        packet.put(0, 78)
        packet.put(1, 83)
        packet.put(2, 65)
        packet.put(3, 80)
        packet.putShort(4, 2)
        packet.putInt(8, opcode)
        packet.putInt(12, body.size)
        packet.putLong(16, request)
        for (index in body.indices) packet.put(24 + index, body[index])
        return packet.array()
    }

    fun body(size: Int): ByteBuffer = ByteBuffer.allocate(size).order(ByteOrder.LITTLE_ENDIAN)
}
