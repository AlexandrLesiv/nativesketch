# Shared read-only Apple adapter

`ReadOnlyDocument` owns a private Rust ABI 2 handle and exposes only import, state,
rendered paths and bounds derived from those paths. It has no edit/selection/history
API. Every replacement validates in a fresh handle; failure retains the previous
copy. Native view navigation never calls Rust editing commands.

`WatchCopyStore` commits a validated candidate and metadata as one atomic envelope,
then swaps the in-memory candidate. `ViewerReplay` exercises this same code on the
host and inside simulator applications. The bundled fixture is produced by the
Rust `portable-fixture` example, not a second document-format implementation.

Source responsibilities are separate: `WatchLibraryStore` validates and persists the
bounded library, `SourceCoordinator` owns request generations and staged replacement,
and `SourceComparison` borrows metadata into the Rust ordering check. Shared source
metadata, catalog persistence, HTTP transfer, cancellation and server endpoint adapters
live under `apps/apple-sources`. Extraction preserves all existing replay assertions.

`node scripts/check.ts --platform apple-viewers` runs host adapter/source replays,
including URLSession fixtures. It does not launch an Apple simulator or prove native
WatchConnectivity, remote/crown interaction or physical-device behavior.
