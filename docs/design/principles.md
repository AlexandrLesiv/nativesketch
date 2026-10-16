# Native drawing UX

The UI/UX designer owns interaction specifications and audits implemented behavior.
This role collaborates with platform and rendering engineers; it does not certify latency
or physical pen feel from screenshots.

## Platform design coordination

ui_ux_designer coordinates shared tool meanings, terminology, journeys and accessibility
goals, with a separate design/review brief for each affected platform. Native layouts,
navigation, controls and input conventions may differ. Coordinate feasibility with macos_ui,
ios_ui, android_ui, windows_ui and lead-assigned Linux/web implementers. Request focused
platform design reviews through the lead when needed; no additional permanent designer
roles or recursive delegation are implied. Keep the existing concurrency limits.

Each brief identifies platform conventions, supported input/window sizes, state/error flows,
native control mapping, acceptance IDs, reviewer and evidence gaps. Consolidate variants
and findings into one versioned Figma handoff for owner approval. Engineers provide feasibility
feedback, not an alternative source of design approval. Review the running platform afterward.

ChromeOS review follows the actual delivery route: Android desktop or browser, with shared
Chromebook acceptance cases. Do not invent a third interface. Linux design targets GNOME
with GTK4/libadwaita, not every Linux desktop's appearance. iPhone/iPad, macOS, visionOS 2D
and TV/Watch viewer interactions remain distinct. Missing expertise/device evidence is
recorded explicitly; one host's review cannot certify the others.

## Principles

- Make the canvas the primary workspace; keep common actions discoverable.
- Use real platform menus, toolbars, controls, text entry, dialogs, and accessibility APIs.
- Prefer familiar macOS behavior over reproducing Concepts' visual layout exactly.
- Keep drawing state, selection state, and navigation state clear and reversible.
- Respect system appearance, text sizing, contrast, reduced motion, and focus behavior.
- A UI mockup is design evidence, not implementation or native-widget evidence.
- Avoid placing expensive blur/backdrop effects over the live canvas without measurements.

## Designer deliverable for a feature

1. User goal and concise interaction flow.
2. Proposed native control mapping and canvas-only custom elements.
3. State table: initial, active, selected, disabled, empty, cancelled, and error as applicable.
4. Keyboard, mouse, trackpad, and pen behavior; accessibility labels and focus order.
5. Layout/resizing behavior and a simple wireframe when it resolves ambiguity.
6. Observable acceptance cases for implementation and review.
7. Open product choices, clearly separated from established requirements.

Deliver a recommended design. New pages/screens and tools require the Figma/owner approval
gate below, even if the designer has no unresolved questions. Routine implementation and
bug fixes within an approved design do not require another full design approval cycle.

## Figma handoff and owner approval

Owner instruction recorded 2026-10-16. Before implementing a new application page/screen
or tool, ui_ux_designer prepares a Figma design and explicitly presents it to the owner
for approval. Approval of a feature idea is not approval of its detailed design. Do not
silently bypass this gate when the owner is absent or a Figma connection is unavailable.

1. Lead defines authorized scope, affected platforms and non-goals. Designer and the
   assigned platform engineer check native feasibility before polishing the proposal.
2. Organize one project design file by feature/task and platform, with reusable native
   patterns/components. Avoid creating a cross-platform widget system or copying one
   platform's visual style to all hosts. Prefer official platform design resources.
3. Include flow, representative window/device sizes, light/dark where applicable, controls,
   selection/disabled/empty/loading/error states, focus/shortcuts, input and accessibility
   notes. Use a clickable prototype when transitions are material; do not fake pen physics.
4. Present direct file/frame links and a named version/checkpoint, concise native-control
   mapping, acceptance IDs, tradeoffs and the explicit question of approval before coding.
   Store the handoff record under docs/design/ with task ID, links/frame IDs, version/date,
   exported review images when available, approval evidence and known deviations.
5. Only the owner's explicit approval enables implementation of that design. Record who
   approved which version/scope and link the record from the task. Figma "Ready for dev"
   is an optional handoff indicator, not proof of owner approval. No paid feature required.
6. Platform engineers implement native controls and shared Rust commands. Inspect the
   actual app against the approved design with the designer. Material flow/layout/control
   deviations return for approval; routine native sizing/font adaptations within the agreed
   rules do not. Approval of a mockup never substitutes for native E2E/accessibility checks.

Keep visual intent in Figma; keep behavioral contracts, acceptance tests and approval records
in the repository, linked rather than duplicated. A mutable "latest" link is insufficient
to identify what was approved. Do not treat generated design code as production native UI.
If Figma editing/export is unavailable, report that concrete limitation and continue useful
authorized non-UI work. Request access or an explicit alternative; never claim an HTML/SVG
mockup is an approved Figma design. This policy does not install a Figma connector or grant
account permissions. No new app pages/tools are introduced by documenting this process.

References: [Figma handoff](https://help.figma.com/hc/en-us/articles/360040521453-Optimize-design-files-for-developer-handoff),
[version history](https://help.figma.com/hc/en-us/articles/360038006754-View-a-file-s-version-history),
[Apple design resources](https://developer.apple.com/design/resources/).

## First milestone proposal (requires task-level design)

One document window with a large canvas, native pen controls, and standard Undo/Redo menu
commands. Mouse drawing must work; tablet input adds pressure when available. Navigation
must not accidentally draw. State should survive window resizing without geometric jumps.
Do not add inert buttons for tools that are not implemented.

## Design review evidence

Inspect the running native app at small and large window sizes, light/dark appearances,
keyboard-only navigation, and relevant accessibility settings. Record observed behavior
and issues with reproduction steps. Ask the owner to assess real pen feel at milestone
acceptance. Design approval cannot substitute for implementation evidence.

