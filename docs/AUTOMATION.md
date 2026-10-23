# Vox JSON operations

`vox-cli capabilities` lists actual operations, voice identities and speech languages.
`vox-cli request` reads one JSON object from stdin; `vox-cli serve` reads newline-delimited JSON
and emits correlated responses. Inspect `ok` on every response; errors carry a stable `error` code.
Never infer success from process exit or a health endpoint.

```json
{"action":"create","path":"/absolute/new/Story.vox","title":"Story","request_id":"1"}
```

Mutations return `result.project`, `result.path`, `result.analysis` and `result.segment_plan`. Use the returned
`project.version` as the next mutation's `version`; never guess or increment it on the client.

| Action | Additional input |
| --- | --- |
| `capabilities`, `probe` | No project; optional explicit diagnostic `endpoint` for probe |
| `create` | New absolute `.vox` path and title |
| `import_shape_transfer` | New absolute `.vox` `path`, transfer `file`, producer `expected_blake3` for Shape continuation |
| `restore_shape_draft` | `path`, expected `version`, `kind: "authoring"` or `"speech"`; restores without adoption |
| `shape_transfer_receipt` | Existing `path`; reads the preserved original transfer |
| `open`, `status`, `validate` | Existing `path` |
| `save` | `path`, expected `version`, complete `document`, optional title |
| `adopt`, `render` | `path`, expected `version` |
| `cancel` | Serve-mode stop request, after the current segment |
| `accept`, `discard` | `path`, expected `version`, exact `audio_id` |
| `restore_source` | `path`, expected `version`, exact `source_id` |
| `playback`, `export` | `path`, exact `audio_id`; new `destination` for export |
| `import_script` | `path`, expected `version`, selected UTF-8 `file`, up to 64 KiB |
| `import_cue` | `path`, expected `version`, exact cue `label`, selected WAV `file` |
| `voice_preview` | Exact voice alias and optional short text; temporary sample only |
| `begin_echo` | `path`, expected `version`, accepted `audio_id`, new external destination |
| `review_echo` | `path`, expected `version`, original `handoff`, returned file, explicit `allow_pcm24` |

Read the complete initial document from the project. Preserve its options revision and versioned
preset bindings. `production_script` uses canonical rules; `plain` speaks the authored text after explicitly configured pronunciation rules.
Role/cue prerequisites remain distinct from provider availability.

Render requires the working draft to match the adopted source and produces a candidate. Only
accept advances the audio head. Serve emits `event:"progress"` with the active request ID before
its final response. Stop never undoes accepted/exported outputs. After render failure, read status
before another mutation because its journal can advance the version.

Playback returns a verified temporary WAV and real PCM peaks without playing it. Export never
clobbers a file. No command accesses another app's database, grants Runtime authority, launches
arbitrary shell commands or silently enables cloud execution.

## Project material operations

All project writes use the current `version`; `status` returns the updated manifest/analysis.
`import_reference` takes `name`, exact `transcript`, local PCM16 WAV `file`, `consent: true` and an
honest `recorded` origin. `bind_reference` takes immutable library `id` and a `role` (empty for default).
`reference_preview` takes `id` and bounded `text`, producing temporary synthetic audio without
acceptance. Arbitrary file paths never become a voice identity or a provider credential.

`add_builtin` takes one catalog `key`; `import_sound` takes a WAV `file` and `name`.
`generate_sound` takes `name` and `spec: {prompt, seconds: 1..30, seed: u32, music: bool}`. It creates
only a library candidate; `accept_sound` prepares a script copy, and `bind_sound` binds its `id` to a
cue `label`. `discard_sound` removes only unaccepted candidates. `material_file`, `export_material`
and `material_echo` take `kind: "voice" | "sound"` and `id`; export additionally takes a new
`destination`. Echo edits are reimported as new material, preserving originals.

