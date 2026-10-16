package dev.nativesketch.portable

import java.nio.ByteBuffer
import org.junit.Assert.*
import org.junit.Test

class InputJniHostTest {
    @Test
    fun copiedInputPreflightRejectionsRetainActualWorkAndReleaseExactlyOnce() {
        HostFixture().use { app ->
            val operation = app.queue(5)
            app.take()
            val raw = RawHost()
            val work = app.channel.work
            val sources =
                listOf<Pair<ByteBuffer?, Long>>(
                    null to 1,
                    ByteBuffer.allocate(4) to 4,
                    PortableBuffers.direct(4) to -1,
                    PortableBuffers.direct(4) to 5,
                    raw.receipt to 4,
                    PortableBuffers.direct(1) to 67_108_866,
                )
            for ((source, length) in sources) raw.rejected(
                raw.calls.workCopyInput(work, source, length, raw.receipt)
            )
            val info = app.work(100)
            assertEquals(Long.MIN_VALUE + 3, info.getLong(112))
            val source = direct("bad".toByteArray()).asReadOnlyBuffer()
            source.position(2)
            assertEquals(0, raw.calls.workCopyInput(work, source, 3, raw.receipt))
            val input = raw.receipt.getLong(32)
            assertEquals(work, raw.receipt.getLong(24))
            assertNotEquals(0L, input)
            assertEquals(0, raw.calls.inputFree(input, raw.receipt))
            assertEquals(2, raw.receipt.getInt(4))
            assertEquals(0L, raw.receipt.getLong(32))
            raw.rejected(raw.calls.inputFree(input, raw.receipt))
            app.work(105, WirePacket.body(8).putInt(0, 2).array())
            app.complete()
            app.call(32, words(app.session, operation))
        }
    }
}
