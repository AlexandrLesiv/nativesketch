# Watch document sources

The Watch is an independent, view-only app (`WKSupportsRunningWithoutiOSApp`),
with optional iPhone integration. Cached documents open without either source.
Open Details → Library to select, or Details → Refresh to retrieve a selected copy.
An active pan/zoom stages geometry updates until Apply Update. Metadata-only updates
advance the version without resetting navigation. Errors retain the previous copy.

An explicit iPhone Send Copy action stores a shared snapshot in the iPhone catalog.
Its document UUID is stable for the editor session; successful New/Open creates a new
UUID. File identity across reopening is not yet implemented. Subsequent explicit
shares link to the previous shared revision. Watch requests cannot read arbitrary
phone files. Requests use protocol 2, UUID correlation, 16-item pages under 48 KiB and
bulk `transferFile`; a 4 second deadline allows fallback. Late correlated file replies
are ignored. Unsolicited explicit copies enter the library without changing an
existing selection. Legacy v1 transfers/cache migrate to isolated phone-only IDs;
the legacy cache remains intact. The Watch stores at most 8 documents /48 MiB encoded.
The phone catalog holds at most 128 explicitly shared drawings.

Phone is preferred. Published/synced documents can fall back to the server for the
same namespace/document identity. Phone-only drafts accurately report that no
server copy exists. Rust decides duplicate/stale/conflict/identity acceptance;
Swift verifies SHA256 bytes and imports into a candidate Rust document before an
atomic cache write. Request generations prevent obsolete requests selecting a file.

## Optional development server contract

No production server, credentials, sign-in UI or authentication provider is bundled.
The production configuration is deliberately unconfigured. For a controlled public
fixture server, set `NativeSketchServerURL` in the Watch Info.plist at build time to
an HTTPS base URL without user info/query/fragment. Never put credentials in this key.
A future authenticated provider must supply account separation and Keychain-based
credentials; current transport is for public/developer data only.

- GET `<base>/drawings`: JSON array, max 128 descriptors/256 KiB.
- GET `<base>/drawings/<encoded-document-ID>?namespace=<namespace>&revision=<revision>`:
  raw `.nsketch` bytes matching the catalog hash, max 8 MiB.
- Descriptor keys: `namespaceID`, `documentID`, `revisionID`, `parentRevisionID`,
  `contentSHA256` (64 lowercase hex), `serverGeneration` (UInt64), `kind` (0 draft,
  1 published), `title`. Published revisions require generation>=1.
- Namespace is an account/library identity, not a title. The server must return
  immutable bytes for a revision, or fail; clients do not infer identity from names.
- HTTPS and same-origin redirects only. Streaming and declared sizes are bounded,
  credentials/cookies/cache disabled, request/resource timeouts 15/20 seconds.

`./scripts/check-apple-viewers` runs the actual Rust policy/validation/cache and
coordinator replay plus URLSession URLProtocol transport tests. Simulator Watch
`--integration-test` runs the same DEBUG replay. Fixture code is absent from device
builds. Real WatchConnectivity file delivery still requires paired physical devices.
