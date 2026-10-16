import Testing

@testable import NativeSketch

struct RenderPendingRetryTests {
  @Test func newerDropReusesOldTimerWithoutLosingLatestRequest() {
    var retry = RenderPendingRetry<Int>()
    let firstTimer = retry.enqueue(1)
    let secondTimer = retry.enqueue(2)
    #expect(firstTimer)
    #expect(!secondTimer)
    let fired = retry.take(matching: 2)
    #expect(fired == 2)
    #expect(retry.key == nil)
    let duplicate = retry.take(matching: 2)
    #expect(duplicate == nil)
  }

  @Test func requestWithoutDropInvalidatesOldRetry() {
    var retry = RenderPendingRetry<Int>()
    let scheduled = retry.enqueue(1)
    #expect(scheduled)
    let fired = retry.take(matching: 2)
    #expect(fired == nil)
    #expect(retry.key == nil)
    let laterTimer = retry.enqueue(3)
    #expect(laterTimer)
  }
}
