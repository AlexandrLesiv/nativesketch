# 008: ChromeOS and Googlebook desktop adapter

Status: waiting. Owner authorized implementation on 2026-10-14; follows task 007.

## Outcome

One Kotlin/Material Views editor for Android, compatible ChromeOS and Googlebook.
Shared Rust editing and provisional renderer remain unchanged. Build native ARM64 and
x86-64 libraries; adapt by window size, not device branding. No release publishing.

## Acceptance

Compact and expanded native controls, accessible focus, keyboard editing commands with
text-field isolation, mouse/stylus distinction, wheel/trackpad and Space-drag navigation,
secondary button never inks, live resizing cancels temporary gestures but preserves
committed document/history/selection. Native file-open intents and file drops reuse
bounded import and dirty guards. Existing phone behavior continues working. Build both
ABIs and run replay on available Android runtime; inspect small/large windows and keyboard.
Real ChromeOS/Googlebook hardware, physical stylus and production renderer measurements
must be explicitly distinguished from host/emulator evidence. Do not claim process-death
recovery unless implemented and tested. Fresh review and designer assessment required.
