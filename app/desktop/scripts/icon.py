"""Sync the supplied Commit Command artwork without regenerating or redrawing it."""
from pathlib import Path
import shutil
root=Path(__file__).resolve().parents[3]
assets=root/'docs/graphics/commit-command-brand-assets'
target=root/'app/desktop/src-tauri/icons'
target.mkdir(parents=True,exist_ok=True)
shutil.copy2(assets/'CommitCommand.iconset/icon_128x128.png',target/'icon.png')
shutil.copy2(assets/'CommitCommand.icns',target/'CommitCommand.icns')
