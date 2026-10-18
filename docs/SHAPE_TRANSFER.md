# Shape speech transfer

Vox consumes `vox.shape-transfer/1` file packages into a new independent `.vox` project. Shape
publishes selected exact accepted text, unfinished writing and speech drafts, cue WAVs and optional
already accepted audio. Vox never opens or writes the Shape database and uses only its own Runtime
credential when the user later asks to synthesize.

## Import and open

The Shape producer returns `manifest_path` and `manifest_blake3`. Pass both unchanged:

```json
{"action":"import_shape_transfer","file":"/absolute/transfer/manifest.json","expected_blake3":"<exact producer hash>","path":"/absolute/Story.vox"}
```

The desktop exposes the same use case and opens the returned project:

```sh
Vox --import-shape-transfer /absolute/transfer/manifest.json --project /absolute/Story.vox --expected-blake3 HASH
```

`file` may name the package directory or its manifest. A manual standalone CLI import can omit
`expected_blake3`; a Shape-initiated continuation must include the producer's exact hash. Invalid
schemas, source pins, content lengths, WAV contracts, changed hashes, absolute/escaping relative
paths and symlinked materials fail before a final project exists. The entire package is admitted
before a private staging project is published. Interrupted or cancelled imports remove their
staging directory. Cancellation after publication does not undo a completed project.

An identical manifest replay at the same destination returns the existing project without changing
its version, draft, accepted heads or later user edits. Another transfer at that destination fails.
The destination is explicit; importing never appends to an unrelated existing project.

## Preserved history and drafts

The initial working draft is the exact unfinished writing draft when one exists. Otherwise the
preserved speech draft is shown, followed by accepted text when neither unfinished draft applies.
Writing and speech drafts remain separate because the speech draft may target an older accepted
text revision. Vox does not silently apply those older settings to a different writing draft.

The sidebar shows the current working-draft kind and **Restore Shape writing draft** /
**Restore Shape speech draft** actions. The speech action restores its exact pinned input text,
default voice and catalog revision, language, pace, delivery and role/cue bindings. Its executable
mode follows the saved speech operation, while the original accepted text format stays in the
receipt and accepted-source history. Restoring
changes only the working draft. It does not adopt a script, generate a take or replace accepted
audio. Both original drafts remain available after later edits.

A legacy speech draft with no configuration stays unconfigured, including after restore. The
voice remains unselected and execution is blocked until the user explicitly chooses supported
settings. Unsupported catalogs, incomplete reference voices or operations Vox cannot reproduce
are retained with explicit migration issues; they are never silently replaced by a current preset.

Already accepted Shape WAVs retain their exact bytes, original receipt and Shape provenance. Their
source binding uses the historical text actually named by the original transformation, even when
the current accepted script differs. An audio item without a provable input-text pin remains
external imported history with an explicit issue. Neither case claims a new Vox synthesis.
New renders and returned edits still create candidates and require explicit acceptance.

Every transferred material and the exact original manifest become immutable content-addressed
objects in the new project. `project.shape_import` records their references, preserved drafts and
issues. `shape_transfer_receipt` reads the original structured receipt; it does not consult Shape
or the original package. The imported project reopens independently after those sources disappear.

## Automation and bounds

`restore_shape_draft` takes the normal `path` and expected `version`, plus `kind: "authoring" |
"speech"`. It uses the same manifest compare-and-swap as other project writes. `shape_transfer_receipt`
is read-only and takes `path`. All responses use the regular correlated JSON envelope.

Admission allows at most 8 MiB of manifest JSON, 4 MiB per text material, 128 MiB per accepted WAV,
128 cue materials of at most 8 MiB each, and 168 MiB of unique transferred bytes. Cues must be exact
24 kHz mono PCM16 WAVs. The resulting Vox manifest keeps its existing 16 MiB limit. Imported text
above the 64 KiB synthesis limit is preserved with an execution issue. Original graphs, unknown
saved authoring fields and source receipts are evidence, never executable adapters.

The offline owner tests cover history, both drafts, unconfigured/unsupported settings, replay,
manifest fencing, compare-and-swap, tampering, path escape, size admission, cancellation and reopen
without the package. Native `--smoke-shape-transfer OUTPUT` can accompany the import arguments to
verify the real open path and activate the visible restore actions using an isolated fixture.
