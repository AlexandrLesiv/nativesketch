package dev.nativesketch.portable

/** One channel per actual JVM thread; writable direct buffers never escape this object. */
internal class TransportChannel {
    private val owner = Thread.currentThread()
    private val calls = nativeCalls()
    private val buffers = PortableBuffers()
    private val holder = TransferHolder()
    private val replies = ReplyReader(calls, buffers, holder)
    val app: Long
        get() = holder.app

    val work: Long
        get() = holder.work

    val result: Long
        get() = holder.result

    val reply: Long
        get() = holder.reply

    val input: Long
        get() = holder.input

    private fun prepare(packet: ByteArray): Long {
        check(Thread.currentThread() === owner)
        check(!holder.quarantined && holder.reply == 0L)
        val length = buffers.packet(packet)
        buffers.prepareReceipt()
        return length
    }

    fun create(packet: ByteArray) {
        check(holder.app == 0L)
        val length = prepare(packet)
        replies.accept(calls.appNew(buffers.packet, length, buffers.receipt))
    }

    fun call(packet: ByteArray) {
        val length = prepare(packet)
        replies.accept(calls.appCall(holder.app, buffers.packet, length, buffers.receipt))
    }

    fun takeWork(packet: ByteArray) {
        check(holder.work == 0L)
        val length = prepare(packet)
        replies.accept(calls.appTakeWork(holder.app, buffers.packet, length, buffers.receipt))
    }

    fun workCall(packet: ByteArray) {
        check(holder.result == 0L)
        val length = prepare(packet)
        replies.accept(
            calls.workCall(holder.work, buffers.packet, length, holder.input, buffers.receipt)
        )
    }

    fun complete(packet: ByteArray) {
        val length = prepare(packet)
        replies.accept(
            calls.appComplete(holder.app, buffers.packet, length, holder.result, buffers.receipt)
        )
    }

    fun copyInput(bytes: ByteArray) {
        check(Thread.currentThread() === owner && !holder.quarantined)
        check(holder.input == 0L && holder.reply == 0L)
        require(bytes.size <= 67_108_865)
        val source = PortableBuffers.direct(bytes.size)
        for (index in bytes.indices) source.put(index, bytes[index])
        buffers.prepareReceipt()
        replies.accept(
            calls.workCopyInput(holder.work, source, bytes.size.toLong(), buffers.receipt)
        )
    }

    fun replyBytes(): ByteArray {
        check(Thread.currentThread() === owner && !holder.quarantined)
        return replies.bytes()
    }

    fun releaseReply() {
        check(Thread.currentThread() === owner && !holder.quarantined)
        replies.free()
    }

    fun releaseInput() {
        check(Thread.currentThread() === owner && !holder.quarantined)
        buffers.prepareReceipt()
        replies.accept(calls.inputFree(holder.input, buffers.receipt))
    }

    fun freeApp() {
        check(Thread.currentThread() === owner && !holder.quarantined && holder.reply == 0L)
        buffers.prepareReceipt()
        replies.accept(calls.appFree(holder.app, buffers.receipt))
    }

    fun retireStoppedResult() {
        check(Thread.currentThread() === owner && !holder.quarantined)
        buffers.prepareReceipt()
        replies.accept(calls.resultRetireStopped(holder.result, buffers.receipt))
    }

    fun releaseWork(slot: TransferSlot) {
        check(Thread.currentThread() === owner && !holder.quarantined)
        check(slot.work == 0L && holder.work != 0L && holder.input == 0L && holder.reply == 0L)
        slot.work = holder.work
        holder.work = 0
    }

    fun acquireWork(slot: TransferSlot) {
        check(Thread.currentThread() === owner && !holder.quarantined)
        check(holder.work == 0L && slot.work != 0L)
        holder.work = slot.work
        slot.work = 0
    }

    fun releaseResult(slot: TransferSlot) {
        check(Thread.currentThread() === owner && !holder.quarantined)
        check(slot.result == 0L && holder.result != 0L && holder.reply == 0L)
        slot.result = holder.result
        holder.result = 0
    }

    fun acquireResult(slot: TransferSlot) {
        check(Thread.currentThread() === owner && !holder.quarantined)
        check(holder.result == 0L && slot.result != 0L)
        holder.result = slot.result
        slot.result = 0
    }
}
