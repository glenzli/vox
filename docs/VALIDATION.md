# Initial Vox validation — 2026-10-18

The independent implementation is a local development preview. No Git publication, public
release, Windows build or automatic `.shape` project conversion is claimed.

| Boundary | Evidence |
| --- | --- |
| Portable scripts and PCM | 57 passing Rust tests, including migrated grammar, readiness, exact pauses/repeats, cue bounds, segmentation, retry invalidation and receipt checks |
| Independent project | Tests for concurrent expected-version writes, stale candidates, exact source bytes, immutable audio after rewrites, corrupted objects, interrupted runs and no-clobber export |
| Native app | Packaged startup plus real QML editor-to-worker persistence, live English/Chinese switching, light/dark/system appearance and a queued old-project mutation fence |
| Localization | 179 finished Chinese messages, source-extraction comparison and exact placeholder checks |
| Source gate | Rust formatting, workspace Clippy with warnings denied, native formatting, and tracked/new source whitespace checks |
| Runtime authorization | Separate managed `vox` App credential, nine speech aliases and one exact local TTS route; no cloud egress, fallback or resource administration |
| Real synthesis | Two physical Chinese/English voice Jobs through default SDK Discovery and exact Consumer Core; 8.5 seconds of mono 24 kHz PCM16, one exact replay, explicit acceptance, reopen and verified WAV export |
| Native audition | Real macOS Cocoa application decoded the generated audio, reported 8500 ms and advanced playback while muted |
| Real Echo | Current Echo source built through its managed scratch script; real independent stdio process applied gain, exported PCM24 WAV, replayed an idempotent export and returned to Vox as a candidate |
| Return acceptance | PCM24 return first rejected without conversion consent, then converted explicitly, reviewed, accepted, reopened and exported; original accepted WAV remained byte-identical |
| Disclosure | Independent FFprobe observed `ai_generated` in Echo's returned WAV comment |
| Icon | Icon Forge normalization completed; original source SHA-256 unchanged; PNG alpha, ICNS and ICO formats verified |

The original speech run's Jobs are `audio_769020718dae48c78f195accc88b6269` and
`audio_a83e04d2874c43528f71b5b4c7c692e1`. The local workflow report is
`/private/tmp/vox-live-20261018c/report.json`; its Echo transcript is in
`/private/tmp/vox-live-20261018c/echo-resumed/report.json`. These are local test artifacts, not
public endpoints. The completed recording was reused while repairing the external editor setup;
there was no duplicate synthesis for that retry.

The pre-existing daily Echo Debug app predates its independent editing protocol. A current,
separately built Echo executable is required. Vox rejects that old entry instead of treating
unrecognized flags as a Catalog path. Native capability discovery is bounded and runs in an
isolated working directory before launch. Echo's daily application and library are not upgraded
by this Vox task.

The installed Qt media backend is macOS native. Its offscreen media loader did not advance;
actual audition was therefore checked in a real Cocoa window, rather than treating a headless
decode check as playback proof. This verifies decoding and timing, not subjective voice quality.

Preview images were inspected for layout in light English and dark Chinese. Source changes after
the speech run affected voice-sample presentation, project dispatch fencing and desktop behavior;
the unchanged synthesis owner retained the real Job evidence. Full source checks and packaged
checks were repeated for the final desktop inputs.

## Button contrast and integrated titlebar, 2026-10-18

The `onAccent` theme property returned an invalid QColor in the running QML tree, producing black
text/icons on dark primary buttons. Renaming it to `accentText` restores valid foreground colors.
Buttons retain Qt's normal content rendering, and disabled foregrounds use the readable secondary
color. A rendered-pixel regression check failed before this correction and passes afterward.

The 44 px toolbar now occupies the expanded macOS client area, hides the duplicate native title,
reserves space for native traffic lights, and supports dragging and double-click maximize/restore.
Shape's attributed native alignment owner also reapplies positions after Cocoa's deferred layout.

- Required `build_and_promote_debug.sh` gate passed: 57 Rust tests, formatting/Clippy, 179 complete
  Chinese messages, native build, and all three packaged checks.
