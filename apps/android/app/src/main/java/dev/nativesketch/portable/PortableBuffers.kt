package dev.nativesketch.portable

import java.nio.ByteBuffer
import java.nio.ByteOrder

internal class PortableBuffers {
    val packet: ByteBuffer = direct(1_048_576)
    val receipt: ByteBuffer = direct(128)
    val copy: ByteBuffer = direct(65_616)

    fun packet(bytes: ByteArray): Long {
        require(bytes.size <= packet.capacity())
        for (index in bytes.indices) packet.put(index, bytes[index])
        return bytes.size.toLong()
    }

    fun prepareReceipt() {
        for (offset in 0 until 128 step 8) receipt.putLong(offset, 0)
        receipt.putInt(0, -1)
    }

    companion object {
        fun direct(size: Int): ByteBuffer =
            ByteBuffer.allocateDirect(size).order(ByteOrder.LITTLE_ENDIAN)
    }
}
