# 003: Record and replay normalized drawing input

Status: waiting on task 001's stable input contract.
Roles: rust_engine, build_reliability, reviewer.

Create a versioned synthetic-input fixture format and headless replay runner. Include
timestamps, phase, device capability, pressure, cancellation, and transform scenarios.
Do not record user files or device identifiers unless deliberately needed and approved.

Verify equivalent sample batches produce equivalent semantic strokes. Add fixture failures
for cancelled strokes, missing pressure, long sessions, and malformed input. Timing replay
is for engine behavior; it does not prove hardware pen-to-screen latency.