- `--smoke-appearance` passed with the software renderer and the final packaged Cocoa application.
  Eight stages cover light/dark, active/inactive windows, disabled actions, maximize and restore/
  resize. Actual text and icon pixels satisfy 4.5:1 contrast for enabled primary actions and 3:1
  for disabled actions. Native checks confirm the full-size client area, hidden duplicate title,
  visible traffic lights centered in the toolbar, and no overlap with app content.
- The validated, signed artifact was promoted without rebuilding. Native executable SHA-256:
  `0e293b98175d96b2d60f04395c77ce6566ecfc30a8b4a2522bd73b20d33fb20d`.
  Speech worker identity remains unchanged.
- The previous saved app was closed normally and the updated app reopened the user's recent
  project. UI inspection shows the original 240-character draft and both accepted recordings;
  `project.json` remains byte-identical. No synthesis or Echo edit was repeated for this UI fix.

Native images are in `/private/tmp/vox-ui-native-final-20261018`; the current local application is
`/Users/g4i/lab/.vox-local-build/current-debug/Vox.app`. No commit or push was performed.

## Script highlighting and logical line numbers, 2026-10-18

The plain-text editor now distinguishes directive keys (muted blue), role/cue identities (warm
brown), numeric controls (weight), headings (weight) and authored notes (gray italic). These are
lexical display styles; Rust still decides whether a script is executable. Light/dark palettes
switch live. Logical source line numbers follow wrapped blocks, mark the current line and share
the text viewport's top/bottom clipping. The icon toggle defaults on and persists independently.

The required build gate passed 57 Rust tests, formatting/Clippy, 180 complete Chinese messages,
the native build, and four packaged checks. The editor check additionally covers Chinese/English
tokens, Unicode and escaped brackets, wrapped lines, actual number pixels after scrolling, plain
mode, selection without gutter text, undo, exact saving, and switching from an empty project back
to an existing script. The final signed Cocoa application passed the same check. Native layout
snapshots live in `/private/tmp/vox-editor-native-fixed-20261018`.

Initial live reopening exposed a formatting-only TextEdit notification being treated as an empty
edit. That build was closed and its default preview pointer rolled back. The pre-update snapshot
restored the user's exact 239-character draft through the expected-version project transaction;
only the manifest version advanced (12 to 14), with all other fields including accepted audio
unchanged. Explicit guarded loading and comparison with the previously observed editor text now
prevent formatting refreshes from becoming edits. The new reopening regression covers this path.
Software and native cold-start replay of the actual affected project copy leave its manifest
byte-identical; evidence is `/private/tmp/vox-editor-cold-open-20261018/report.json`.

The corrected preview was reopened on the user's project. Accessibility/UI inspection confirms
the full draft, both accepted recordings, and the line-number toggle switching off and back on.
The live manifest remains identical to the restored snapshot after opening and toggling.
The promoted executable SHA-256 is
`57257f8a020ed88ee90b1eedda0e92673680dd3774ea8093c10af7c34b7965bd`;
the speech worker is unchanged. No new speech/Echo operation, commit or push was performed.

## Reference voices, editor fonts and sound library — 2026-10-18

- Initial independent app/editor implementation is locally committed as `2f411d2`. Official Rust
  clone transport is separately committed in Infer Runtime as `8dc48ce` and pinned by Vox. No push
  or public release is implied.
- The source gate `/private/tmp/vox-studio-final-gate.log` passed 68 Rust tests, fmt/Clippy, 284
  complete Chinese messages and all five packaged tests. New tests cover exact reference words,
  consent, foreign-project binding rejection, immutable source revisions, candidate adoption,
  explicit downmix/resampling, owner-only files, pre-dispatch cancellation and inactive role/cue
  inputs when switching to plain narration.
- Native capture checks inject bounded float/stereo PCM through the actual recorder conversion and
  WAV publication path, import it through the packaged worker, and prove cancellation removes the
  disposable preview. They enumerate the native microphone but never capture a person or accept an
  OS microphone permission. Real microphone recording still requires the user’s first permission
  and a suitable device; acoustic reference quality is not established by these checks.
- Script editor tests apply/persist a different font and size, verify actual gutter geometry after
  wrapping/scrolling, preserve selection/undo/source bytes, and cold-reopen the project without a
  false draft mutation. Signed Cocoa checks passed for the editor and the new library/capture form.
