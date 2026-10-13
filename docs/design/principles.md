# Native drawing UX

The UI/UX designer owns interaction specifications and audits implemented behavior.
This role collaborates with platform and rendering engineers; it does not certify latency
or physical pen feel from screenshots.

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

Deliver a recommended design; request owner input only for a consequential unresolved
product choice. Routine layout choices can proceed within the agreed scope.

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

