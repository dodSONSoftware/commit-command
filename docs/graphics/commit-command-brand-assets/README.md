# Commit Command — Branding Assets

## Files

- `commit-command-banner-light-transparent.svg`: transparent SVG with dark lettering for light backgrounds.
- `commit-command-banner-dark-transparent.svg`: transparent SVG with light lettering for dark backgrounds.
- `commit-command-banner-*-preview.png`: full-resolution transparent raster previews.
- `commit-command-icon-1024.png`: high-resolution icon PNG.
- `CommitCommand.icns`: macOS icon bundle.
- `CommitCommand.iconset/`: all ten Apple iconset PNGs (16–1024 px, including @2x sizes).

## Implementation notes

The banners use scalable SVG shapes and editable text, with the **approved illustrated icon embedded as a PNG** to retain the original glossy details. Thus the banners are self-contained SVG files, but not fully vector-only. The iconset is likewise derived directly from the approved icon artwork, not redrawn. The backgrounds of both banners are transparent.

The title uses a system sans-serif fallback; its appearance can vary across renderers. If strict pixel-identical typography is required, outline the text in a vector editor after selecting the production typeface.

### macOS build integration

In a Tauri app, use `CommitCommand.icns` in the macOS bundle resources/icons configuration, or export the included PNG sizes as required by the Tauri packaging tool.

On macOS, the iconset can also be rebuilt using:

```bash
iconutil -c icns CommitCommand.iconset -o CommitCommand.icns
```