- The final UI refinement replaces dynamic translated-name lookups with notification-backed voice,
  language and built-in catalogs. Focused packaged appearance/studio checks and signed native studio
  checks passed after that change; unchanged Rust/generation evidence was reused. Native diagnostics
  included macOS IMK/sandbox-extension messages, with no QML binding/resource errors or failing checks.
- `/private/tmp/vox-studio-live-20261018-c/report.json` proves real `app_id=vox` clone preview,
  Stable Audio 3 Small SFX and Small Music, Chinese local prompt preparation, explicit sound adoption,
  mixed preset/reference roles with an adopted cue, accepted WAV export and byte-identical reopen.
  All three generation receipts are local, no-fallback and zero-cost. The reference is synthetic
  speech generated for this isolated test, not a human recording.
- `/private/tmp/vox-studio-live-20261018-c/final-consumer.json` proves the final worker again rendered
  a mixed script, accepted/exported it and reopened it. It binds worker hash and per-segment receipts.
- Vox’s existing credential was retained. Console-owned apply happened with every provider queue
  idle; config read-back excludes only Vox and proves all other apps/runtime settings unchanged.
  The own grant adds local clone, sound and text preparation, without cloud or observer permission.
- The pre-existing user project stayed byte-identical to `/private/tmp/vox-studio-user-before.json`
  (239 characters, two accepted takes, version 14); no sample script or accepted audio was replaced.

## Compact desktop controls and bilingual documentation — 2026-10-18

Shared controls now use 28 px buttons and fields, a single 12 px dropdown chevron, a 3 px pace
track with a 12 px handle, and themed, scrollable option lists. Script tools occupy one row;
role/cue bindings occupy compact rows. Take cards are 182 × 60 px and the audio area is 172 px
high instead of 236 px. Script font preferences and source semantics are unchanged.

- Formatting/Clippy, all 68 Rust tests and 284 complete Chinese messages passed in
  `/private/tmp/vox-density-gate.log`. The first appearance run caught a faint disabled waveform
  icon after it was reduced to 14 px; retaining a 16 px icon corrected it without relaxing checks.
- The final Qt build passed all five packaged checks in `/private/tmp/vox-density-ui-recheck.log`.
  Unchanged Rust evidence was reused after that QML-only adjustment.
- The promoted, signed Cocoa app passed appearance, editor and studio checks in
  `/private/tmp/vox-density-native.log`, including light/dark, activation, disabled actions,
  maximize/restore, source preservation, fonts, and material forms. Images are retained under
  `/private/tmp/vox-density-native-20261018`. Native macOS diagnostics were present; no QML
  binding/resource error or failing check was observed.
- Live UI inspection opened the existing project, expanded the voice menu, navigated with the
  keyboard and cancelled without changing its voice. The manifest remained byte-identical to
  `/private/tmp/vox-density-user-before.json`: version 14, 239 characters and two accepted takes.
- Chinese `README.md` and English `README.en.md` have reciprocal language links and verified
  relative documentation links. Both explicitly link the existing full MIT `LICENSE` and retain
  third-party attribution and model-license boundaries.

Artifact/input hashes and the validation paths are recorded in
`/private/tmp/vox-density-evidence-20261018/report.json`. The validated app was promoted without
another build. At that checkpoint these UI/documentation changes were uncommitted; they were later
committed as `e2cc820`. No push, synthesis job or microphone recording was performed for that refinement.

## Grouped script reading — 2026-10-18

Reading now presents role/language/delivery metadata with selectable speech, compact pause/sound
rows, and a bordered repeat group showing count, gap and cue. Parser-owned authored-note markers
separate actual notes from historical role/language control events. Rejected source lines remain
visible with their issue and an explicit source-line edit action. The projection keeps exact input
text, suppressing old-source cards while a changed draft is waiting for analysis. None of these
display fields are persisted or change executable events, digests or acceptance.

- `/private/tmp/vox-reading-gate.log` passed Rust formatting, Clippy and 70 Rust tests, including
  unchanged execution-plan bytes/digests, repeat grouping, authored notes and rejected Unicode/CRLF
  source. That evidence was reused for later QML/test-only corrections.
