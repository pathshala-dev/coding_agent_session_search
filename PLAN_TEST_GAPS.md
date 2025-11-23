# Test Coverage Gap Report (bd-tests-foundation)

## Current coverage snapshot (Nov 23, 2025)
- Unit:
  - `tests/connector_codex.rs` (smoke parse)
  - `tests/ui_footer.rs`, `tests/ui_help.rs`, `tests/ui_hotkeys.rs`, `tests/ui_snap.rs` (help/footer CLI smoke)
  - `search::query` unit (filters+pagination)
- Integration/E2E: none
- Install scripts: none
- Watch/incremental: none
- Logging assertions: none
- Benchmarks: present but synthetic; not asserted in CI beyond runtime.

## High-priority gaps
1) Connectors (real fixtures, no mocks)
   - Need per-connector parse/normalize tests for: Claude Code, Cline, Gemini CLI, OpenCode (DB), Amp, Codex.
   - Cover external_id dedupe, idx resequencing, snippet mapping, created_at handling.
2) Storage / indexer
   - SqliteStorage: schema_version getters, fts rebuild helper, transaction rollback on error.
   - Indexer: full flag truncation, append-only add_messages, since_ts routing, watch state persistence.
3) Search
   - Filters (agent/workspace/time) interaction, snippet highlighting order, pagination boundaries.
4) TUI
   - Snapshot tests for search bar tips, filter pills with clear hotkeys, detail tabs presence.
5) Watch / incremental
   - Integration test to touch files and ensure connector-targeted reindex and watch_state.json bump.
6) Installers
   - install.sh / install.ps1 checksum enforcement (good/bad), path hints, DEST respected using local file:// fixtures.
7) Logging
   - Structured tracing spans for connectors/indexer, captured in tests for key events.

## Proposed test tasks (beads)
- bd-unit-connectors: fixtures + per-connector tests (see below).
- bd-unit-storage: Sqlite schema/version/transaction tests.
- bd-unit-indexer: full vs incremental vs append-only coverage.
- bd-unit-search: filter/highlight/pagination tests.
- bd-unit-tui-components: snapshot tests for bar/pills/detail tabs.
- bd-e2e-index-tui-smoke: seed fixtures, run index --full, launch tui --once, assert logs.
- bd-e2e-watch-incremental: watch run + file touch, assert targeted reindex + watch_state bump.
- bd-e2e-install-scripts: checksum pass/fail, DEST install.
- bd-logging-coverage: tracing span assertions.
- bd-ci-e2e-job: wire above into CI with timeouts.
- bd-docs-testing: README testing matrix + env knobs.

## Fixture plan
- Place under `tests/fixtures/`:
  - codex_rollout.jsonl (small 3-msg)
  - cline_task.json
  - gemini_tmp_example.json
  - claude_project/.claude + .claude.json pair
  - opencode.db (minimal SQLite with 1–2 sessions)
  - amp/thread-123.json

## Next immediate steps
- Add first connector test using fixtures (Codex or Amp) to validate pattern.
- Build tracing test helper to capture spans/logs.
- Add minimal makefile-like script in tests/util for temp dirs and env setup.
