package dev.nativesketch.portable

import java.util.concurrent.atomic.AtomicReference
import org.junit.Assert.*
import org.junit.Test

class WorkerJniHostTest {
    @Test
    fun realWorkerTransferRejectsForeignCompletionAndAppliesOnce() {
        HostFixture().use { app ->
            app.draw()
            val operation = app.queue()
            app.take()
            val slot = TransferSlot()
            app.channel.releaseWork(slot)
            val failure = AtomicReference<Throwable?>()
            val thread = Thread {
                try {
                    val raw = RawHost()
                    raw.rejected(
                        raw.calls.appCall(
                            app.channel.app,
                            direct(WirePacket.frame(5)),
                            24,
                            raw.receipt,
                        )
                    )
                    assertEquals(11, raw.receipt.getInt(0))
                    val worker = TransportChannel()
                    worker.acquireWork(slot)
                    worker.workCall(WirePacket.frame(101))
                    worker.releaseReply()
                    val original = worker.work
                    assertThrows(TransportFailure::class.java) {
                        worker.workCall(
                            WirePacket.frame(104, WirePacket.body(8).putInt(0, 4).array())
                        )
                    }
                    assertEquals(original, worker.work)
                    worker.workCall(
                        WirePacket.frame(104, WirePacket.body(8).putInt(0, 1).putInt(4, 3).array())
                    )
                    assertEquals(0L, worker.work)
                    assertNotEquals(0L, worker.result)
                    worker.releaseReply()
                    worker.releaseResult(slot)
                } catch (error: Throwable) {
                    failure.set(error)
                }
            }
            thread.start()
            thread.join(10_000)
            assertFalse(thread.isAlive)
            failure.get()?.let { throw AssertionError("Worker failed", it) }
            app.channel.acquireResult(slot)
            val result = app.channel.result
            HostFixture().use { other ->
                val raw = RawHost()
                raw.rejected(
                    raw.calls.appComplete(
                        other.channel.app,
                        direct(WirePacket.frame(31)),
                        24,
                        result,
                        raw.receipt,
                    )
                )
                assertEquals(11, raw.receipt.getInt(0))
                raw.rejected(raw.calls.resultRetireStopped(result, raw.receipt))
            }
            assertEquals(result, app.channel.result)
            app.complete()
            assertEquals(0L, app.channel.result)
            assertEquals(0, app.call(32, words(app.session, operation)).getInt(156) and 1)
        }
    }

    @Test
    fun consumedEncodeFailureRemainsOwnedBeforeConsumerParsing() {
        HostFixture(24).use { app ->
            app.draw()
            val operation = app.queue()
            app.take()
            assertThrows(IllegalArgumentException::class.java) {
                app.channel.workCall(WirePacket.frame(101))
                throw IllegalArgumentException("Deliberate consumer failure after adoption")
            }
            assertEquals(0L, app.channel.work)
            assertNotEquals(0L, app.channel.result)
            assertNotEquals(0L, app.channel.reply)
            app.channel.releaseReply()
            app.complete()
            assertEquals(1, app.call(32, words(app.session, operation)).getInt(156) and 1)
        }
    }

    @Test
    fun corruptReadConsumesOwnedInputWithoutReplacingDocument() {
        HostFixture().use { app ->
            val operation = app.queue(5)
            app.take()
            app.channel.copyInput("corrupt".toByteArray())
            val input = app.channel.input
            assertNotEquals(0L, input)
            assertThrows(TransportFailure::class.java) {
                app.channel.workCall(WirePacket.frame(101))
            }
            assertEquals(input, app.channel.input)
            app.work(102)
            assertEquals(0L, app.channel.input)
            assertEquals(0L, app.channel.work)
            app.complete()
            app.call(32, words(app.session, operation))
            assertEquals(0L, app.call(3, words(app.session)).getLong(112))
        }
    }
}
