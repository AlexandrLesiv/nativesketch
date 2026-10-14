# 014: Network document source and account integration

Status: waiting. Roles: rust_engine, macos_ui, build_reliability, reviewer.
Dependencies: 007.

Plan and implement a real document-source service once storage/provider/account requirements are selected. Extend the bounded source-adapter contract without changing iPhone-first/offline semantics or silently uploading local drawings.

Acceptance:
- Agree provider, authentication, account/namespace boundaries, service versioning and owner-authorized deployment before connecting production data.
- Test Watch independent authentication and credential storage/expiry/revocation without requiring a reachable iPhone for every fetch.
- Fetch only the selected published document identity; do not substitute another account or similarly named drawing when iPhone is unavailable.
- Exercise stale replies, hash/size/schema validation, interrupted transport, cancellation, cache/account switching, logout and conflicts.
- Define background refresh/battery/network budgets and explicit user-visible offline/error states.
- Supply fixture and real staging-service tests, security-focused independent review and operational limitations. Automatic upload/collaboration is outside this task unless separately scoped.

The current implementation intentionally has no production authentication/service configuration. This task is a backlog definition, not authorization to select or deploy a paid service.
