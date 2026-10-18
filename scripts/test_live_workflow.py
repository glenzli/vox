#!/usr/bin/env python3
"""Opt-in real Vox -> Infer -> candidate -> accept -> export -> Echo -> candidate proof."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import wave


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli",required=True,type=Path)
    parser.add_argument("--output",required=True,type=Path)
    parser.add_argument("--echo-binary",type=Path)
    parser.add_argument("--resume",action="store_true",help="Reuse a previously rendered, accepted fixture after an external-editor failure")
    args=parser.parse_args()
    args.output.mkdir(parents=True,exist_ok=args.resume)
    project=args.output/"Vox welcome.vox"
    transcript=[]
    def call(action,**fields):
        request={"action":action,**fields}
        result=subprocess.run([str(args.cli),"request"],input=json.dumps(request).encode()+b"\n",stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=360)
        lines=[json.loads(line) for line in result.stdout.splitlines()]
        response=next(v for v in reversed(lines) if "ok" in v)
        transcript.append({"action":action,"ok":response["ok"],"error":response.get("error")})
        if not response["ok"]:raise RuntimeError(f"{action}: {response.get('error')}")
        return response["result"]
    state=call("open",path=str(project)) if args.resume else call("create",path=str(project),title="Vox · A first conversation")
    if not args.resume:
        doc=state["project"]["draft"]
        doc["format"]="production_script"
        doc["text"]="[production: dialogue]\n[role: 旁白]\n[role: Guide]\n[cue: turn; sound: chime]\n\n[speaker: 旁白]\n[language: Chinese]\n你好，欢迎来到 Vox。\n[pause: 0.3s]\n[repeat: 2; gap: 0.4s; cue: turn]\n[speaker: Guide]\n[language: English]\nEvery story begins with a voice.\n[end-repeat]\n"
        voices=json.loads(subprocess.check_output([args.cli,"capabilities"]))["voices"]
        doc["options"]["roles"]={name:{"alias":voices[i]["alias"],"catalog_revision":"infer.speech.voice-aliases@20260922.1"} for name,i in [("旁白",0),("Guide",5)]}
    def mutate(action,**fields):
        nonlocal state
        state=call(action,path=str(project),version=state["project"]["version"],**fields)
        return state
    if args.resume:
        assert len(state["project"]["audio"])==1, "Resume only the incomplete original fixture"
        candidate=state["project"]["audio"][0]
    else:
        mutate("save",document=doc)
        assert state["analysis"]["ready"]
        mutate("adopt")
        print("Rendering two real multilingual voice segments through Vox authorization…",flush=True)
        mutate("render")
        candidate=state["project"]["candidates"][-1]
    provenance=candidate["provenance"]
    assert provenance["app_id"]=="vox"
    assert provenance["placement"]=="local" and provenance["fallback"]=="none"
    assert len(provenance["speech_segments"])==2
    assert any(p.get("replay_of") is not None for p in provenance["speech_script"]["pieces"])
    assert all(s["runtime"]["app_id"]=="vox" for s in provenance["speech_segments"])
    if not args.resume: mutate("accept",audio_id=candidate["id"])
    state=call("open",path=str(project))
    original=args.output/("original-resumed.wav" if args.resume else "original.wav")
    call("export",path=str(project),audio_id=candidate["id"],destination=str(original))
    original_hash=hashlib.sha256(original.read_bytes()).hexdigest()
    preview=call("playback",path=str(project),audio_id=candidate["id"])
    assert preview["peaks"] and max(preview["peaks"])>0
    with wave.open(str(original),"rb") as wav:
        assert (wav.getnchannels(),wav.getsampwidth(),wav.getframerate())==(1,2,24000)
        duration=wav.getnframes()/wav.getframerate()
    report={"project":str(project),"cli_sha256":hashlib.sha256(args.cli.read_bytes()).hexdigest(),
        "real_speech":True,"app_id":"vox","physical_jobs":[s["job_id"] for s in provenance["speech_segments"]],
        "exact_repeat":True,"duration_seconds":duration,"reopened":True,"exported":str(original)}
    if args.echo_binary:
        print("Editing the accepted take with the real Echo process…",flush=True)
        mutate("begin_echo",audio_id=candidate["id"],destination=str(args.output/("handoff-resumed" if args.resume else "handoff")))
        echo_output=args.output/("echo-resumed" if args.resume else "echo")
        subprocess.run([sys.executable,str(Path(__file__).with_name("test_echo_process.py")),"--binary",str(args.echo_binary),"--source",state["source_file"],"--evidence",str(echo_output)],check=True,timeout=90)
        raw=echo_output/"return-0.wav"
        request={"action":"review_echo","path":str(project),"version":state["project"]["version"],"handoff":state["handoff_file"],"file":str(raw),"allow_pcm24":False}
        result=subprocess.run([str(args.cli),"request"],input=json.dumps(request).encode()+b"\n",stdout=subprocess.PIPE,check=True)
        refused=json.loads(result.stdout)
        assert refused.get("error")=="pcm24_confirmation_required"
        ticket=state["handoff_file"]
        mutate("review_echo",handoff=ticket,file=str(raw),allow_pcm24=True)
        returned=state["project"]["candidates"][-1]
        assert returned["parent_audio"]==candidate["id"] and returned["edit"]["editor"]=="echo"
        assert len(state["project"]["audio"])==1
        mutate("accept",audio_id=returned["id"])
        state=call("open",path=str(project))
        assert len(state["project"]["audio"])==2
        call("export",path=str(project),audio_id=returned["id"],destination=str(args.output/"echo-edit.wav"))
        call("export",path=str(project),audio_id=candidate["id"],destination=str(args.output/"original-after-echo.wav"))
        assert hashlib.sha256((args.output/"original-after-echo.wav").read_bytes()).hexdigest()==original_hash
        report.update({"real_echo":True,"explicit_pcm24_conversion":True,"original_unchanged":True,"echo_report":str(echo_output/"report.json")})
    report["transcript"]=transcript
    (args.output/"report.json").write_text(json.dumps(report,ensure_ascii=False,indent=2)+"\n")
    print(json.dumps({k:v for k,v in report.items() if k!="transcript"},ensure_ascii=False),flush=True)


if __name__=="__main__":main()
