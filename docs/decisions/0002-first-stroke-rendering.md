# 0002: Provisional first-stroke presentation

Status: prototype boundary only; production renderer decision OPEN.
Date: 2026-10-14

Task 001 needs a visible editable stroke while task 002 requires comparable GPU
experiments before committing to a production renderer. For this milestone Swift
paints Rust-generated document-space paths using CoreGraphics. The Rust document,
pressure curve, selection, transforms, persistence, and C ABI do not contain
CoreGraphics objects. This adapter is disposable, not a multi-backend framework.

This is not completion of task 002 and does not select CoreGraphics over Rust-Skia
or Vello. No comparable Skia/Metal versus pinned Vello workload has run. A production
renderer decision must still supply task 002's identical scenes, GPU settings,
versions, raw timings, visual comparisons, memory/tail/stall data, integration risks,
and human acceptance. CPU first-stroke timings cannot resolve that comparison.

The first-stroke branch can be reviewed as an interaction and document milestone.
It is not evidence for production renderer readiness, sustained GPU performance,
end-to-end latency, or physical pen feel.
