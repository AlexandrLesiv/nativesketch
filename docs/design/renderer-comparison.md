# Native renderer comparison host

Experiment-only interaction contract, defined by ui_ux_designer on 2026-10-14.

Use actual AppKit selectors for Backend, Fixture, Resolution and Mode; native Run/Stop
buttons; read-only phase/result status and Reveal in Finder. Label the window Native
Sketch — Renderer Comparison. The canvas is a deterministic, noninteractive preview;
no gestures may modify benchmark inputs. Display actual physical render dimensions.

Idle permits configuration. Initializing/warming/measuring lock configuration, disable
Run and enable Stop. Stop requests safe cancellation; incomplete artifacts remain clearly
labeled. Completion restores controls without a modal dialog. Failures display the cause
and result/log location, then allow retry after safe teardown. Update status between phases
or at a bounded recorded frequency, not on every measured frame.

Provide keyboard traversal, explicit labels, Space activation and Cmd-period Stop. Do not
make Return accidentally start while a selector/dialog has focus. Announce phase/error
changes, not every sample. System appearance controls native chrome; fixture colors and
background remain fixed. Minimum useful layout is approximately 640x480 points; controls
must remain accessible at small sizes. Idle previews aspect-fit without changing the
measured texture dimensions.

Resize, minimize, screen/backing-scale changes or lost drawable during a timed trial
invalidate/cancel that trial with a recorded reason. Restore permits a new trial, never
silent resumption into complete evidence. Closing during a trial safely cancels and retains
partial evidence. Unsupported dimensions fail visibly rather than silently substituting.

Acceptance: native/keyboard/AX controls; no reentrant runs; responsive Stop; incomplete-run
retention; actionable init/output errors and retry; lifecycle safety; small/large and
light/dark layouts; identifiable backend/device/mode/dimensions/output. Review matching
fixture frames for stroke unions, joins/caps, clipping, transforms and image channels,
orientation/filtering. Runtime inspection and screenshots cannot certify pen latency.
