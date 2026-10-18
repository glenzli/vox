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
another build. These UI and documentation changes are local working-tree changes; no new commit,
push, synthesis job or microphone recording was performed for this refinement.
