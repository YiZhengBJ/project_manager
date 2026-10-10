#!/usr/bin/env python3
"""Create a macOS installer without requesting Finder automation permission."""
import pathlib
import platform
import shutil
import subprocess
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[1]

if platform.system() != 'Darwin':
    raise SystemExit('This packaging script builds a macOS DMG; use native Tauri bundles on other systems.')

app = ROOT / 'src-tauri' / 'target' / 'release' / 'bundle' / 'macos' / 'Project Manager.app'
if not app.is_dir():
    raise SystemExit('Build the app first with: pnpm build:app')
output = ROOT / 'src-tauri' / 'target' / 'release' / 'bundle' / 'dmg'
output.mkdir(parents=True, exist_ok=True)
dmg = output / ('Project Manager_1.0.0_' + platform.machine() + '.dmg')
with tempfile.TemporaryDirectory(prefix='project-manager-dmg-') as temporary:
    staging = pathlib.Path(temporary)
    shutil.copytree(app, staging / app.name, symlinks=True)
    (staging / 'Applications').symlink_to('/Applications')
    subprocess.run(['hdiutil', 'create', '-volname', 'Project Manager', '-srcfolder', str(staging),
                    '-ov', '-format', 'UDZO', str(dmg)], check=True)
print('DMG created: ' + str(dmg))
