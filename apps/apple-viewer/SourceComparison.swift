import CSketchFFI
import Foundation

private func withSourceVersion<T>(
  _ descriptor: SourceDescriptor, _ body: (UnsafePointer<SketchSourceVersion>) throws -> T
) rethrows -> T {
  let values = [
    descriptor.namespaceID, descriptor.documentID, descriptor.revisionID,
    descriptor.parentRevisionID, descriptor.contentSHA256,
  ].map { Array($0.utf8) }
  return try values[0].withUnsafeBufferPointer { a in
    try values[1].withUnsafeBufferPointer { b in
      try values[2].withUnsafeBufferPointer { c in
        try values[3].withUnsafeBufferPointer { d in
          try values[4].withUnsafeBufferPointer { e in
            var version = SketchSourceVersion(
              namespace_id: SketchSourceText(bytes: a.baseAddress, length: a.count),
              document_id: SketchSourceText(bytes: b.baseAddress, length: b.count),
              revision_id: SketchSourceText(bytes: c.baseAddress, length: c.count),
              parent_revision_id: SketchSourceText(bytes: d.baseAddress, length: d.count),
              content_sha256: SketchSourceText(bytes: e.baseAddress, length: e.count),
              server_generation: descriptor.serverGeneration, kind: descriptor.kind)
            return try body(&version)
          }
        }
      }
    }
  }
}
func compareSource(_ current: SourceDescriptor, _ candidate: SourceDescriptor) throws -> UInt32 {
  try current.validate()
  try candidate.validate()
  var decision: UInt32 = 0
  let status = withSourceVersion(current) { a in
    withSourceVersion(candidate) { b in sketch_source_compare(a, b, &decision) }
  }
  guard status == 0 else { throw SourceFailure.metadata }
  return decision
}
