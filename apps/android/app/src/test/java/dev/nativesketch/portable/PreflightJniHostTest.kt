package dev.nativesketch.portable

import java.nio.ByteBuffer
import org.junit.Assert.*
import org.junit.Test

class PreflightJniHostTest {
    @Test
    fun invalidReceiptCannotMutateExistingApp() {
        HostFixture().use { app ->
            val raw = RawHost()
            val packet = raw.packet()
            val receipts =
                listOf(
                    null,
                    ByteBuffer.allocate(128),
                    PortableBuffers.direct(127),
                    PortableBuffers.direct(128).asReadOnlyBuffer(),
                )
            for (receipt in receipts) {
                assertTrue(
                    raw.calls.appCall(
                        app.channel.app,
                        packet,
                        packet.capacity().toLong(),
                        receipt,
                    ) != 0
                )
                raw.sameDocument(app)
            }
            val overlapping = PortableBuffers.direct(256)
            for (index in 0 until packet.capacity()) overlapping.put(index, packet.get(index))
            assertTrue(
                raw.calls.appCall(
                    app.channel.app,
                    overlapping,
                    packet.capacity().toLong(),
                    overlapping,
                ) != 0
            )
            assertEquals(packet.getInt(8) or Int.MIN_VALUE, overlapping.getInt(64))
            assertEquals(packet.getLong(16), overlapping.getLong(72))
            raw.sameDocument(app)
        }
    }

    @Test
    fun malformedPacketAndExplicitLengthsRejectBeforeMutation() {
        HostFixture().use { app ->
            val raw = RawHost()
            val packet = raw.packet()
            val invalid =
                listOf<Pair<ByteBuffer?, Long>>(
                    null to 1,
                    ByteBuffer.allocate(256) to 24,
                    packet to -1,
                    packet to Long.MAX_VALUE,
                    packet to (packet.capacity() + 1L),
                )
            for ((source, length) in invalid) {
                raw.rejected(raw.calls.appCall(app.channel.app, source, length, raw.receipt))
                raw.sameDocument(app)
            }
            val broken = raw.packet()
            broken.put(0, 0)
            raw.rejected(
                raw.calls.appCall(app.channel.app, broken, broken.capacity().toLong(), raw.receipt)
            )
            raw.sameDocument(app)
        }
    }

    @Test
    fun readOnlyInputAndNonzeroPositionsDoNotChangeOffsets() {
        HostFixture().use { app ->
            val raw = RawHost()
            val packet = direct(WirePacket.frame(5)).asReadOnlyBuffer()
            packet.position(7)
            packet.limit(8)
            raw.receipt.position(120)
            assertEquals(0, raw.calls.appCall(app.channel.app, packet, 24, raw.receipt))
            val reply = raw.receipt.getLong(16)
            assertEquals(app.channel.app, raw.receipt.getLong(8))
            for (offset in 24..48 step 8) assertEquals(0L, raw.receipt.getLong(offset))
            for (offset in 56 until 128) assertEquals(0, raw.receipt.get(offset).toInt())
            val destination = PortableBuffers.direct(256)
            destination.position(200)
            destination.limit(201)
            assertEquals(0, raw.calls.bufferCopy(reply, 0, destination, 168, raw.receipt))
            destination.limit(256)
            assertEquals(78, destination.get(0).toInt())
            assertEquals(1L, destination.getLong(72))
            raw.free(reply)
        }
    }

    @Test
    fun malformedCopyAndWrongKindsRetainBufferOwnership() {
        HostFixture().use { app ->
            val raw = RawHost()
            val reply = raw.query(app.channel.app)
            for (destination in
                listOf(
                    null,
                    ByteBuffer.allocate(256),
                    PortableBuffers.direct(256).asReadOnlyBuffer(),
                    raw.receipt,
                )) {
                raw.rejected(raw.calls.bufferCopy(reply, 0, destination, 64, raw.receipt))
            }
            for (offset in listOf(-1L, Long.MAX_VALUE)) raw.rejected(
                raw.calls.bufferCopy(reply, offset, PortableBuffers.direct(256), 64, raw.receipt)
            )
            raw.rejected(
                raw.calls.bufferCopy(reply, 0, PortableBuffers.direct(256), 65_617, raw.receipt)
            )
            raw.rejected(raw.calls.bufferLength(app.channel.app, raw.receipt))
            raw.rejected(raw.calls.bufferLength(Long.MIN_VALUE + 19, raw.receipt))
            assertEquals(0, raw.calls.bufferLength(reply, raw.receipt))
            raw.free(reply)
            raw.rejected(raw.calls.bufferFree(reply, raw.receipt))
        }
    }
}
