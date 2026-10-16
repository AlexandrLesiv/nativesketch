package dev.nativesketch.portable

/** Receipt buffer ownership is confined to its TransportChannel's thread. */
internal class ReplyReader(
    private val calls: NativeCalls,
    private val buffers: PortableBuffers,
    private val holder: TransferHolder,
) {
    fun accept(status: Int) {
        holder.adopt(buffers.receipt, status)
        check(!holder.quarantined) { "Portable transport quarantined; ownership retained" }
        if (status != 0)
            throw TransportFailure(status, ByteArray(72) { buffers.receipt.get(56 + it) })
    }

    fun bytes(): ByteArray {
        check(holder.reply != 0L)
        buffers.prepareReceipt()
        accept(calls.bufferLength(holder.reply, buffers.receipt))
        require(holder.count in 0..1_048_576)
        val bytes = ByteArray(holder.count.toInt())
        var offset = 0
        while (offset < bytes.size) {
            val length = minOf(buffers.copy.capacity(), bytes.size - offset)
            buffers.prepareReceipt()
            accept(
                calls.bufferCopy(
                    holder.reply,
                    offset.toLong(),
                    buffers.copy,
                    length.toLong(),
                    buffers.receipt,
                )
            )
            check(holder.count in 1..length.toLong())
            for (index in 0 until holder.count.toInt()) bytes[offset + index] =
                buffers.copy.get(index)
            offset += holder.count.toInt()
        }
        return bytes
    }

    fun free() {
        if (holder.reply == 0L) return
        buffers.prepareReceipt()
        accept(calls.bufferFree(holder.reply, buffers.receipt))
    }
}

internal class TransportFailure(val status: Int, val wireError: ByteArray) :
    IllegalStateException("Portable transport status $status")

/** Single producer/consumer slot. Caller establishes happens-before (thread start/join/queue). */
internal class TransferSlot {
    var work = 0L
    var result = 0L
}
