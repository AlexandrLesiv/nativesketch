package dev.nativesketch.portable

import java.nio.ByteBuffer

/** Preallocated state. Adoption performs no allocation, callbacks, or reply parsing. */
internal class TransferHolder {
    var app = 0L
    var reply = 0L
    var work = 0L
    var input = 0L
    var result = 0L
    var count = 0L
    var status = 0
    var flags = 0
    var quarantined = false

    fun adopt(receipt: ByteBuffer, returned: Int) {
        status = receipt.getInt(0)
        flags = receipt.getInt(4)
        val nextApp = receipt.getLong(8)
        val nextReply = receipt.getLong(16)
        val nextWork = receipt.getLong(24)
        val nextInput = receipt.getLong(32)
        val nextResult = receipt.getLong(40)
        count = receipt.getLong(48)
        if (flags and 8 != 0) app = 0 else if (nextApp != 0L) app = nextApp
        if (flags and 16 != 0) reply = 0 else if (nextReply != 0L) reply = nextReply
        if (flags and 1 != 0) work = 0 else if (nextWork != 0L) work = nextWork
        if (flags and 2 != 0) input = 0 else if (nextInput != 0L) input = nextInput
        if (flags and 4 != 0) result = 0 else if (nextResult != 0L) result = nextResult
        if (returned != status || flags and 31.inv() != 0 || status == 6) quarantined = true
    }
}
