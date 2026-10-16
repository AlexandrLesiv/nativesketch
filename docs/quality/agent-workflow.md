# Agent workflow: reviewable architecture and quality ownership

Policy updated 2026-10-16. These are agent obligations, not claims that automated gates
already enforce them. Owner-requested android_ui and ios_ui implementation roles are now
configured; no extra permanent designer/quality coordinator or hosted service is introduced.

## Architecture review before implementation

Lead + rust_engine prepare a short decision packet for changes to language/runtime
boundaries, a custom protocol/serialization format, document format, mutable-state ownership,
thread/executor topology, renderer, security/trust model, or a major framework/dependency.
Include the problem, simpler alternatives (including no change), concrete API/type examples,
dependency diagram, lifetime/thread/error rules, performance/security risks, affected hosts,
migration/rollback/removal plan and acceptance evidence. Record owner approval and scope
in the decision document; do not invent an approval date or infer it from agent agreement.

Ask for the owner's review explicitly before implementing the material choice. A general
"proceed with refactoring" or lead-approved contract does not authorize such a choice.
Already explicit owner approval remains valid; do not re-ask for routine implementation
within it. Material deviations need renewed review. Continue independent authorized work
while waiting. Routine extraction, bug repairs and tests inside accepted boundaries do not
need an architecture permission cycle. The typed-binding direction is owner-approved;
its concrete ABI review is specified in task 022A.

## Task brief and bounded changes

New pages/screens and tools follow the [Figma approval gate](../design/principles.md):
designer proposal, early native feasibility, owner approval of an identified version,
platform implementation, then designer inspection of the running app. Routine fixes inside
approved designs proceed without a fresh full design cycle. Missing access never waives
owner approval. These are role boundaries, not a need to run every specialist on every task.

Each brief identifies scope/non-goals, starting revision, owned paths, contract owner,
acceptance/invariant IDs, dependencies, diagnostic events, thread/resource lifetimes,
required real-host evidence and stop condition. Note which decisions are owner-approved.
Use responsibility-based types/modules, not speculative class hierarchies or arbitrary
200-line fragments. Explain why any new crate/framework/abstraction earns its maintenance
cost. List which old code will be removed and when; compatibility paths need an exit gate.

Implement one vertical behavior slice at a time. Separate mechanical extraction from
behavior, transport and scheduling changes so regressions are attributable. Use small
reviewable commits. Do not repeat the previous broad multi-platform diff pattern.

## Existing roles and added responsibilities

| Role | Required focus |
| --- | --- |
| Lead/integrator | Scope/owner review, decision record, narrow integration, dependency and retirement inventory, honest status/handoff |
| rust_engine | Shared semantics, typed ABI, single ownership, transaction safety, unsafe/FFI threat analysis, invariant/differential tests |
| macos_ui / ios_ui / android_ui / windows_ui / assigned Linux or web implementer | Native intent provenance, typed wrappers, native lifecycle/focus, real UI tests, platform diagnostics |
| build_reliability | Reproducible generated declarations, lint/dependency/size gates, diagnostic sink/redaction tests, failure injection and trustworthy evidence |
| rendering_performance | Buffer/copy/allocation costs, queue age and tail latency, cache correctness, bounded tracing overhead, reproducible visible benchmarks |
| ui_ux_designer | Coordinate shared behavior and platform-specific Figma/review briefs; native hierarchy, accessibility, states, focus, appearance, resize and input variants |
| Fresh reviewer | Independent diff/evidence review; explicit maintainability, security, ownership, logging and performance-risk pass |

Use android_ui for Android/ChromeOS native delivery and ios_ui for iPhone/iPad plus explicitly
assigned visionOS 2D. The lead assigns Linux/web and TV/Watch viewer ownership explicitly,
with one writer per shared Apple contract/module. The designer requests focused platform
design reviews through the lead; no recursive delegation or larger concurrency budget.
Use a focused existing reviewer for high-risk security questions (FFI, imported
files, providers, credentials/network when scoped). Do not add a permanent architecture,
logging or security agent just to create another approval layer. Reconsider specialization
only if repeated tasks demonstrate a concrete expertise or capacity gap.

## Structured logging responsibilities

Implementers instrument shared command/work boundaries and native intent origins, with
stable event codes and correlated IDs. Define the schema and privacy allowlist before new
events. See [typed migration contract](../architecture/022-typed-bindings-migration.md).
Never use ad-hoc prints as a substitute for required production diagnostics. Temporary
debug prints must not leak into shipping paths. No raw document dumps, paths, credentials,
unbounded logs, per-sample production output or logging failures that interrupt editing.

Reliability owns bounded queues/sinks/rotation/drop counters and fault/redaction fixtures;
feature implementers own events and behavior tests. Performance measures logging overhead.
Reviewer checks coverage at action/result/failure boundaries and duplicate/misleading events.
Logs support diagnosis; assert application state in tests instead of treating a PASS log
as functional proof. Logging and recovery journals have different guarantees.

## Review and evidence gates

- Maintainability: clear purpose/dependency direction, <=200 physical source lines,
  no duplicated policy, no unnecessary compatibility path, predictable public interface.
- Security: validate hostile imports and FFI values, bound allocations, protect ownership,
  avoid raw pointer misuse and unsafe callbacks, keep credentials/content out of logs;
  check dependency provenance and test-only controls are absent from ordinary artifacts.
- Functionality: acceptance IDs map to discovered tests; command/adapter/native E2E remain
  distinct. Exercise negative, cancellation, recovery, shutdown and stale-result paths.
- Performance: budgets require evidence; benchmark suspected hot-path changes rather than
  assuming more threads or a new binding is faster. Keep desktop permission requirements.
- Visual/native design: running-app inspection on reserved hosts plus accessibility and
  native behavior tests. Screenshots help visual review but cannot certify input behavior.
- Evidence: record exact source/build, host/device, attempts, results and gaps. Independent
  review is real review of the diff; authored documentation is not execution evidence.

Completion packet: behavior change, files/responsibilities, owner decisions, checks/results,
performance and design evidence where applicable, remaining risks, old-path retirement,
and next bounded task. Fresh review after material fixes; human approval for main merge.
