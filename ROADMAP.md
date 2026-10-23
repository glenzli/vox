# Vox roadmap

## First independent production slice

- Independent `.vox` projects, recoverable working drafts and exact source/audio revisions.
- Bounded Shape-to-Vox transfers into independent projects: exact accepted text/audio, historical
  source pins, separate unfinished writing/speech drafts, cue bytes and visible draft restoration.
- Plain narration and Shape-compatible multi-role scripts, listening repeats, pauses, delivery
  and explicit sound-cue bindings.
- Own Infer Runtime App, nine voice presets, local synthesis, segment progress and stop after
  the current segment. Whole-render retry cache remains in-process. Segment production keeps immutable candidates and explicit selections in the project.
- Selected audition with actual PCM peaks, explicit acceptance, source restoration and WAV export.
- Neutral icon-led desktop, English/Chinese and live light/dark/system themes.
- Echo independent editor through pinned file handoff and explicit WAV return for review.
- Installed script fonts and size, live preview/reset, syntax highlighting and logical line numbers.
- Versioned project reference voices: microphone recording/import, exact transcript, permitted local
  use, original audition, clone preview and default/per-role selection.
- Eight procedural sound effects plus imported/generated local sound/music candidates. Explicit
  acceptance prepares mono 24 kHz script PCM while retaining the original and its Job/prompt receipt.
- Voice and sound working copies open in Echo; edited exports return as new reviewed materials.

## Production workspaces

- Persistent segment alternatives, local re-generation and explicit per-segment selection; exact text,
  pronunciation, voice and operation identity controls reuse. Assembly creates a new whole-take candidate.
- Scoped pronunciation rules retain source bytes and belong to each source revision.
- SRT/VTT export uses actual assembled audio durations, including pauses and repeated passages.
- Independent sound-effect, ambience and short instrumental music workspaces using existing local
  Runtime routes; bounded batches, retained partial results, loop audition and original-quality export.
- Description-based voice design through the existing Runtime speech contract: bounded samples,
  durable receipts, explicit named synthetic-reference acceptance and reuse through voice cloning.
- Project-owned chapters, shared cast/pronunciation, bounded text imports, resumable batch candidates,
  unchanged-segment reuse, explicit chapter acceptance and ordered WAV/caption/source export.
- Chapter-specific Echo handoff and returned candidates; edited audio exports without stale captions.
- Audio-pinned subtitle review and local forced alignment of a confirmed transcript, with explicit
  candidate acceptance and SRT/VTT export after Echo edits.
- Ordered chapter compilation with silent gaps, explicit format normalization, merged subtitles,
  chapter markers and revision-pinned provenance; reviewed preflight prevents stale exports.
- Offline subtitle correction with bounded cue audition, text/time edits, split/merge and strict
  plain SRT/WebVTT import. Saved corrections remain candidates with immutable parent/import evidence
  until explicitly accepted for chapter and compiled delivery.
- Complete songs and singing production are outside the product scope.

Implementation, focused checks, packaged checks and real Runtime/Echo evidence are recorded
separately in [validation](docs/VALIDATION.md). Source implementation is not a public release.

## Subsequent forms

Production labels guide authoring without restricting the grammar to a genre. New narration,
dialogue and listening templates reuse the same pipeline. Larger forms need real contracts:

1. Broader Shape project migration beyond the implemented selected speech transfer, with explicit
   material selection and independently accepted compatibility.
2. Performance-driven voice conversion with its own audio-to-audio contract, distinct from reference cloning.
3. Time-aligned or video-led dubbing with its own media and synchronization boundary.
4. Native Echo completion receipts/live return after proving the bounded file workflow.
   Advanced adjustment and mixing remain Echo responsibilities.

Do not add generic plugin machinery, duplicate Echo's editor, or couple application databases
in anticipation of those forms.