- The final native build and all six packaged checks passed in `/private/tmp/vox-reading-ui-final.log`.
  The new reading check covers card/group geometry, rejected text, literal plain narration, live
  Chinese/English and light/dark switching, correct source-line focus, stale-preview suppression
  and byte-identical project preservation. Translation extraction verifies all 307 Chinese messages.
- Translated combo models previously reset their displayed format/voice index. Shared controlled
  selections now restore the document/preferences value. The reading check verifies both format
  and a non-default voice across language changes.
- The signed Cocoa app passed reading and appearance checks in `/private/tmp/vox-reading-native.log`.
  Light/dark images are under `/private/tmp/vox-reading-native-20261018`. Native macOS diagnostics
  were present, with no QML binding/resource errors or failing checks.
- The same validated app was promoted to the normal local preview without rebuilding. Live pointer
  interaction on the existing project opened the reading page, followed the line-7 edit action,
  and returned to reading. Its manifest stays byte-identical to
  `/private/tmp/vox-reading-user-before.json` (239 characters, version 14, two accepted takes).
  The pre-existing missing-role issue remains explicit; the task did not rewrite the user's script.

Source/artifact/project identities are recorded in `/private/tmp/vox-reading-evidence-20261018/report.json`.
The compact UI/documentation commit is `e2cc820`; this reading change is committed separately.
No push, synthesis job, Echo edit or microphone capture was required for this refinement.

## Published SDK and clean checkout build — 2026-10-18

Infer Runtime SDK commit `8dc48ce137428e6c11d5d50a3bf94c090be7f3b9` is now published on
`origin/main`; both the remote ref and GitHub commit API return that exact revision. The existing
SDK commit needed no further source changes. Its clean archived tree passed all 39 SDK tests;
the three SAM prompt regression tests for its parent commit also passed. The 37 pre-existing
uncommitted Infer media/model-catalog files remain byte-identical and were not included.

- Vox was freshly cloned from `https://github.com/glenzli/vox.git` at `2f009e4`, with no Git object
  alternates and global/system Git configuration disabled. Cargo home, Cargo target and desktop
  build directories were empty before dependency retrieval.
- `cargo fetch --locked` downloaded the pinned SDK from its HTTPS remote and the locked registry
  dependencies. The fetched SDK checkout resolves to the exact published revision above; no local
  Infer checkout or pre-populated Cargo cache was used.
- With that isolated Cargo home and `CARGO_NET_OFFLINE=true`,
  `scripts/build_and_promote_debug.sh` passed formatting, Clippy, all 70 Rust tests, all six packaged
  desktop checks and 307 complete Chinese translations. The fresh source checkout stayed clean.
- The script promoted only to an isolated temporary application directory. Signature verification
  and a signed Cocoa `--smoke-exit` startup passed. macOS emitted a sandbox-extension diagnostic;
  no QML load/binding error or failing check was observed. The normal preview application and
  the user's existing 239-character project with two accepted takes were preserved.

Evidence, source/artifact identities and logs are in
`/private/tmp/vox-sdk-clean-20261018-c_1ybir5/report.json`. This verifies a fresh checkout and empty
dependency/build caches on the existing macOS ARM64 host (Rust/Cargo 1.96.0, CMake 4.4.0,
Ninja 1.13.2 and Qt 6.11.2), not a newly provisioned operating system or a new Runtime synthesis
session. The bilingual README now documents the initial online fetch before the offline build.
Only publication documentation changed after this build; production source and the lockfile match
the validated checkout.

## Desktop use review and asynchronous state fixes — 2026-10-22

Three parallel GPT-6.1 Sol/max reviews covered project durability, desktop/material workflows and
script editing/reading. The integrated changes retain draft edits across asynchronous replies, pin
take-operation targets, fence project transitions, cancel queued/preparing generation, and wait for
pending imports before closing. Executor initialization errors now finalize failed render journals.
Editor project identity clears cross-project undo history; same-cursor navigation scrolls correctly;
long reading metadata wraps without hiding the edit action. PCM16 reference capture retains boundary
samples and enforces its 60-second buffer cap. No project format or grammar change is required.

