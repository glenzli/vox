# Vox roadmap

## First independent production slice

- Independent `.vox` projects, recoverable working drafts and exact source/audio revisions.
- Plain narration and Shape-compatible multi-role scripts, listening repeats, pauses, delivery
  and explicit sound-cue bindings.
- Own Infer Runtime App, nine voice presets, local synthesis, segment progress and stop after
  the current segment. Retry cache is in-process, not a persistent segment timeline.
- Selected audition with actual PCM peaks, explicit acceptance, source restoration and WAV export.
- Neutral icon-led desktop, English/Chinese and live light/dark/system themes.
- Echo independent editor through pinned file handoff and explicit WAV return for review.

Implementation, focused checks, packaged checks and real Runtime/Echo evidence are recorded
separately in [validation](docs/VALIDATION.md). Source implementation is not a public release.

## Subsequent forms

Production labels guide authoring without restricting the grammar to a genre. New narration,
dialogue and listening templates reuse the same pipeline. Larger forms need real contracts:

1. Supported Shape-to-Vox project/material handoff with accepted audio, source revisions and
   exact role/cue choices. Retain Shape compatibility until independently accepted.
2. Persistent segment alternatives, re-recording and pronunciation controls with exact source
   and settings invalidation.
3. Runtime-supported voice design or references with explicit consent, disclosure and provenance.
   The initial preset workflow does not expose these as executable.
4. Time-aligned or video-led dubbing with its own media and synchronization boundary.
5. Native Echo completion receipts/live return after proving the bounded file workflow.
   Advanced adjustment and mixing remain Echo responsibilities.

Do not add generic plugin machinery, duplicate Echo's editor, or couple application databases
in anticipation of those forms.
