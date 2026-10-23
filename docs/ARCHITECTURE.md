# Vox ownership and durability

CLI and native desktop call the same `vox-core` use cases. QML owns interaction and appearance,
never project commit authority. `WorkerClient` owns correlated process requests and cancellation;
`DesktopBackend` fences responses by project and draft identity. A separate probe worker keeps
an unavailable Runtime from blocking local project work.
Draft replies retain their dispatch-time edit identity; take operations pin the selected ID.
Project transitions fence editing until completion, while short material operations keep typing
available. Pending requests include save continuations so closing cannot destroy a temporary
reference before import. Cancellation covers queued generation and its save/adopt preparation.

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
`ScriptDocumentState` clears undo/redo on project identity changes even when both projects contain
identical text. Line navigation scrolls the requested block into view even if the cursor is already
there. Reading metadata wraps within the card without displacing its source-line action.

`domain::speech_script::reading` projects the authoritative parser plan and authored-note markers
into display-only speech cards and repeat groups. Rejected source lines remain visible with their
issues; execution events, source digests and persisted documents are unchanged. Each projection
retains its exact input text so `ScriptReader` suppresses stale previews while saving. `ReadingEntry`
renders selectable literal text and metadata, and routes explicit line navigation to `ScriptEditor`.
The shared combo restores document/preference selections after translated models are replaced.

`domain::speech_script` migrates Shape's grammar and readiness tests. `assembly` owns complete
control-to-action compilation, local cues, silence, exact replay and PCM validation. The pinned
official Infer SDK owns Discovery, contracts, transport and owner-only credential loading.
The desktop uses parser readiness to name missing role/cue choices before enabling generation;
dirty text waits for fresh analysis. The backend independently rechecks readiness before adopting
a source revision, so keyboard or direct calls cannot journal an unready script.
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
Executor initialization errors finalize the run as failed through the same journal path as synthesis
errors; they do not leave a running record behind.

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
claiming a native completion receipt or mutating an accepted source. Sound-generation cancellation
stops after the current model request, retains validated completed candidates and prevents the
next variation from starting.


`project::shape_transfer` owns admission of the narrow `vox.shape-transfer/1` package and atomic
creation of an independent project. It preserves original manifest/material objects, accepted
Shape audio with its historical source pin, and separate unaccepted writing/speech drafts. Replay
never overwrites later edits. Unsupported imported settings remain visible and non-executable;
CLI and desktop share the same use case, with visible draft restoration in the sidebar. See
[Shape transfer](SHAPE_TRANSFER.md) for the file and selection-hash boundary.

## Persistent segment production

`project::segments` owns compilation, candidates, explicit selection, assembly and measured timing
receipts consumed by caption export. Definitions bind authored text, effective spoken text, voice, language, delivery and reference
content identities. Matching selections survive unrelated source edits; changed inputs need a new
segment take. Source revisions and accepted whole audio remain immutable. Re-selecting an older
accepted segment changes only the selection for the next assembly.

Assembly reuses the existing compiler for cues, sample-accurate pauses and repeat replay. It validates
each selected segment and builds a new whole-take candidate with selection and frame-timing receipts.
Normal audio acceptance revalidates that receipt and exact PCM. SRT/VTT export follows these measured
speech ranges and original source text; it makes no word-alignment or target-video-duration claim.

`pronunciation` owns bounded, case-sensitive literal substitutions. Replacements are not recursively
matched. Longer text wins; more specific language/role rules override global rules. Document and
operation defaults omit empty rule arrays to preserve legacy digests. Synthesis receipts distinguish
authored input identity from effective spoken input.

The CLI snapshots the segment plan from the same Manifest it returns. `desktop_production` keeps
prepare/adopt/render/accept lifecycles together, including cancellation before dispatch, captured
project and edit identities, and pinned subtitle export targets. Main selects voice or sound
workspaces; AudioPanel remains the single playback owner.

## Sound workspaces