- `scripts/build_and_promote_debug.sh` passed formatting, Clippy, all **85 Rust tests** and all
  **six packaged desktop checks**. The final catalog contains **325 complete Chinese messages**.
  Later QML corrections passed the full desktop suite, followed by focused project/appearance/reading
  checks for the final recording-card accessibility action; unchanged Rust evidence was reused.
- Signed Cocoa appearance, editor, studio and reading checks passed. A native interaction exposed
  the close-confirmation race when generation completes before confirmation; its regression now
  passes both packaged and Cocoa checks. Native focus checks ran with only their own test window.
  macOS emitted sandbox-extension/IMK diagnostics without failing the final checks.
- Native pointer/accessibility use on an isolated project exercised editing, role assignment,
  reading, synthesis, playback, acceptance, WAV export and built-in sound preview. Missing role
  voices are now named before rendering; dirty text waits for current analysis. Sound preview clears
  take selection so take-specific export/Echo actions do not target unrelated audio.
- The final worker produced a **5.48 s, 24 kHz, mono PCM16** dialogue through Vox's own Runtime
  authorization. Its segment Jobs are `audio_0237132e298148b385570d60132ef2a1` and
  `audio_52198425b9eb47f8bfe18ed3bd2a0019`. Native playback reached the end, the take was accepted,
  and exported bytes match the accepted object. Export SHA-256 is
  `5a3a536986ba0aa6b027a3e788d42443283733a8f367449179e02e4a61f25056`.
- Synthetic recorder fixtures validate conversion/import/cancellation without capturing a person.
  Real microphone permission, acoustic quality and device hot-plug were not exercised. Echo and
  AI sound generation were not repeated in this review; their earlier live receipts remain scoped
  to the previous runs above.
- The existing user project remains byte-identical: version 14, 239 characters and two accepted
  takes. The validated app is promoted to the normal `current-debug/Vox.app` pointer; this review
  does not commit or push its source changes.

Logs, snapshots, source/artifact identities and the final live report are under
`/private/tmp/vox-review-20261022-c_4c7028/`. The initial source gate is `gate.log`, final desktop
checks are `ui-final.log` and `audio-ui.log`, native checks are in `native/`, and consolidated
evidence is `report.json`. All generated files and test projects remain outside the checkout.

## Segment production and sound workspaces — 2026-10-22

Three parallel GPT-6.1 Sol/max tasks implemented persistent segment alternatives and assembly,
pronunciation rules, measured subtitles, and independent SFX, ambience and short instrumental
music workspaces. Full songs are outside the product scope. Voice design, performance conversion
and timecoded/video dubbing remain roadmap items, not implemented capabilities.

- The required `scripts/build_and_promote_debug.sh` gate passed formatting, Clippy and all
  **108 Rust tests**. Two new desktop checks exposed a QML delegate-scope error and a test fixture
  digest-decoding error; both were corrected. The final integrated desktop build passed all
  **eight packaged checks** and contains **446 complete Chinese messages**. A subsequent change
  keeps the stop control visible during segment generation; the production and sound checks
  passed again. Unchanged Rust evidence was reused.
- Signed native Cocoa production and sound-workspace checks passed, covering pronunciation
  persistence, source navigation, cancelled preparation, concurrent edits, audition fences,
  category and language changes, explicit acceptance, reopening and exact original export.
  English/dark and Chinese/light screenshots are retained with the test output.
- Real generation through Vox's own authenticated local Runtime connection produced two speech
  segments, regenerated just the first, retained the second selection, and assembled a **6.07 s**
  take with pauses, a repeated passage and its cue. Reselecting the earlier first segment restored
  the original take's exact bytes. SRT/VTT exports contain the authored words, use measured audio
  durations and include the repeated passage twice; pronunciation substitution only changes
  synthesis input. The WAV export SHA-256 is
  `cc0e97d840d0748b23c5025bc028534d3e7b48446b8c0f43e7a615081e68b604`.
- The same real run generated two SFX variants, one ambience and one short instrumental music
  candidate through the local Stable Audio routes. Distinct seeds, explicit acceptance, original
  WAV export, source preservation and byte-identical reopening passed. These checks establish
  generation and data flow; they do not establish perceptual quality or seamless looping.
