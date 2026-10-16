package dev.nativesketch.portable

import java.util.concurrent.atomic.AtomicReference
import org.junit.Assert.*
import org.junit.Test

class LifetimeJniHostTest {
    @Test
    fun replyCapacityRecoversWithoutLosingOperation() {
        HostFixture().use { app ->
            val raw = RawHost()
            val held = mutableListOf<Long>()
            try {
                repeat(64) { held.add(raw.query(app.channel.app)) }
                raw.rejected(
                    raw.calls.appCall(app.channel.app, direct(WirePacket.frame(5)), 24, raw.receipt)
                )
                assertEquals(3, raw.receipt.getInt(0))
            } finally {
                held.forEach { raw.free(it) }
            }
            raw.sameDocument(app)
        }
    }

    @Test
    fun preparedSceneLeaseReusesIdentityAndBlocksPrematureFree() {
        HostFixture().use { app ->
            app.draw()
            val viewport =
                WirePacket.body(56)
                    .putLong(0, app.session)
                    .putLong(8, 0)
                    .putDouble(16, 0.0)
                    .putDouble(24, 0.0)
                    .putDouble(32, 500.0)
                    .putDouble(40, 500.0)
                    .putDouble(48, 10.0)
            val scene = app.call(40, viewport.array()).getLong(72)
            assertNotEquals(0L, scene)
            viewport.putLong(8, scene)
            assertEquals(scene, app.call(40, viewport.array()).getLong(72))
            val raw = RawHost()
            raw.rejected(raw.calls.appFree(app.channel.app, raw.receipt))
            val descriptors =
                app.call(41, WirePacket.body(16).putLong(0, scene).putInt(12, 1).array())
            assertEquals(1, descriptors.getInt(72))
            app.call(44, words(scene))
            assertThrows(TransportFailure::class.java) { app.call(44, words(scene)) }
        }
    }

    @Test
    fun actualOwnerThreadExitAllowsOnlyStoppedResultRetirement() {
        val slot = TransferSlot()
        val failure = AtomicReference<Throwable?>()
        val owner = Thread {
            try {
                val app = HostFixture()
                app.queue()
                app.take()
                app.channel.releaseWork(slot)
                // Deliberate actual JVM-thread exit, not Activity detach or manufactured cleanup.
            } catch (error: Throwable) {
                failure.set(error)
            }
        }
        owner.start()
        owner.join(10_000)
        assertFalse(owner.isAlive)
        failure.get()?.let { throw AssertionError("Owner failed", it) }
        val worker = TransportChannel()
        worker.acquireWork(slot)
        worker.workCall(WirePacket.frame(105, WirePacket.body(8).putInt(0, 6).array()))
        worker.releaseReply()
        assertNotEquals(0L, worker.result)
        worker.retireStoppedResult()
        assertEquals(0L, worker.result)
    }
}