`project::materials` owns bounded batches with category, batch, variation and seed metadata. Each
validated result is saved as an independent candidate before the next variation starts. Cancellation
or later failure retains completed candidates and returns an explicit batch outcome. SFX and ambience
use the local sound model; short music requests instrumental output from the local music model. The
original prompt and submitted prompt remain inspectable. Loop audition repeats playback, with no
claim of seamless synthesis. Original-quality audio and script-compatible accepted PCM stay distinct.

Complete songs are outside Vox's product scope. Performance-driven conversion and timecoded/video-led
dubbing require their own input, application-grant and output contracts.

## Voice design and chapter production

`infer_runtime::voice_design` uses the pinned SDK's existing speech request with the independent
`speech.design_voice` intent and Vox's own local-only grant. It validates the successful Job,
physical deployment, synthetic PCM and output digest. `project::voice_design` journals each bounded
sample batch and persists completed candidates before reporting progress. Explicit acceptance saves
a named synthetic reference, retaining the exact sample transcript and design provenance. Subsequent
speech uses the existing reference-clone route. Generated references have synthetic acceptance
records, distinct from permitted-use confirmations for recorded or imported human references.

`project::chapters` owns ordered chapter identities and batch journals. A chapter is a derived,
project-owned child bundle under `chapters/<id>.vox`; both parent and child identifiers are checked
before access. It reuses the existing Project, segment, acceptance and Echo use cases. Archiving a
chapter retains its immutable source and audio history. Shared settings retain each chapter's text
and format, and copy required immutable voice/cue materials. Publishing one chapter's settings to
the collection is explicit.

Batch work pins collection settings, adopts current chapter drafts and reuses exact matching segment
candidates. Assembly validates selected candidate bytes without advancing segment or whole-audio
acceptance. Stop and failure keep completed work. Lost producer journals become
`interrupted_unknown`; retry is explicit, and completed outputs are considered before new dispatch.
Export includes only current accepted chapter audio and source metadata, in collection order.
Measured captions are included when the accepted audio has valid assembly timings; arbitrary Echo
edits do not inherit obsolete timings. Existing source-pinned Echo tickets bind the child project.

The desktop keeps collection identity separately from the currently edited child. Existing script,
voice, segment and audio controls operate on that child; collection actions retain their captured
parent and selected-chapter identities. Dedicated voice-design and long-form adapters share the
worker's single foreground task and cancellation boundary.

## Subtitle review and compiled delivery

`project::captions` owns audio-pinned subtitle revisions, selections and interrupted-run recovery.
Its measured track projection reuses assembly receipts; `infer_runtime::captions` sends exact
confirmed words to the existing local `audio.align` SDK operation. It checks bounded times and
transcript coverage and retains the canonical response as an immutable object. Job metadata proves
App, intent, physical deployment and local/no-fallback policy; input/output hashes identify consumed
and retained bytes, rather than claiming a payload signature that the Job API does not expose.
Generation produces a candidate and explicit acceptance selects it for one immutable audio revision.
An Echo return has a distinct audio identity and cannot inherit the old selection. Unchanged measured
tracks retain authored text; realignment retains the confirmed spoken transcript and estimates timing.

`project::chapters::compilation` owns export-only joining and its preflight fingerprint. It locks the
parent and selected children, validates current source/settings and accepted audio, and binds selected
order, captions and output options before writing. Preserve mode requires one PCM format; explicit
normalization produces mono 24 kHz PCM16 without replacing any stored take. Silent gaps and chapter
positions use output frames; merged subtitle timestamps derive from those positions. Publication is
an atomic no-clobber directory containing audio, source and chapter/provenance records. Mixing and
timeline editing remain Echo responsibilities.

`DesktopDelivery` owns correlated preview, alignment and compilation requests. Audio/project changes
invalidate open reviews and pending callbacks. User transcript edits remain separate from refreshed
project snapshots; adopting timing and exporting a reviewed compilation are explicit actions.