- Native use on the isolated live project exercised segment regeneration, audition, acceptance,
  assembly, sound-category navigation and loop audition. It exposed two usability defects:
  accepting a segment renumbered alternatives, and entering a sound workspace retained a speech
  playback target. Stable alternative ordering and clearing the playback context corrected both.
- Automatic Runtime discovery intermittently reported unavailable during the run. The local health
  and explicitly addressed authenticated probe succeeded, and a subsequent ordinary discovery
  probe and the app's Check connection action both recovered. No fixed-port fallback was added;
  this observation is retained as an environment limitation without claiming a diagnosed cause.
- The original user project remains byte-identical at version 14, with 239 characters and two
  accepted takes. The updated preview is available at the normal `current-debug/Vox.app` pointer.
  Echo editing and microphone capture were not repeated in this expansion; their earlier evidence
  stays scoped to the earlier runs. No source commit or push is part of this validation.

Evidence is under `/private/tmp/vox-expand-20261022/`: `gate.log`, `context-final.log`,
`controls-final.log`, `native-production-final/`, `native-sound-final/`, `live-2/report.json` and
the consolidated `report.json`. Media, fixtures and build products remain outside the checkout.

## Voice design and long-form chapter production — 2026-10-23

Three parallel GPT-6.1 Sol/max tasks implemented synthetic voice design, durable chapter production
and the desktop workflow. The integration reuses the pinned Infer Runtime SDK and the existing
segment renderer, voice library and independent Echo protocol. Full songs remain outside scope.

- The required `scripts/build_and_promote_debug.sh` workflow passed formatting, strict Clippy and
  **139 Rust tests**. Desktop integration subsequently corrected QML syntax, a chapter-reorder
  `QJsonValueRef` aliasing bug and translated status refresh; unchanged Rust evidence was reused.
  The final integrated desktop suite passed **nine packaged checks**, including chapter CRUD,
  save-before-switch, missing-child recovery, design cancellation and project callback fences.
  English/dark, Chinese/light and 1080 × 740 minimum-window layouts were inspected.
  The final status-only correction passed the focused chapter/design check again, including
  all-reused, mixed, cancelled and failed outcomes. All **550 Chinese messages** are complete.
- Real generation through Vox's own local Runtime authorization produced two designed voice
  samples, saved a named synthetic reference and reused it for two chapters through
  `speech.clone_voice`. The design Jobs are `audio_0307ca270b94403eb51d102c21fd2fc7` and
  `audio_1933026006c646edbeedc3c664aa5a47`. The original model receipts, exact sample text and
  designed-voice provenance remain in the project and its owned chapter packages.
- Stopping the real batch after its first completed segment preserved that segment. Continuing
  reused it and completed both chapters without automatically accepting their audio. Explicit
  acceptance followed by a repeated batch skipped both chapters. Changing one sentence generated
  **one new segment**, reused the unchanged segment and left the other chapter manifest
  byte-identical. Accepted chapter exports included measured SRT/VTT and a relative-path manifest.
- A real independent Echo process applied gain and exported PCM24 WAV with an idempotent replay.
  Explicit conversion returned the edited audio as a candidate; it became current only after
  chapter acceptance. The edited chapter exports its WAV and a reason that subtitles are
  unavailable, while the other chapter retains its measured captions. No stale timing was reused.
- Native Cocoa interaction generated another **5.28 s Chinese design sample**, played it to the
  end and saved a named synthetic voice. Its Job is `audio_9aec5ec1936a4fd3ad22696ff8d80920`.
  The chapter reader played the accepted Echo result to the end. Running the desktop batch reused
  both chapters without changing either child manifest; the native export dialog produced both
  WAVs with the correct caption availability. This exposed an inaccurate all-reused completion
  message, which was corrected independently of the unchanged generation/export owners.
- Runtime configuration changed only Vox's existing grants, adding the local design route and
  retaining its credential. The Console restarted the idle daemon; other applications were
  unchanged. Startup took about five minutes during an existing audio-worker dependency probe.
  No Infer Runtime source, SDK pin or Echo daily application update was required.

