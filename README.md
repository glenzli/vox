# Vox

脚本驱动的配音制作。Script-led voice production.

Vox is an independent Qt desktop application for narration, multi-speaker dialogue and listening
materials. This is the first development preview, not a published release.

Write or import a UTF-8 script, choose voices and language, render a take, audition it, then accept
or export it. Accepted audio and exact script versions survive reopening and later rewrites.
Roles, delivery, scenes, pauses, sound cues and repeats reuse Shape's tested production owners.
Repeats replay identical PCM without another model call.

Vox uses its own Infer Runtime authorization for local-only, zero-cost, no-fallback synthesis.
Nine voice presets and automatic or explicit speech language are supported. The interface supports
English and Simplified Chinese, with light, dark and system-following themes and accessible icons
in a neutral black, white and gray palette.

Accepted audio can open in [Echo](https://github.com/glenzli/echo)'s independent editor. Return its
WAV export for review as a new candidate. The handoff pins the source revision and keeps the
original audio; PCM24-to-PCM16 conversion requires an explicit choice.

## Run locally

Requires Rust 1.96+, CMake 3.28+, Ninja and Qt 6.9+ with Quick, Quick Controls, Multimedia, SVG
and Linguist Tools. macOS Apple Silicon is the initial verified target.

```sh
./scripts/build_and_promote_debug.sh
./scripts/run_debug.sh
./scripts/run_debug.sh /absolute/path/MyProject.vox
```

Builds and canonical Debug apps live in the sibling `.vox-local-build`, outside this checkout.
`VOX_LOCAL_BUILD_ROOT`, `VOX_BUILD_DIR` and `VOX_CARGO_TARGET_DIR` select other external paths.
The promoted app contains `vox-cli`. Qt must be installed on the development host; redistributable
and notarized packaging is a separate release task.

Infer Console issues a separate `vox` App credential. `scripts/provision_infer.py --help` describes
the explicit operator helper for the narrow speech grant and `~/.config/vox/infer-credential`.
It never uses Shape's credential. Enable grants through Console when its queues are idle.
Settings can import an already issued Vox credential. Offline editing and saved audio stay usable.

## Source map

| Owner | Responsibility |
| --- | --- |
| `crates/vox-core/src/domain` | Portable audio and Shape-compatible script grammar |
| `crates/vox-core/src/assembly.rs` | Exact bounded script-to-PCM compilation and validation |
| `crates/vox-core/src/infer_runtime` | Official SDK, Vox identity, local synthesis and progress |
| `crates/vox-core/src/project` | Independent bundle, drafts, immutable objects and acceptance |
| `crates/vox-core/src/handoff.rs` | Echo working copies, explicit return and PCM24 conversion |
| `apps/vox-cli` | Shared JSON operations for humans, AI and the desktop |
| `apps/desktop` | Qt/QML authoring, direction, audition and review |

See [architecture](docs/ARCHITECTURE.md), [automation](docs/AUTOMATION.md),
[script rules](docs/SPEECH_SCRIPT.md), [source provenance](docs/SHAPE_PROVENANCE.json),
[third-party notices](THIRD_PARTY_NOTICES.md), [roadmap](ROADMAP.md), and
[normalized icon provenance](assets/branding/README.md).

Shape's existing projects and speech UI remain intact. This migration reuses production code;
it does not rewrite Shape databases or automatically convert entire `.shape` projects. Exported
text/scripts can be imported into Vox. Voice cloning, persistent per-segment re-recording and
video dubbing remain future product decisions.
