# Vox application icon

The current balanced flat mark was created with **built-in imagegen** on 2026-10-18.
`app-icon-source.png` is preserved byte-for-byte from
`exec-955bc06f-21df-46f8-9b4a-c59bdc46bc30.png`.

- Original SHA-256: `cad8a3c90505e5dbc2e72e970e202eb2cd8fdd7dfdb4cbc1b95a47d0b77d8550`
- Design meaning: Five equally slim, flat voice strokes use a compact rhythm and measured height steps. The muted coral center and warm ivory side strokes represent spoken performance without sculptural depth. Vox remains brand preparation only, with no standalone application.
- Previous sources and complete prompts are preserved in the
  [dimensional archive](archive/2026-10-18-dimensional/README.md) and
  [first flat study](archive/2026-10-18-flat-study/README.md).

## Imagegen edit prompt

The archived first flat study was the exact reference for this proportion edit.

```text
Use case: precise-object-edit. Edit target: the provided flat Vox icon.
Redesign the proportions and spacing of this five-bar voice mark. The current center stroke is too towering, the whole group too spread out, and the side strokes feel detached.
Keep exactly five front-facing vertical rounded bars on an opaque pure black square. Keep the four side bars soft warm ivory and the center bar muted coral salmon. Use clean uniform matte solid fills with NO texture, gradient, highlights, shadows, bevels or glow.
Make ALL FIVE bars the same elegant width, about 6.8 percent of canvas width. Set the four gaps to about 3.3 percent of canvas width, so the total group width is about 47 percent. Make the heights progress smoothly: outer bars about 19 percent of canvas height, inner pair about 32 percent, central bar about 43 percent. All bars share one horizontal centerline. The center bar should feel integrated, not a tall separate pillar. Ends are smooth semicircles. Avoid turning the outer strokes into circular dots.
Center the compact mark optically in the square with generous balanced black margins. Its proportions should read as a cohesive voice waveform, calm and confident, with a balanced height-to-width relation at 32 pixels.
No text, letters, borders, extra symbols, outer app tile or transparency. Keep this an opaque black square artwork. Main change is the size hierarchy, spacing and visual balance; preserve the five-stroke identity and restrained ivory/coral palette.
```

## Mechanical outputs

`app-icon.png` is 1024×1024. `app-icon.icns` contains the standard macOS 16, 32, 64, 128,
256, 512 and 1024 pixel representations, including the paired Retina entries. These are brand resources only; there is no Vox application.
Both derive directly from the opaque RGB original, reusing the established native black-background
presentation. No repainting, cropping, filtering, masking or background removal is applied.
No Windows binary or ICO is claimed.

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
