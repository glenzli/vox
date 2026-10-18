# Vox ownership and durability

CLI and native desktop call the same `vox-core` use cases. QML owns interaction and appearance,
never project commit authority. `WorkerClient` owns correlated process requests and cancellation;
`DesktopBackend` fences responses by project and draft identity. A separate probe worker keeps
an unavailable Runtime from blocking local project work.

Desktop buttons own their text and symbolic-icon colors in both window activation states.
`mac_titlebar` keeps native macOS controls aligned with the single QML toolbar across window
size/state changes. These owners do not mutate project data.
The shared compact QML controls own desktop dimensions, themed dropdowns and numeric/text inputs;
their Qt controls retain keyboard navigation and accessibility. Script typography stays independent
of interface density and is still owned by the editor preferences.

`ScriptEditor.qml` composes the plain-text authoring control, lexical `ScriptHighlighter`, and
`LineNumberGutter`. Syntax colors are presentation only; the Rust parser remains authoritative.
The gutter paints logical source lines from a viewport snapshot prepared on the GUI thread,
keeping wrapped lines and scrolling aligned without adding text or undo entries to the document.
Guarded source loading and an observed-text comparison keep format-only TextEdit notifications
out of the draft mutation path, including when an existing production script is first opened.

`domain::speech_script::reading` projects the authoritative parser plan and authored-note markers
into display-only speech cards and repeat groups. Rejected source lines remain visible with their
issues; execution events, source digests and persisted documents are unchanged. Each projection
retains its exact input text so `ScriptReader` suppresses stale previews while saving. `ReadingEntry`
renders selectable literal text and metadata, and routes explicit line navigation to `ScriptEditor`.
The shared combo restores document/preference selections after translated models are replaced.

`domain::speech_script` migrates Shape's grammar and readiness tests. `assembly` owns complete
control-to-action compilation, local cues, silence, exact replay and PCM validation. The pinned
official Infer SDK owns Discovery, contracts, transport and owner-only credential loading.
Vox checks its own App identity, exact local deployment, no-fallback/zero-cost policy and successful
Job/Attempt receipts. There is no fixed-port fallback in the product.

A `.vox` directory contains `project.json` and `objects/<BLAKE3>`; separate files carry OS locks.
Mutations take an expected version, acquire the lock, reread the version and atomically publish
an updated manifest. Immutable objects are flushed and checked against digest/length at use.
Working drafts remain separate from adopted script revisions and accepted audio.

Rendering pins an adopted script, journals the run and releases the manifest lock for network work.
It rechecks version/source before publishing a candidate. Explicit acceptance revalidates source,
bytes, contract and provenance. Older accepted audio stays readable after rewrites. A separate
render lock prevents overlapping producers. Cancel stops between segments; it does not claim
provider-side interruption. An unlocked journaled run becomes `interrupted_unknown` after process
loss; reopening never redispatches it. Retry cache requires exact text/settings and is in-process.

Echo gets a revision-pinned WAV working copy with its public generated-source declaration. Neither
app opens the other's database. Tickets bind project, script and accepted audio. Manual file return
is recorded as user-selected, not asserted as native completion. PCM24 conversion uses Shape's
tested rounding, retaining ancillary chunks and raw-return hash. A returned WAV is only a candidate;
acceptance advances the audio head. Parent audio retains synthesis provenance, while an edited WAV
is never claimed as the original Runtime output.

Narration, dialogue and listening share one pipeline. Future forms add actual source/selection/
acceptance responsibilities without changing the grammar or prebuilding executor frameworks.

`project::materials` owns immutable reference-voice records, permitted-use receipts, library sounds,
explicit sound acceptance and role/cue binding. Optional empty fields preserve legacy document
serialization/digests. Bindings retain full reference revisions; accepted speech remains tied to its
source. Plain narration materializes only active default-reference inputs, leaving unused script
roles/cues out of execution. New project roots and manifests/objects use owner-only Unix permissions;
reference previews and Echo material working-copy directories are private.

`ReferenceRecorder` owns permission-on-click, input selection, bounded PCM capture, sample conversion,
preview lifetime and cancellation. Its capture is disposable until the project imports it. The desktop
material adapter shares the existing project/version fences and worker admission; project changes
never consume a stale callback. `MaterialsDialog` and `ReferenceDialog` own their forms and review.

The official pinned SDK supplies the separate clone capability (`clone_voice_wav`). A speech executor
serializes calls and materializes exact reference bytes by digest, scopes cloning to local Base at
1×/neutral, and validates the independent clone Job. Mixed scripts carry per-segment preset or clone
receipts; repeats still replay exact PCM. No human voice is captured by automated live tests.

`infer_runtime::sound` owns prompt preparation, sound model choice and local Job/output validation.
Sound choices use the Runtime’s model selector; named request routes are prohibited by that contract.
`materials` provides procedural sounds and explicit low-pass/downmix/resampling for script copies.
Original 44.1 kHz stereo generated audio is retained. Material Echo edits use isolated copies with
synthetic disclosure when appropriate; manual WAV reimport creates a new candidate/reference, without
claiming a native completion receipt or mutating an accepted source. Cancellation stops after the
current model request and suppresses late candidates.


`project::shape_transfer` owns admission of the narrow `vox.shape-transfer/1` package and atomic
creation of an independent project. It preserves original manifest/material objects, accepted
Shape audio with its historical source pin, and separate unaccepted writing/speech drafts. Replay
never overwrites later edits. Unsupported imported settings remain visible and non-executable;
CLI and desktop share the same use case, with visible draft restoration in the sidebar. See
[Shape transfer](SHAPE_TRANSFER.md) for the file and selection-hash boundary.
