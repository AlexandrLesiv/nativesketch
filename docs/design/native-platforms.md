# Native platform interaction agreement

Designed before implementation by ui_ux_designer on 2026-10-14. Scope is first-stroke
parity across hosts, including mobile adaptation, not the full product backlog.

Rust owns document samples, optional pressure, geometry, hit testing, selection, uniform
transforms, undo/redo, validation, and file bytes. Hosts own native events/controls,
viewport conversion, authorized file locations, and disposable path presentation.
Coordinates remain top-left/y-down document-space doubles; absent pressure stays absent.
Native file-provider URIs and browser files require byte import/export. Export alone does
not mark clean; a revision token acknowledges only a successfully completed save of that
revision. Download-only browsers cannot prove persistence and must not silently mark clean.

| Host | Native interface |
| --- | --- |
| macOS | AppKit toolbar, menus, color/number controls, Open/Save panels |
| iPhone/iPad | UIKit navigation/toolbar, segmented tools, style/selection sheet, color and document pickers |
| Android | Material Views toolbar/buttons/toggle group, text fields/dialogs, Storage Access Framework |
| Windows | WinUI CommandBar/MenuBar, color picker, NumberBox, native file pickers |
| Linux | GTK4/libadwaita header bar, toggle/number/color controls, native file dialogs |
| Browser | Semantic HTML buttons, color/number inputs, dialog, file input/download, canvas |

Mobile uses a compact document bar with Undo/Redo and reachable tool/style/selection
controls. Second touch cancels pending ink/transform before two-finger pan/pinch. Single
finger/stylus draws in Pen; Select taps hit-test and handles uniformly scale. At least
44pt Apple / 48dp Android touch targets; enlarge invisible hit regions for canvas handles.
Desktop supports Space-drag/scroll navigation. Every host has visible zoom/reset commands,
next/previous stroke selection and numeric scale. P/V/Delete must not steal text editing.
Escape/capture loss/deactivation cancel; rotation/resize cancels before remapping input.
Preserve scale and document point at viewport center through layout changes.

Empty state has usable Pen and disabled unavailable operations. Selection uses outline
and handles, not just color. Native control labels, enabled states, focus, and canvas
stroke/selection/zoom description are exposed to accessibility. File operations cancel
transient input; invalid Open preserves document; save failure preserves dirty state and
prior good data. Unsaved replacement offers Save/Discard/Cancel where a confirmed write
is available. Compact layouts use native overflow/sheets rather than clipping controls.

Use current SDK standard AppKit/UIKit chrome for Liquid Glass with native older-OS
fallback; no fabricated glass over live ink. Android uses Material Components for Views.
Sources: https://developer.apple.com/documentation/technologyoverviews/adopting-liquid-glass
and https://developer.android.com/develop/ui/views/theming/look-and-feel .

Verify shared fixture roundtrips, transaction/history parity, second-touch cancellation,
field shortcuts, corrupt-open/save errors, narrow/large and light/dark layouts. Physical
pen feel, VoiceOver/TalkBack speech, unavailable OS hosts and production GPU behavior need
separate evidence. Standard controls or screenshots alone do not certify these.