`cancel` stops before the next sound request. A validated current sound result is retained as a
candidate; no provider-side interruption is claimed. The worker admits one foreground generation at a time.
`test_studio_live.py` explicitly exercises real local generation using a synthetic reference in a
new external project; `--resume` continues that named test project after an interrupted check.

Shape transfer schema, exact manifest fencing, replay behavior and preserved-draft semantics are documented in [Shape transfer](SHAPE_TRANSFER.md).

## Segment production and pronunciation

Save the complete document, then adopt it before generating segments. `pronunciation_rules` is an
optional document array of `{text, spoken, language?, role?}`. Empty rules are omitted, preserving
legacy document digests. Rules affect spoken input; they never replace authored source bytes.

| Action | Additional input |
| --- | --- |
| `segment_plan` | Existing `path`; returns exact current draft plan and reusable candidates |
| `render_segment` | `path`, expected `version`, exact `segment_key` from current plan |
| `accept_segment` | `path`, expected `version`, `segment_id`; explicit selection for assembly |
| `segment_playback` | `path`, `segment_id`; verified WAV and peaks |
| `assemble_segments` | `path`, expected `version`; all current segments must have selected takes |
| `export_subtitles` | `path`, exact `audio_id` with measured or accepted aligned timing, new `destination`, `format: "srt"` or `"vtt"` |

Assembly creates a whole-take candidate. `accept` remains the only operation advancing the whole
audio head. Repeated passages reuse the same PCM; subtitle timings include local pauses and cues.
Caption text comes from the source, not its pronunciation substitutions. A segment candidate is
never silently accepted. After failure or cancellation, read status before another mutation.

## Sound batches

`generate_sound_batch` adds `category: "sfx" | "ambience" | "short_music"` and `variations: 1..4`
to the existing `name`/`spec` request. Category selects the local SFX or instrumental-music route.
Each variant uses a distinct seed. No complete-song mode is provided.

The response includes `sound_batch: {batch_id, requested, completed, state, error}` alongside the
latest project snapshot. Check `state`: `completed`, `cancelled`, or `partial_failed`. Completed
candidates survive cancellation or a later variation failure. Inspect each result before acceptance.
Loop audition is a playback convenience; it does not claim seamless loop generation.

## Voice design

`generate_voice_design_batch` takes `path`, current `version`, `variations: 1..4` and
`spec: {description, text, language}`. It uses Vox's independent `speech.design_voice` grant through
the existing official speech SDK. Progress counts persisted samples, not tokens. Its final snapshot
includes `voice_design_batch: {run_id, requested, completed, state, error?}`; inspect `state` even
when the enclosing request succeeds. Partial failure and cancellation retain completed candidates.

Candidates live in `project.voice_design_candidates`, with exact input, immutable original audio,
synthetic disclosure and validated Job/physical-model provenance. `material_file`, `export_material`
and `material_echo` accept `kind: "voice_design"`. `accept_voice_design` takes `id` and a `name` and
saves a synthetic reference in `voice_library`. The ordinary `bind_reference` and clone preview
operations then apply. `discard_voice_design` removes only an unaccepted candidate. Reopening marks
lost running jobs as `interrupted_unknown` and never regenerates them automatically.

## Long-form chapters

Collection operations use the parent `path` and its current `version`. Chapter paths are derived
and checked by core against both project identities; arbitrary paths cannot be added to a collection.
Responses include `parent_path`, `parent_project` and `chapters`. `chapter_open` also returns the
selected child's normal `path`, `project`, `analysis` and `segment_plan` so existing editor operations
can be reused. Refresh the collection after modifying a child. `chapters_error` reports unavailable
chapter summaries instead of presenting an empty list as successful recovery.

