package dev.nativesketch.portable

import org.junit.Assert.*
import org.junit.Test

class ReceiptTest {
    @Test
    fun absoluteOffsetsPreserveHighBitsAndConsumedRejection() {
        val receipt = PortableBuffers.direct(128)
        val holder = TransferHolder()
        holder.work = Long.MIN_VALUE + 7
        holder.input = -3
        receipt.putInt(0, 8)
        holder.adopt(receipt, 8)
        assertEquals(Long.MIN_VALUE + 7, holder.work)
        assertEquals(-3L, holder.input)
        receipt.putInt(0, 0).putInt(4, 3).putLong(40, Long.MIN_VALUE + 9)
        receipt.position(117)
        holder.adopt(receipt, 0)
        assertEquals(0L, holder.work)
        assertEquals(0L, holder.input)
        assertEquals(Long.MIN_VALUE + 9, holder.result)
        assertFalse(holder.quarantined)
    }

    @Test
    fun terminalMismatchPreservesKnownOwnership() {
        val receipt = PortableBuffers.direct(128).putInt(0, 6)
        val holder = TransferHolder()
        holder.app = 19
        holder.adopt(receipt, 6)
        assertTrue(holder.quarantined)
        assertEquals(19L, holder.app)
    }
}
