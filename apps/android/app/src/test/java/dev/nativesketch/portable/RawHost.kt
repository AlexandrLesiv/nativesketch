package dev.nativesketch.portable

import java.nio.ByteBuffer
import org.junit.Assert.*

internal class RawHost {
    val calls = nativeCalls()
    val receipt = PortableBuffers.direct(128)

    fun free(buffer: Long) {
        assertEquals(0, calls.bufferFree(buffer, receipt))
    }

    fun query(app: Long, packet: ByteArray = WirePacket.frame(5)): Long {
        assertEquals(0, calls.appCall(app, direct(packet), packet.size.toLong(), receipt))
        return receipt.getLong(16)
    }

    fun rejected(status: Int) {
        assertTrue(status != 0)
        assertEquals(status, receipt.getInt(0))
        assertEquals(0, receipt.getInt(4))
        for (offset in 8..48 step 8) assertEquals("offset $offset", 0L, receipt.getLong(offset))
    }

    fun sameDocument(app: HostFixture) {
        assertEquals(1L, app.call(5).getLong(72))
    }

    fun packet(): ByteBuffer = direct(fixture("create.cap3"))
}