| Action | Additional input |
| --- | --- |
| `chapter_list` | Read current chapter summaries |
| `chapter_open` | `chapter_id`; recover and open that owned child |
| `chapter_add` | `title`, optional `text` and `format` (defaults to empty plain text) |
| `chapter_save` | `chapter_id`, complete `document`; retains authored text/format with shared settings |
| `chapter_rename` | `chapter_id`, `title` |
| `chapter_reorder` | Complete ordered `chapter_ids` list |
| `chapter_archive` | `chapter_id`, optional `archived` (defaults to true); preserves files/history |
| `chapter_import` | `files`: selected UTF-8 TXT/Markdown paths, imported in order; originals and byte ranges retained |
| `chapter_shared_settings` | `chapter_id`; explicitly publish this child's cast/defaults/pronunciation/cues to the collection |
| `chapter_batch` | Optional `chapter_ids` (all active by default), optional explicit `retry_interrupted: true` |
| `chapter_accept` | `chapter_id`, exact reviewed `audio_id` |
| `chapter_export` | Optional `chapter_ids`, new `destination` directory |
| `chapter_begin_echo` | `chapter_id`, accepted `audio_id`, new handoff `destination` |
| `chapter_review_echo` | `chapter_id`, original `handoff`, returned `file`, explicit `allow_pcm24` |

Imports are bounded to 8 MiB per file and 64 KiB per chapter, with at most 256 chapter records.
Markdown uses repeated level-one headings, or repeated level-two headings when there are not
multiple level-one headings. Headings inside fenced code do not split chapters. Preamble and exact
source bytes are retained; oversized sections and plain text split at paragraph boundaries where
possible before the chapter limit is reached.
Batch generation pins shared settings, adopts the current chapter drafts and reuses matching segment
candidates. It never accepts segments or chapter audio. Progress adds `chapter_id`, `chapter_index`,
`total_chapters`, `completed_segments`, `total_segments` and `status`; ordinary `completed`/`total`
reflect the current chapter. The final `chapter_batch` journal has a `status` and per-chapter entries,
including reused-segment counts and errors. Stop retains the current completed model output and
prevents further dispatch; later batches reuse it. Unexpected interruption requires explicit retry.

Chapter export rejects stale or unaccepted audio. It writes ordered WAVs plus `manifest.json` with
authored scripts, source/audio identities and provenance. `srt`/`vtt` are relative paths when the
accepted audio has measured assembly timing or an explicitly accepted alignment, otherwise null with `subtitles_unavailable_reason`.
Echo returns require review and explicit chapter acceptance; arbitrary edits do not retain old
caption timing. Exports do not overwrite an existing destination.

`scripts/test_longform_live.py` exercises real design, reference reuse, two-chapter cancellation and
resume, unchanged-result reuse, changed-segment generation, accepted exports and the independent
Echo editor in a new external project. It never records a human voice or modifies a user project.

## Subtitle alignment and review

`subtitle_preview` takes `path`, `audio_id` and an optional `candidate_id`. Its ordinary snapshot
adds `track`, `track_error`, `transcript`, `language`, `duration_ms`, `accepted_audio`, pending `candidates` and
`runs`. Missing timing returns a null track with a reason. The transcript suggestion comes from
the selected audio's immutable source, including effective spoken pronunciation and repeats;
after editing, the caller must supply the words actually present in the recording.

`align_subtitles` takes `path`, expected `version`, accepted `audio_id`, and
`spec: {transcript, language}`. Language is one of the explicit speech language names; `auto` is
not accepted. It uses Vox's own local `audio.align` grant through the pinned official SDK.
Each request is bounded to 64 KiB of transcript, 128 MiB of PCM16 WAV and 30 minutes of audio.
An unavailable transcript suggestion reports `transcript_error` without hiding an existing valid
caption track; long recordings should be divided into chapters before realignment.
The final snapshot includes `subtitle_outcome: {run_id, candidate_id, state, error}`. Check the
outcome even when the outer request succeeds. Stop waits for the current request and retains a
completed candidate; it does not claim to abort provider inference. Lost running journals become
`interrupted_unknown` and are never automatically resubmitted.

