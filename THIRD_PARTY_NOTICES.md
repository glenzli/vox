# Source provenance and licenses

Vox is MIT licensed. Portable script/audio contracts, parsing/readiness, compilation, PCM assembly,
speech synthesis/control, Job receipt validation and adjacent regression tests were migrated from
[Shape](https://github.com/glenzli/shape), revision
`d47789fc2dd604b14cc2a651e49889513bbd8677`. The PCM24 converter and real Echo fixture driver
come from the same revision. `docs/SHAPE_PROVENANCE.json` records original paths and SHA-256 values.
Native macOS titlebar alignment is adapted from that revision too, with a hidden duplicate title.
Vox adapts names/imports, extracts speech-only policy, removes unrelated owners and uses its own
application identity and persistence.

Shape's MIT permission notice is retained in `docs/licenses/Shape-MIT.txt`. The pinned official
`infer-runtime-client` and artifact-lease crates keep their own source/license identity in Cargo.
No model weights are copied.

Original imagegen artwork and prompts remain in `assets/branding`. Icon Forge (MIT) mechanically
normalizes the app assets. Interface SVGs were authored for Vox; no icon font is required.
Echo remains a separate executable and supported file/protocol boundary. No Echo DSP, Catalog,
project store or linked native engine is copied into Vox.
