# Rust Mobile Game Lab

- Read `prompts/codex-poc.md`, `docs/progress.md`, and the assigned issue before work.
- Report to rio in Japanese. Distinguish implementation, build, emulator/simulator, physical device, and performance evidence.
- Only this repository is in scope. Do not change or depend on persona repositories.
- Confirm the session working root and effective `workspace-write`, `on-request`, `auto_review` permissions before implementation or worker execution. A config file alone is not evidence of effective settings.
- The user authorizes repository-local issues, branches, commits, pushes, PRs, and checked/reviewed merges. Use merge commits, never squash, and respect branch protection.
- Use the verified repository-local Git author configuration. Do not add AI attribution.
- Establish minimal contracts before independent worker implementation. Give each worker a separate worktree, issue, owned files, base branch, acceptance conditions, and permission boundary. The integration owner owns shared contracts and Cargo.lock.
- Verify mobile partial embedding early, before integrating game logic and networking. Do not substitute desktop evidence for mobile evidence.
- Review gates: native/FFI lifecycle, multiplayer recovery, then both games/SDK/final acceptance. Fix serious findings and rerun affected checks.
- Do not close issues with unverified acceptance conditions; omit automatic closing keywords on partial PRs.
- Keep `docs/progress.md` current with actual issue/PR URLs, evidence, and next commands. PoC completion requires every mandatory condition, including physical device verification.
