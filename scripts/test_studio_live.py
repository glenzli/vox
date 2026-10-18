#!/usr/bin/env python3
"""Explicit real local Consumer check using generated (not human) reference speech."""
import argparse,json,subprocess,hashlib
from pathlib import Path
parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--cli',type=Path,required=True);parser.add_argument('--output',type=Path,required=True);parser.add_argument('--resume',action='store_true');args=parser.parse_args()
args.output.mkdir(parents=True,exist_ok=args.resume);project=args.output/'Studio.vox'; state={};receipts=[]
def request(action,**fields):
    payload={'action':action,**fields}
    if action not in ('probe','create','voice_preview'):
        payload.update(path=str(project),version=state['version'])
    completed=subprocess.run([str(args.cli),'request'],input=json.dumps(payload),capture_output=True,text=True,timeout=620)
    lines=[json.loads(line) for line in completed.stdout.splitlines() if line.strip()]
    response=next((r for r in reversed(lines) if 'ok' in r),None)
    if not response or not response['ok']:raise RuntimeError((action,response,completed.stderr[-1000:]))
    result=response['result'];state.update(result.get('project',{}));print(action,'ok',flush=True);return result
probe=request('probe');assert probe['ready'],probe
if args.resume and project.exists(): state.update(json.loads((project/'project.json').read_text()));request('open')
else: request('create',path=str(project),title='Vox studio acceptance')
words='Welcome to Vox. This is a local synthetic reference. Every story begins with a voice.'
if not state.get('voice_library'):
    sample=request('voice_preview',voice='speech.voice.en.warm_male.v1',text=words)
    request('import_reference',name='Synthetic local reference',transcript=words,file=sample['file'],consent=True,recorded=False)
reference=state['voice_library'][-1]['id']
preview=request('reference_preview',id=reference,text='A familiar voice, with a new story to tell.')
receipts.append(preview['provenance'])
request('bind_reference',id=reference,role='Guide')
for name,prompt,music in [('Quiet bell','One soft bell in a quiet room. No speech, music or drums.',False),('古琴短乐','安静的古琴短音乐，不含人声。',True)]:
    request('generate_sound',name=name,spec={'prompt':prompt,'seconds':2,'seed':42,'music':music})
    material=state['sound_library'][-1];assert material['content'] is None
    receipts.append(material['generation']['provenance']);request('accept_sound',id=material['id'])
sound=state['sound_library'][0];request('bind_sound',id=sound['id'],label='turn')
draft=state['draft'];draft['format']='production_script';draft['text']='[production: dialogue]\n[role: Guide]\n[role: Narrator]\n[cue: turn; sound: external]\n\n[speaker: Guide]\nA familiar voice tells a new story.\n[audio: turn]\n[speaker: Narrator]\nEvery story begins with a voice.'
draft['options']['roles']['Narrator']={'alias':'speech.voice.zh.bright_female.v1','catalog_revision':'infer.speech.voice-aliases@20260922.1'}
request('save',document=draft);request('adopt');request('render')
audio=state['candidates'][-1];request('accept',audio_id=audio['id']);request('export',audio_id=audio['id'],destination=str(args.output/'combined.wav'))
original=(project/'project.json').read_bytes();request('open');assert (project/'project.json').read_bytes()==original
assert len(state['audio'])==1 and len(state['voice_library'])==1 and len(state['sound_library'])==2
report={'project':str(project),'audio':audio['id'],'script_bytes':len(state['draft']['text'].encode()),'receipts':receipts,'combined':audio['provenance'],'manifest_sha256':hashlib.sha256(original).hexdigest(),'reopen_unchanged':True}
(args.output/'report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2));print(json.dumps({'passed':True,'output':str(args.output),'jobs':[p['intent'] for p in receipts]},ensure_ascii=False))
