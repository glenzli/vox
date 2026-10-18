# Vox

English · [简体中文](README.md)

**Script-led voice production.**

Vox is an independent Qt desktop application for narration, multi-speaker dialogue and listening
materials. Write scripts, direct voices, synthesize locally, then audition, accept and export takes
while keeping exact script versions and accepted audio. This is a development preview, not a published
release. macOS Apple Silicon is the initial verified platform.

## Workflow

1. Create a `.vox` project, write or import UTF-8 text, and choose a narration, dialogue or listening template.
2. Choose presets or reference voices for narration and individual roles; set language, pace and sound cues.
3. Render a candidate, listen and compare, explicitly accept your preferred take, then export WAV.
   Earlier accepted takes remain available after script revisions.
4. For trimming, fades or mixing, edit a copy in [Echo](https://github.com/glenzli/echo), then return its
   exported WAV to Vox for review and acceptance as a new candidate.

## Features

| Feature | Available now |
| --- | --- |
| Script editor | Syntax highlighting, logical line numbers, wrapping, write/read views, and independent script font/size settings with preview and reset |
| Multi-speaker production | Shape-compatible roles, delivery, scenes, pauses, cues and repeats; repeats replay identical PCM without another model call |
| Voice presets | Nine presets, with automatic or explicit speech language |
| Reference voices | Record a microphone sample or import mono/stereo PCM16 WAV, enter its exact transcript and confirm permitted use; assign to narration or any role |
| Sound library | Eight procedural built-in effects, WAV import, and local Stable Audio 3 sound-effect and short-music generation |
| Versions and materials | Independent projects, autosaved drafts, immutable materials, and explicit script/audio acceptance |
| Echo handoff | Independent editing copies for takes, voice references and sounds, preserving originals with manual WAV return |
| Interface | Compact icon-led tools, a neutral black/white/gray palette, English/Chinese, and light/dark/system themes |

Original reference audio, exact transcripts and permitted-use confirmations stay in the project.
Microphone access is requested only when Record is pressed. The current Base clone route supports
1× pace and neutral delivery. Automated checks use synthetic references and injected audio samples;
real microphone capture requires a suitable device and first-use system permission.

Sound generation supports 1–30 seconds, an explicit seed and English or Chinese descriptions.
Chinese descriptions use local prompt preparation; original and effective prompts and Runtime receipts
are retained. After audition and explicit acceptance, Vox prepares a script-compatible audio copy while
keeping the original WAV for export or Echo editing.

Echo returns enter the candidate list first. PCM24-to-PCM16 conversion requires an explicit choice.
The applications exchange files and retain independent project stores and databases.

## Run locally

Requires Rust 1.96+, CMake 3.28+, Ninja and Qt 6.9+ with Quick, Quick Controls, Multimedia, SVG and
Linguist Tools. Build scripts use the offline Cargo dependency cache; prepare the locked dependencies
before the first build.

```sh
./scripts/build_and_promote_debug.sh
./scripts/run_debug.sh
./scripts/run_debug.sh /absolute/path/MyProject.vox
```

Build products and the current Debug app live in the sibling `.vox-local-build`, outside the checkout.
`VOX_LOCAL_BUILD_ROOT`, `VOX_BUILD_DIR` and `VOX_CARGO_TARGET_DIR` select other external paths.
The app contains `vox-cli`; Qt must still be installed on the development host. Redistributable
packaging, notarization and public release are separate work.

### Connect Infer Runtime

Vox uses its own `vox` App authorization for local-only, zero-cost, no-fallback synthesis.
Issue a separate credential in Infer Console and import it in Vox settings. The operator helper is
documented through:

```sh
python3 scripts/provision_infer.py --help
```

The default credential path is `~/.config/vox/infer-credential`. `--studio` additionally requests
Vox-only local reference voices, sound generation and Chinese prompt preparation. Apply grants while
Console queues are idle. Vox does not use Shape's credential. Offline editing and saved audio remain usable.

## Development and documentation

| Owner | Responsibility |
| --- | --- |
| `crates/vox-core/src/domain` | Portable audio contracts and Shape-compatible script grammar |
| `crates/vox-core/src/assembly.rs` | Bounded script compilation, PCM assembly and validation |
| `crates/vox-core/src/infer_runtime` | Official SDK, Vox identity, local synthesis and progress |
| `crates/vox-core/src/project` | Independent bundles, drafts, immutable materials and acceptance |
| `crates/vox-core/src/handoff.rs` | Echo working copies, explicit return and PCM24 conversion |
| `apps/vox-cli` | Shared JSON operations for the desktop, users and automation |
| `apps/desktop` | Qt/QML authoring, voice direction, audition and review |

See [architecture](docs/ARCHITECTURE.md), [automation](docs/AUTOMATION.md),
[script rules](docs/SPEECH_SCRIPT.md), [validation](docs/VALIDATION.md), [roadmap](ROADMAP.md),
and [icon provenance](assets/branding/README.md).

This migration reuses Shape's speech production code with its provenance and license retained.
Shape's existing projects and speech interface remain intact. Exported text/scripts can be imported;
entire `.shape` projects are not automatically converted. Persistent per-segment re-recording,
voice design from descriptions and video dubbing remain future work.

## License

Vox is available under the [MIT License](LICENSE). Migrated code and dependency attribution are
recorded in [third-party notices](THIRD_PARTY_NOTICES.md) and the [Shape source manifest](docs/SHAPE_PROVENANCE.json).
Model weights are not distributed with the project; their providers determine their licenses.
