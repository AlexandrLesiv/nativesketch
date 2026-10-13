# 002: Choose the initial renderer using evidence

Status: ready. Roles: rendering_performance, build_reliability, reviewer.

Compare Rust-Skia over Metal and a pinned Vello version/variant inside a native macOS host.
Use the same stroke outlines, quality, resolution, scene fixtures, and presentation settings.
Do not interpret Vello CPU results as a comparison with Skia's GPU backend.

Cover a live stroke, dense static document, pan/zoom, translucent overlaps, clipping, and
imported images. Capture CPU/GPU times, frame-time tails, missed deadlines, peak memory,
first-use stalls, and sustained behavior. Measure end-to-end latency separately if equipment
is available. Serial benchmark runs; no other active agent builds/UI automation on that host.

Deliver reproducible experiment code, exact versions and hardware, raw measurements,
visual comparisons, integration risks, and a decision recommendation. Update a decision
record after acceptance. Do not present a preference as a measured winner.

