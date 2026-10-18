#!/usr/bin/env python3
"""Verify that the native English source catalog has a complete Chinese presentation."""
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import xml.etree.ElementTree as E

path=Path(__file__).resolve().parents[1]/"apps/desktop/translations/vox_zh_CN.ts"
root=E.parse(path).getroot()
keys={(c.findtext("name"),m.findtext("source")) for c in root.findall("context") for m in c.findall("message")}
extractor=shutil.which("lupdate") or "/opt/homebrew/opt/qttools/bin/lupdate"
with tempfile.TemporaryDirectory(prefix="vox-translations-") as temporary:
    extracted=Path(temporary)/"source.ts"
    subprocess.run([extractor,str(path.parents[1]/"qml"),str(path.parents[1]/"src"),"-ts",str(extracted),"-no-obsolete"],check=True,capture_output=True)
    expected={(c.findtext("name"),m.findtext("source")) for c in E.parse(extracted).getroot().findall("context") for m in c.findall("message")}
    missing=expected-keys
    if missing:raise SystemExit(f"Source messages absent from Chinese catalog: {sorted(missing)}")
count=0
for context in root.findall("context"):
    seen=set()
    for message in context.findall("message"):
        source=message.findtext("source") or ""
        translation=message.find("translation")
        if source in seen:raise SystemExit(f"Duplicate source in {context.findtext('name')}: {source}")
        seen.add(source)
        if translation is None or translation.get("type") in ("unfinished","vanished","obsolete") or not (translation.text or "").strip():raise SystemExit(f"Missing Chinese translation: {source}")
        if sorted(re.findall(r"%\d+",source))!=sorted(re.findall(r"%\d+",translation.text)):raise SystemExit(f"Changed placeholders: {source}")
        count+=1
print(f"{count} complete Chinese messages; placeholders preserved")