The real workflow report is `/private/tmp/vox-longform-live-20261023/report.json`, with cancellation,
changed-segment, export and Echo evidence beside it. Its worker SHA-256 is
`ef40c5feb7cd6d530435e584b3c2995cef916abee48ed5751b24cd40d5bc8a61`, identical to the packaged worker
before promotion signing. Signing changes the signature trailer and its Mach-O size fields;
comparison of all remaining executable bytes confirms that the promoted app and worker retain the
validated payload. Those identities are in the native report's `signature-comparison.json`.
These runs validate execution and persistence; they do not rate perceptual voice similarity or
record a human microphone. Generated media and fixtures remain outside the checkout.
Native interaction and export evidence is in `/private/tmp/vox-native-longform-20261023/report.json`;
the original user project remains byte-identical. This phase does not commit or push source changes.
The final signed preview is `/Users/g4i/lab/.vox-local-build/current-debug/Vox.app`.

## Subtitle realignment and compiled chapter delivery — 2026-10-23

Three parallel GPT-6.1 Sol/max tasks implemented audio-pinned caption review, chapter compilation
and their desktop workflows. Integration reuses the existing pinned Runtime SDK and independent
Echo editor; no Runtime source or SDK publication was needed.

- The required `scripts/build_and_promote_debug.sh` workflow passed formatting, strict Clippy and
  **161 Rust tests**. Initial desktop checks found a read-only QML layout assignment; the focused
  delivery check then exposed a compilation dialog constrained by its sidebar parent. Both were
  corrected, along with translated completion status. The final incremental build passed all
  **ten packaged checks**; unchanged Rust evidence was reused. All **645 Chinese messages** are
  complete. Chinese/light and English/dark delivery panels were inspected.
- The final signed worker passed `scripts/test_delivery_live.py` against a new isolated copy of
  the two-chapter fixture. A real Echo process edited the accepted audio and returned PCM24 WAV;
  explicit PCM16 conversion and acceptance preceded local forced alignment. Job
  `audio_d0bd50ed7344419e9c884ae2d28e4577` ran through Vox's own `audio.align` authorization on
  `mlx_qwen3_forced_aligner_0_6b`. Candidate preview, explicit subtitle acceptance, SRT/VTT export
  and chapter-package export succeeded. Export before subtitle acceptance was rejected.
- Compilation correctly rejected preservation of mixed 48 kHz stereo and 24 kHz mono inputs.
  Explicit normalization produced mono 24 kHz PCM16 with a 350 ms chapter gap, four merged cues,
  frame-based chapter markers and pinned source/audio records. The resulting WAV has **230161
  frames / 9590 ms**. Existing output was not overwritten; a changed chapter invalidated the
  reviewed fingerprint. Reopening preserved the accepted alignment, and the input fixture and
  untouched chapter remained byte-identical.
- Native Cocoa use of the final signed app played Chinese audio through to its end, submitted
  the confirmed mixed Chinese/Latin transcript, reviewed and accepted a new alignment, and
  exported SRT through the native save dialog. Job `audio_5cf1953c6c1a4b42b09c2e7f8f072dbc`
  returned two cues with the original punctuation. The native chapter dialog then exercised the
  mixed-format warning, explicit conversion, 350 ms gap, preflight and complete package export.
  The exported frame count, captions and markers match the reviewed settings.
- Provisioning added only the alignment route to Vox's existing Runtime grants and retained its
  credential. Other applications were unchanged; the idle local daemon was restarted through
  Console. User projects were not used for testing. The original welcome project remains
  byte-identical, and all generated media, fixtures, screenshots and builds are outside the checkout.

The signed app is `/Users/g4i/lab/.vox-local-build/releases/20261023-160404-21007/Vox.app`, also
available through `current-debug/Vox.app`. Its worker SHA-256 is
`06bbe60a4cab7f76a1d878e1dee9f3d2b2a3f07a9595db32884e16a04cd31cef`.
Final real Runtime/Echo evidence is `/private/tmp/vox-delivery-final-live-20261023/report.json`;
native interaction, subtitle and compiled package evidence is under
`/private/tmp/vox-native-delivery-20261023/`. The consolidated validation and artifact identities
are `/private/tmp/vox-delivery-validation-20261023.json`.
Alignment times are model estimates for review, not an independent verification of spoken words.
This phase leaves source changes uncommitted and does not push.

