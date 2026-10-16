package dev.nativesketch.portable

import java.nio.ByteBuffer
import org.junit.Assert.*
import org.junit.Test

private class PendingExceptionProbe {
    external fun callWithPendingException(
        app: Long,
        packet: ByteBuffer?,
        length: Long,
        expected: Throwable,
        receipt: ByteBuffer?,
    ): Int
}

class PendingExceptionJniHostTest {
    @Test
    fun existingJvmExceptionIdentitySurvivesWithoutDispatchOrReceiptPublication() {
        System.load(requireNotNull(System.getProperty("nativesketch.jni.probe.path")))
        HostFixture().use { app ->
            val probe = PendingExceptionProbe()
            val receipt = PortableBuffers.direct(128)
            for (index in 0 until 128) receipt.put(index, 91)
            val packet = direct(fixture("create.cap3"))
            val expected = IllegalArgumentException("Original pending JVM exception")
            val caught =
                assertThrows(IllegalArgumentException::class.java) {
                    probe.callWithPendingException(
                        app.channel.app,
                        packet,
                        packet.capacity().toLong(),
                        expected,
                        receipt,
                    )
                }
            assertSame(expected, caught)
            for (index in 0 until 128) assertEquals(91, receipt.get(index).toInt())
            assertEquals(1L, app.call(5).getLong(72))
        }
    }
}
