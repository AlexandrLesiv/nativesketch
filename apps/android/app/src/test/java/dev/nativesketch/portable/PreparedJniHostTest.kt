package dev.nativesketch.portable

import org.junit.Assert.*
import org.junit.Test

class PreparedJniHostTest {
    @Test
    fun pagedPreparedLeaseBlocksShutdownAfterSessionDetachUntilExplicitRelease() {
        HostFixture().use { app ->
            val pointer = Long.MIN_VALUE + 11
            app.call(
                10,
                app.context() +
                    WirePacket.body(24).putInt(0, 6).putLong(8, pointer).putInt(16, 2).array(),
            )
            val samples = WirePacket.body(16 + 300 * 48).putLong(0, pointer).putInt(8, 300)
            for (index in 0 until 300) {
                val start = 16 + index * 48
                samples.putLong(start + 8, Long.MIN_VALUE + index)
                samples.putDouble(start + 16, if (index % 2 == 1) 999999010.0 else 0.0)
                samples.putDouble(start + 24, 20.0).putDouble(start + 32, index.toDouble())
            }
            app.call(12, app.context() + samples.array())
            app.call(
                10,
                app.context() + WirePacket.body(16).putInt(0, 7).putLong(8, pointer).array(),
            )
            val viewport =
                WirePacket.body(56)
                    .putLong(0, app.session)
                    .putDouble(16, 999999000.0)
                    .putDouble(32, 100.0)
                    .putDouble(40, 100.0)
                    .putDouble(48, 2.0)
            val scene = app.call(40, viewport.array()).getLong(72)
            val prepared =
                app.call(
                    45,
                    WirePacket.body(24).putLong(0, scene).putInt(16, 133).putInt(20, 137).array(),
                )
            assertEquals(1, prepared.getInt(72))
            val lease = prepared.getLong(80)
            val count = prepared.getLong(120)
            assertTrue(count > 744)
            app.call(44, words(scene))
            var copied = 0
            while (copied.toLong() < count) {
                val page =
                    app.call(
                        46,
                        WirePacket.body(16)
                            .putLong(0, lease)
                            .putInt(8, copied)
                            .putInt(12, 744)
                            .array(),
                    )
                assertTrue(page.getInt(72) in 1..744)
                copied += page.getInt(72)
            }
            assertEquals(count, copied.toLong())
            app.detachAndDrain()
            assertEquals(0L, app.call(5).getLong(72))
            assertThrows(TransportFailure::class.java) { app.channel.freeApp() }
            assertNotEquals(0L, app.channel.app)
            app.call(47, words(lease))
            assertThrows(TransportFailure::class.java) { app.call(47, words(lease)) }
        }
    }
}