## Offline subtitle refinement — 2026-10-23

The preceding alignment/compilation work was committed locally as `c4cba86` before this phase;
it was not pushed. Three GPT-6.1 Sol/max tasks then implemented cue correction, bounded audition
and the compact editor. New changes remain local and uncommitted.

- The required build workflow passed formatting, strict Clippy and **172 Rust tests**. Coverage
  includes immutable parent snapshots after candidate discard, legacy alignment compatibility,
  bounded plain SRT/WebVTT import, literal Unicode/escaping, rounded audio duration and accepted
  manual/imported revisions in chapter and compilation exports. All **714 Chinese messages**
  are complete.
- Packaged subtitle interaction covers text/timing edits, explicit split boundaries, merging,
  adding/removing cues, pointer-driven candidate save and acceptance, stale import callbacks,
  and retaining drafts across close, audio/candidate changes and project transitions. It exposed
  an empty-track lookup that incorrectly enabled export and a nested confirmation layout loop;
  both were corrected. English/light and Chinese/dark inspectors fit a 1080 × 740 window.
- `scripts/test_subtitle_refinement.py` passed against an isolated copy of the real Echo-edited,
  aligned two-chapter fixture. Manual correction, nested candidate discard, exact SRT/VTT
  round-trips, explicit language metadata, acceptance, chapter export and compilation succeeded.
  Stale versions and invalid cue bounds were rejected without changing the manifest. The original
  alignment remained selectable; reopening preserved the selected import. Source/audio records,
  Runtime journals, the other chapter and the source fixture remained unchanged.
- Native Cocoa use exercised text/time correction, split/merge, Keep editing, candidate save,
  adoption, the actual file dialogs for SRT export/import, and reopening. The subtitle popup owns
  its Save/Escape shortcuts through its visible content item; actual Cmd+S saved a correction
  candidate and Escape closed the inspector. Selecting the second cue played from 2.080 seconds
  and stopped automatically at its end. The native report and screenshots are under
  `/private/tmp/vox-native-refinement-20261023`.
- Real macOS WAV playback passed delayed initialization, seek, cue-end stop and return to ordinary
  playback. Native startup can reset an optimistic seek to zero: the range controller now primes
  silently, reseeks on the initialized clock, then restores the original volume. Cancellation,
  errors, source changes and destruction restore volume; mute is preserved. The deterministic
  checks additionally cover stale queued starts, short/rounded durations and request isolation.
  Offscreen media checks do not substitute for the native result in
  `/private/tmp/vox-refine-final-native-playback-20261023.log`.
- All **11 packaged desktop checks** passed: ten passed in the complete run, and the final
  subtitle interaction recheck closed the remaining failure. The same subtitle workflow passed
  under Cocoa after the fixture used public key events and explicitly activated its window.
  Unchanged core and desktop checks were reused; subsequent changes affected only the key-event
  fixture. Logs are `/private/tmp/vox-refine-final-ctest-20261023.log`,
  `/private/tmp/vox-refine-delivery-focus-final-20261023.log` and
  `/private/tmp/vox-refine-delivery-cocoa-focus-20261023.log`.

The offline workflow evidence is `/private/tmp/vox-refinement-flow-20261023/report.json`.
This phase reuses the preceding real Runtime/Echo fixture and does not dispatch new inference,
change Runtime authorization or modify the user's welcome project.

The signed preview is `/Users/g4i/lab/.vox-local-build/releases/20261023-205752-52913/Vox.app`,
also available through `current-debug/Vox.app`. The app SHA-256 is
`cdd5c7487e037ee9324194424be7abf254f040fb1319f92c292fd79235fee53f` and worker SHA-256 is
`180ca319e95755b988aa736caddd842f6b74be96119c713fa628dc2c1b0fc0f9`.
Code-signing verification passed. Excluding the signature trailer and associated Mach-O size
fields, both payloads match the validated build; the worker remains identical to the offline
workflow input before signing. Identities are recorded in
`/private/tmp/vox-native-refinement-20261023/signature-comparison.json`.
The promoted app was opened on the isolated fixture with the saved imported subtitles visible.
New source changes remain uncommitted and no push was performed.
