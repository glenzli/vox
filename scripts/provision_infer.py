#!/usr/bin/env python3
"""Explicit local operator setup for Vox; never borrows another Consumer credential."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import tomllib
import urllib.request


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--console", required=True, help="Explicit local Infer Console URL")
    parser.add_argument("--cli", required=True, type=Path)
    parser.add_argument("--credential", type=Path, default=Path.home()/".config/vox/infer-credential")
    parser.add_argument("--backup", type=Path, required=True)
    parser.add_argument("--studio", action="store_true", help="Explicitly add local reference-voice, sound and faithful Chinese prompt grants to Vox")
    args = parser.parse_args()
    if not re.fullmatch(r"http://127\.0\.0\.1:\d+", args.console):
        raise SystemExit("Use an explicit numeric-loopback Console URL")
    op = urllib.request.build_opener(urllib.request.ProxyHandler({}))
    html = op.open(args.console+"/", timeout=8).read().decode()
    csrf = re.search(r'<meta name="infer-console-session" content="([^"]+)"', html).group(1)

    def call(path, data=None, method=None):
        request = urllib.request.Request(args.console+path,
            data=None if data is None else json.dumps(data).encode(), method=method,
            headers={"Content-Type":"application/json", "x-infer-console-session":csrf})
        value = json.load(op.open(request, timeout=20))
        if value.get("ok") is False:
            raise RuntimeError("Console rejected the operation")
        return value.get("result", value)

    import subprocess
    aliases = [v["alias"] for v in json.loads(subprocess.check_output([args.cli,"capabilities"]))["voices"]]
    before = call("/api/config")["source"]
    before_config = tomllib.loads(before)
    apps = {a["app_id"]:a for a in call("/api/access/apps")["apps"]}
    if not args.backup.exists():
        fd = os.open(args.backup, os.O_WRONLY|os.O_CREAT|os.O_EXCL, 0o600)
        with os.fdopen(fd,"w") as f: f.write(before)
    if "vox" not in apps:
        if args.credential.exists():
            raise RuntimeError("A Vox credential already exists; review its identity before provisioning")
        issued = call("/api/access/apps", {
            "app_id":"vox", "allowed_intents":["speech.synthesize"],
            "allowed_builtin_tools":[], "allowed_provider_access_classes":["standard"],
            "allowed_cloud_input_modalities":[], "max_pending_jobs":4,
            "default_policy":"local-first", "allowed_policies":["local-first"],
            "request_overrides":{"priority":["interactive"],"placement":["local_only"],
                "prefer":["local"],"offline_required":True,"capability_floor":["capable"],
                "latency":["interactive"],"fallback":["none"],"max_cost_usd":{"min":0.0,"max":0.0}}
        }, "POST")
        if issued.get("app_id")!="vox": raise RuntimeError("Unexpected issued identity")
        args.credential.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
        os.chmod(args.credential.parent,0o700)
        fd=os.open(args.credential,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
        with os.fdopen(fd,"w") as f: f.write(issued["token"]); f.flush(); os.fsync(f.fileno())
    elif not args.credential.is_file():
        raise RuntimeError("Vox exists but its local credential is missing; reissue it explicitly in Console")
    current = call("/api/config")["source"]
    config = tomllib.loads(current)
    updated = current
    if config["apps"]["vox"].get("allowed_speech_voice_aliases") != aliases:
        if "allowed_speech_voice_aliases" in config["apps"]["vox"]:
            raise RuntimeError("Existing Vox voice grant differs; review it before changing")
        if current.count("[apps.vox]\n")!=1: raise RuntimeError("Unrecognized Console config layout")
        updated = current.replace("[apps.vox]\n", "[apps.vox]\nallowed_speech_voice_aliases = "+json.dumps(aliases)+"\n",1)
    route = config["apps"]["vox"].get("routing",{}).get("intents",{}).get("speech.synthesize")
    if route is None:
        updated += '\n[apps.vox.routing.intents."speech.synthesize"]\ndeployment_ids = ["mlx_qwen3_tts_custom_voice_1_7b"]\nmodel_profile_ids = []\n'
    elif route.get("deployment_ids") != ["mlx_qwen3_tts_custom_voice_1_7b"]:
        raise RuntimeError("Existing Vox route differs; review it before changing")
    if updated != current:
        if call("/api/config")["source"] != current: raise RuntimeError("Configuration changed; retry after review")
        call("/api/config",{"source":updated},"PUT")
    if args.studio:
        current_apps = {a["app_id"]:a for a in call("/api/access/apps")["apps"]}
        app = current_apps["vox"]
        allowed = ["speech.synthesize", "speech.clone_voice", "audio.generate_sound", "text.edit"]
        overrides = dict(app["request_overrides"])
        overrides.update({"priority":["interactive","background"], "capability_floor":["capable","foundational"], "latency":["interactive","balanced"]})
        if app["allowed_intents"] != allowed or app["request_overrides"] != overrides:
            fields = ["allowed_builtin_tools", "allowed_provider_access_classes", "allowed_cloud_input_modalities", "max_pending_jobs", "default_policy", "allowed_policies"]
            body = {key:app[key] for key in fields}
            body.update({"app_id":"vox", "allowed_intents":allowed, "request_overrides":overrides})
            call("/api/access/apps/vox", body, "PUT")
        current = call("/api/config")["source"]
        config = tomllib.loads(current)
        updated = current
        routes = {"speech.clone_voice":["mlx_qwen3_tts_base_1_7b"], "audio.generate_sound":["stable_audio_3_sm_sfx_mlx","stable_audio_3_sm_music_mlx"], "text.edit":["ollama_qwen3_5_4b"]}
        for intent, ids in routes.items():
            route = config["apps"]["vox"].get("routing",{}).get("intents",{}).get(intent)
            if route is None:
                updated += '\n[apps.vox.routing.intents.'+json.dumps(intent)+']\ndeployment_ids = '+json.dumps(ids)+'\nmodel_profile_ids = []\n'
            elif route.get("deployment_ids") != ids:
                raise RuntimeError("Existing Vox studio route differs; review it before changing")
        if updated != current:
            if call("/api/config")["source"] != current: raise RuntimeError("Configuration changed; retry after review")
            call("/api/config", {"source":updated}, "PUT")
    after = tomllib.loads(call("/api/config")["source"])
    previous = dict(before_config); final = dict(after)
    previous["apps"] = {k:v for k,v in previous["apps"].items() if k!="vox"}
    final["apps"] = {k:v for k,v in final["apps"].items() if k!="vox"}
    if previous!=final: raise RuntimeError("An unrelated configuration changed; review before enabling")
    print(json.dumps({"app_id":"vox","credential_installed":args.credential.is_file(),
        "speech_aliases":len(aliases),"local_only":True,"other_apps_unchanged":True,
        "restart_required":call("/api/access/apps")["restart_required"],
        "previous_config_sha256":hashlib.sha256(before.encode()).hexdigest()}))


if __name__=="__main__": main()
