/// One timer owns a replaceable latest request; an older timer cannot discard newer work.
struct RenderPendingRetry<Key: Equatable> {
  private(set) var key: Key?
  mutating func enqueue(_ value: Key) -> Bool {
    let needsTimer = key == nil
    key = value
    return needsTimer
  }
  mutating func take(matching current: Key) -> Key? {
    defer { key = nil }
    return key == current ? key : nil
  }
}
