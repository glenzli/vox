# Vox project conventions

Vox owns an independent script-led voice production application. Existing speech remains in Shape
for compatibility. Read README.md, ROADMAP.md and docs/ARCHITECTURE.md before changes.

- Do not claim a runnable application, release or completed migration without evidence.
- Reuse existing script and synthesis owners when implementation is explicitly commissioned.
- Preserve script grammar, speaker choices, cue bytes, source revisions and accepted audio.
- Give Vox its own application authorization; do not impersonate Shape or mutate another app's
  database.
- Keep source provenance and licenses when moving code.
- Build products and generated media stay outside the checkout.
- Use managed shared-workspace coordination for concurrent development and Git publication.

- `vox-core` owns scripts, synthesis, independent persistence and explicit audio acceptance.
  CLI and Qt desktop share these use cases.
- English is the desktop message identity; every Simplified Chinese translation stays complete.
- Keep adjacent Rust regression tests with their semantic owners. Validate through
  `scripts/build_and_promote_debug.sh`; use isolated projects for real Runtime/Echo checks.
