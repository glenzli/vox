# Vox JSON operations

`vox-cli capabilities` lists actual operations, voice identities and speech languages.
`vox-cli request` reads one JSON object from stdin; `vox-cli serve` reads newline-delimited JSON
and emits correlated responses. Inspect `ok` on every response; errors carry a stable `error` code.
Never infer success from process exit or a health endpoint.

```json
{"action":"create","path":"/absolute/new/Story.vox","title":"Story","request_id":"1"}
```

Mutations return `result.project`, `result.path` and `result.analysis`. Use the returned
`project.version` as the next mutation's `version`; never guess or increment it on the client.

| Action | Additional input |
| --- | --- |
| `capabilities`, `probe` | No project; optional explicit diagnostic `endpoint` for probe |
| `create` | New absolute `.vox` path and title |
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
preset bindings. `production_script` uses canonical rules; `plain` speaks exact input text.
Role/cue prerequisites remain distinct from provider availability.

Render requires the working draft to match the adopted source and produces a candidate. Only
accept advances the audio head. Serve emits `event:"progress"` with the active request ID before
its final response. Stop never undoes accepted/exported outputs. After render failure, read status
before another mutation because its journal can advance the version.

Playback returns a verified temporary WAV and real PCM peaks without playing it. Export never
clobbers a file. No command accesses another app's database, grants Runtime authority, launches
arbitrary shell commands or silently enables cloud execution.