`accept_subtitles` and `discard_subtitles` take `path`, expected `version` and subtitle `id`.
Only explicit acceptance selects subtitles for that exact audio revision. Previewing a candidate
never changes export selection. `SubtitleTrack` reports `source: "measured_segments"` or
`"forced_alignment"`, `"manual_revision"` or `"imported_subtitles"`, exact audio/source pins,
optional `revision_id` and `alignment_revision`, and ordered
`cues: [{start_ms, end_ms, text}]`. Alignment timing is an estimate to review, not an ASR accuracy
claim. Measured timing keeps authored captions; alignment preserves the caller's confirmed wording.

### Offline subtitle correction and import

`subtitle_edit` takes `path`, expected `version`, accepted `audio_id`, full
`cues: [{start_ms, end_ms, text}]` and optional `candidate_id`. Without a candidate ID, the base is
the current selected or measured track; an ID selects that exact candidate or accepted revision.
The operation creates a manual candidate with immutable parent evidence. The response is the normal
project snapshot plus `subtitle_candidate_id`; it does not change the accepted subtitle selection.
Splitting, merging and text/time correction use this same bounded cue contract.

`subtitle_import` takes `path`, expected `version`, accepted `audio_id`, local `file`, exact lowercase
`format: "srt" | "vtt"` and optional explicit `language`. Files are bounded to 1 MiB before parsing.
The supported dialect is plain UTF-8 SRT/WebVTT, with optional BOM and CRLF. SRT uses bounded numeric
cue labels and `HH:MM:SS,mmm` timestamps. WebVTT uses `WEBVTT`, optional plain cue identifiers
and `HH:MM:SS.mmm` or `MM:SS.mmm` timestamps; time order, not labels, governs validation.
Markup, layout settings and unsupported blocks
or entities are rejected, not silently discarded. Literal characters are escaped on export and
decoded on import. The imported bytes and format remain immutable provenance. The response likewise
contains `subtitle_candidate_id`; use the existing preview and explicit accept/discard operations.

Both operations reject invalid/overlapping times and cues outside the audio duration rounded to
the nearest millisecond. The
script, accepted audio and genuine alignment receipts remain unchanged. `revision_id` identifies
any saved subtitle revision; `alignment_revision` is present only for model-aligned tracks. Accepted
manual/imported revisions are consumed by ordinary SRT/VTT, chapter and compilation exports.

## Compiled chapter delivery

`chapter_compilation_preflight` takes parent `path`, current `version`, ordered `chapter_ids`
(empty means all active chapters), and `options: {gap_ms, audio_mode, subtitles}`.
`audio_mode` is `preserve` or explicit `normalize_mono24000`; `subtitles` is `required` or
`audio_only`. The silent gap is 0–600000 ms. Defaults are zero gap, preserve and required subtitles.
Its direct response includes `ready`, collection and per-chapter `issues`, the output contract
and `fingerprint`. Preserve mode requires matching formats; normalized output is mono 24 kHz PCM16.

`chapter_compile` adds the reviewed preflight `fingerprint` and a new `destination` directory.
The parent and selected chapters are locked and revalidated before publication; changed source,
accepted audio or subtitle selection returns `compilation_preflight_changed`. The response is
`{exported, destination, manifest}`. The package contains `compilation.wav`, `manifest.json`,
`chapters.json`, `chapters.ffmetadata`, source scripts, and merged SRT/VTT when required.
Audio-only mode does not export partial or stale captions. Inputs and output are bounded to
512 MiB and total duration to 24 hours; the smaller byte limit may govern first. No existing
destination is overwritten, and compilation does not create or accept a replacement project take.

`scripts/test_delivery_live.py` copies an existing isolated long-form fixture, performs a fresh
Echo edit, real local alignment, explicit subtitle acceptance and compiled export, and checks
format choice, stale preflight rejection, no-clobber export and unchanged original input.
