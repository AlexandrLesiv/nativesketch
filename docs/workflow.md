# Agent workflow

## Roles and scheduling

Primary chat: lead/integrator, GPT-6 Astra high.
Specialists in `.codex/agents/`: ui_ux_designer, rust_engine, macos_ui, windows_ui,
rendering_performance, reviewer, build_reliability.
Three subagents maximum; at most two concurrent writers and one feature in flight.
Use designer and reviewer at the relevant stage rather than running every role throughout.
Windows work begins when its milestone is selected; it is not a parallel launch project.

Lead and rust_engine own architecture plans: types, dependencies, state ownership, commands/
effects, threading, file map and invariants. Designer owns native acceptance. build_reliability
owns Node tooling, executable coverage reporting, diagnostics infrastructure and failure
fixtures. Implementers own tests/logging; rendering_performance owns measurement validity;
reviewer independently checks evidence. No extra permanent architecture/logging/test agent.
Explicitly assign qualified workers for Android/Linux/web; those specialist configs do not exist.

The [refactoring plan](refactoring-plan.md) starts from `codex/github-pages@712059d5`.
Do not import the excluded comparison-results branch. New task branches use `codex/`.
Preserve scripts until task 016 proves Node parity; new owned automation is JavaScript.

## Lifecycle

Ready task -> interaction/contract design -> isolated implementation -> integration ->
appropriate checks -> fresh review -> repairs and affected checks -> human merge approval.
The task board in `tasks/board.json` is the durable status source. Use `node scripts/board.ts`
to inspect it. Update it only when the corresponding state is true. Status `done` means
the owner accepted the result; `review` means implementation/evidence await that decision.

Workers receive objective, acceptance cases, owned paths, base revision, dependencies,
relevant documents, required evidence, and a stopping condition. One owner changes each
shared interface. Workers can propose contract changes; lead reconciles consumers first.

Use Codex-managed worktrees when available. For a manual local checkout, the lead can
use `node scripts/new-worktree.ts task-001-rust` from a clean primary checkout.
It creates an ignored local worktree and `codex/task-001-rust` branch from HEAD.
Worktrees do not isolate ports, simulators, data directories, GPU loads, or credentials.
Pass workers their actual checkout path and serialize operations that share those resources.

## Review and authority

Workers may implement, test, repair, and commit on task branches. They do not merge main
or publish releases. The owner reviews a concrete running result and evidence at the end.
Do not ask permission for routine local development already covered by the task.
After two unsuccessful repair cycles with no new evidence, lead changes the hypothesis,
narrows the task, or asks a necessary product/hardware question.

Role restrictions such as review-only are instructions, not security boundaries. Codex
subagents inherit the parent's permissions. This setup does not change global settings,
authentication, network permissions, or sandbox mode. Use the app's appropriate project
permission selection. New project trust is handled by Codex's normal opening flow.

## Evidence packet

Use `templates/change-report.md`: scope, tested revision, checks and results, screenshots/
recordings when relevant, performance data, independent review, limitations, and next step.
Local large artifacts go in ignored `artifacts/`; record reproducible commands and useful
small fixtures in the repository. Do not commit personal data or full raw model transcripts.

## Durable context

AGENTS.md holds stable rules. Product decisions are in docs/product.md and decision records.
Feature design belongs with its task; code and current contracts are authoritative over
old conversation summaries. Update the relevant document when an accepted decision changes.

## Automate later

No new scheduler, unattended model calls or auto-merge is installed by this plan.
Existing GitHub/Pages configuration is preserved; remote protections are not inferred.
After about ten accepted changes, assess review time, repair cycles, post-merge defects,
elapsed time, and cost/usage where available. Compare the current team with a single
implementer plus reviewer. Add a queue/runner via Codex SDK only for observed repetition.
Use real Mac/Windows hosts for their native integration tests. Headless CI cannot certify
physical stylus behavior or end-to-end pen latency.

## Documentation used (checked 2026-10-14)

- https://learn.chatgpt.com/docs/agent-configuration/subagents
- https://learn.chatgpt.com/docs/config-file/config-reference
- https://learn.chatgpt.com/docs/agent-configuration/agents-md
- https://learn.chatgpt.com/docs/environments/git-worktrees
- https://developers.openai.com/blog/rethinking-skills-and-prompts-for-gpt-6-astra

