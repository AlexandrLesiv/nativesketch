/// GPU-independent scheduling policy. A persistent failure waits for external invalidation.
struct RenderScheduleState {
  private(set) var pending = false
  private(set) var outstanding = 0
  private(set) var recoveryNeeded = false
  private(set) var recoveryAttempted = false
  private(set) var blocked = false
  private(set) var closed = false
  var canSchedule: Bool {
    !closed && !blocked && pending && outstanding < 2
      && (!recoveryNeeded || outstanding == 0)
  }
  mutating func request() {
    guard !closed else { return }
    pending = true
    blocked = false
  }
  mutating func begin() { pending = false }
  mutating func deferDrawable() { pending = true }
  mutating func submitted() { outstanding += 1 }
  mutating func completed() { outstanding = max(0, outstanding - 1) }
  mutating func succeeded() { recoveryAttempted = false }
  mutating func failed() {
    pending = true
    if recoveryAttempted {
      blocked = true
    } else {
      recoveryNeeded = true
    }
  }
  mutating func beginRecovery() {
    recoveryNeeded = false
    recoveryAttempted = true
  }
  mutating func close() {
    closed = true
    pending = false
  }
}
