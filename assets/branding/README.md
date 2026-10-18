# Vox application icon

Generated with **built-in imagegen** on 2026-10-18. The selected original artwork is
`app-icon-source.png`, preserved byte-for-byte from `exec-b51bdda8-e0db-4923-82f2-8387b80ce92d.png`.

- Original SHA-256: `c340ddaa6f79e94798c7ac82d5b2f76fec81cd30796d7933a0e369b0223044d6`
- Design meaning: Five sculptural voice strokes represent scripts becoming spoken performances. Their ivory and restrained warm edges distinguish voice production from Echo's sound rings. This is brand preparation only; Vox has no standalone application.

## Original generation prompt

```text
Use case: logo-brand. Create one finished macOS application icon for Vox, an app for turning scripts and dialogue into spoken performances.
Square 1024x1024, pure deep black background edge to edge, no drawn rounded-square tile. A single compact centered voice-wave emblem: exactly FIVE upright slim sculptural pill-shaped strokes of different heights, arranged left to right short / medium / tallest / medium-high / short, vertically centered on the same invisible horizontal baseline. They form a calm, immediately recognizable spoken voice pulse, not a chart. The whole emblem occupies about 70% of the canvas width and 66% of its height. Substantial clean gaps, simple silhouette readable at 32px.
Visual family: black-background geometric art with controlled luminous edges, like Shadow's champagne-lit planes and Echo's glowing silver sound traces. The strokes are softly rounded satin ivory metal, mostly silver-white faces with graphite side shadows. Gentle warm copper and soft coral light grazes the inside edges; a small restrained warm halo, no broad background wash. Slightly dimensional sculptural relief, almost frontal. Very fine material grain, clean premium finish, generous pure black negative space.
Do not use concentric circles (those belong to Echo), a microphone, speech bubble, equalizer UI, controls, text, letters, labels, numbers, music notes, colorful neon, sparkle, tile frame, floor, scenery, or watermark. One coherent quiet emblem, no collage.
```

## Mechanical outputs

`app-icon.png` is 1024×1024. `app-icon.icns` contains the standard macOS 16, 32, 64, 128,
256, 512 and 1024 pixel representations, including the paired Retina entries. Vox's containers are brand resources only; there is no Vox application.
Both derive directly from the opaque RGB original: macOS supplies the native outer presentation,
avoiding a second frame around a precomposed transparent tile. No repainting, cropping, filtering,
masking or background removal is applied. No Windows binary or ICO is claimed.

From this asset directory on macOS, reproduce the outputs with:

```sh
sips --resampleHeightWidth 1024 1024 app-icon-source.png --out app-icon.png
ICON_WORK=$(mktemp -d)
mkdir "$ICON_WORK/app-icon.iconset"
for ICON_ENTRY in 16x16:16 16x16@2x:32 32x32:32 32x32@2x:64 128x128:128 128x128@2x:256 256x256:256 256x256@2x:512 512x512:512 512x512@2x:1024; do
    ICON_LABEL=${ICON_ENTRY%:*}
    ICON_SIZE=${ICON_ENTRY#*:}
    sips --resampleHeightWidth "$ICON_SIZE" "$ICON_SIZE" app-icon.png --out "$ICON_WORK/app-icon.iconset/icon_$ICON_LABEL.png"
done
iconutil --convert icns "$ICON_WORK/app-icon.iconset" --output app-icon.icns
```

The temporary iconset stays outside the repository. These commands never regenerate the original.
