#!/usr/bin/env python3
"""Explicit, isolated real Runtime + Echo acceptance for voice design and chapters."""
import argparse
import hashlib
import json
import selectors
import subprocess
import time
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cli', required=True, type=Path)
    parser.add_argument('--echo', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--resume', action='store_true')
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=args.resume)
    project = args.output / 'Longform.vox'
    evidence = args.output / 'steps.jsonl'

    def state(path=project):
        return json.loads((path / 'project.json').read_text())

    def request(action, path=project, **fields):
        payload = {'action': action, 'path': str(path), **fields}
        if path.exists():
            payload.setdefault('version', state(path)['version'])
        done = subprocess.run([str(args.cli), 'request'], input=json.dumps(payload),
                              text=True, capture_output=True, timeout=900)
        replies = [json.loads(line) for line in done.stdout.splitlines() if line.strip()]
        reply = next((item for item in reversed(replies) if 'ok' in item), None)
        with evidence.open('a') as out:
            out.write(json.dumps({'action': action, 'reply': reply}) + '\n')
        if not reply or not reply['ok']:
            raise RuntimeError((action, reply, done.stderr[-1000:]))
        print(action, 'ok', flush=True)
        return reply['result']

    probe = request('probe')
    assert probe['ready'], probe
    if not project.exists():
        request('create', title='Vox voice design and chapter production')
    request('open')
    if not state().get('voice_design_candidates'):
        result = request('generate_voice_design_batch', variations=2,
                         spec={'description': 'A calm, warm adult narrator with a medium-low pitch, clear diction and an unhurried delivery.',
                               'text': 'Welcome to the studio. Every chapter begins with a voice.',
                               'language': 'English'})
        assert result['voice_design_batch']['state'] == 'completed', result['voice_design_batch']
    designs = state()['voice_design_candidates']
    assert len(designs) == 2
    for sample in designs:
        assert sample['generation']['provenance']['intent'] == 'speech.design_voice'
        assert sample['contract']['origin'] == 'synthetic_speech'
    if not state().get('voice_library'):
        request('accept_voice_design', id=designs[0]['id'], name='Studio Narrator')
    voice = state()['voice_library'][0]
    assert voice['origin'] == 'designed', voice['origin']
    if not state()['draft']['options'].get('reference_roles', {}).get('Narrator'):
        request('bind_reference', id=voice['id'], role='Narrator')
    draft = state()['draft']
    if draft['language'] != 'English':
        draft['language'] = 'English'
        request('save', document=draft)
    source = args.output / 'chapters.md'
    source.write_text('# First chapter\n\nFirst chapter text.\n\n# Second chapter\n\nSecond chapter text.\n')
    active = [c for c in state().get('chapters', []) if not c['archived']]
    if args.resume and len(active) == 1 and not state().get('chapter_batches'):
        request('chapter_archive', chapter_id=active[0]['id'])
        active = []
    if not active:
        request('chapter_import', files=[str(source)])
    chapters = [c for c in state()['chapters'] if not c['archived']]
    assert len(chapters) == 2, chapters
    scripts = [
        '[production: dialogue]\n[role: Narrator]\n\n[speaker: Narrator]\nThe first chapter starts here.\n[pause: 0.1s]\n[speaker: Narrator]\nA quiet morning brings a new idea.',
        '[production: dialogue]\n[role: Narrator]\n\n[speaker: Narrator]\nThe second chapter takes us further.\n[pause: 0.1s]\n[speaker: Narrator]\nWe keep the same voice throughout the story.'
    ]
    if not state().get('chapter_batches'):
        for chapter, text in zip(chapters, scripts):
            opened = request('chapter_open', chapter_id=chapter['id'])
            document = opened['project']['draft']
            document.update(text=text, format='production_script')
            request('chapter_save', chapter_id=chapter['id'], document=document)

    # Stop a live batch at the first reported completed immutable segment.
    cancellation = args.output / 'cancellation.json'
    if not cancellation.exists():
        worker = subprocess.Popen([str(args.cli), 'serve'], stdin=subprocess.PIPE,
                                  stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                  text=True, bufsize=1)
        selector = selectors.DefaultSelector()
        selector.register(worker.stdout, selectors.EVENT_READ)
        events, cancelled = [], False
        deadline = time.monotonic() + 900
        worker.stdin.write(json.dumps({'action': 'chapter_batch', 'path': str(project),
                                      'version': state()['version'], 'request_id': 'batch'}) + '\n')
        worker.stdin.flush()
        try:
            while time.monotonic() < deadline:
                if not selector.select(timeout=15):
                    continue
                line = worker.stdout.readline()
                if not line:
                    raise RuntimeError('Batch worker closed unexpectedly')
                event = json.loads(line)
                events.append(event)
                if event.get('event') == 'progress' and event.get('completed_segments', 0) >= 1 and not cancelled:
                    worker.stdin.write(json.dumps({'action': 'cancel', 'request_id': 'stop'}) + '\n')
                    worker.stdin.flush()
                    cancelled = True
                if event.get('request_id') == 'batch' and 'ok' in event:
                    assert event['ok'], event
                    assert event['result']['chapter_batch']['status'] == 'cancelled', event
                    break
            else:
                raise TimeoutError('Batch cancellation timed out')
        finally:
            worker.stdin.close()
            try:
                worker.wait(timeout=20)
            except subprocess.TimeoutExpired:
                worker.kill()
                worker.wait(timeout=5)
            selector.close()
        cancellation.write_text(json.dumps(events, indent=2))
        assert cancelled
    before_resume = {c['id']: len(state(project / 'chapters' / (c['id'] + '.vox')).get('segment_candidates', [])) for c in chapters}
    resumed = request('chapter_batch')
    assert resumed['chapter_batch']['status'] == 'completed', resumed['chapter_batch']
    assert any(item['reused_segments'] > 0 for item in resumed['chapter_batch']['chapters'])
    for chapter in chapters:
        child = project / 'chapters' / (chapter['id'] + '.vox')
        m = state(child)
        assert len(m['segment_candidates']) == 2
        assert not m.get('accepted_segments', []) and not m['audio'], 'Batch must not accept audio'
        request('chapter_accept', chapter_id=chapter['id'], audio_id=m['candidates'][-1]['id'])
    repeated = request('chapter_batch')
    assert all(item['status'] == 'skipped' for item in repeated['chapter_batch']['chapters'])

    # One changed sentence requires one new segment; the other chapter stays exact.
    change_evidence = args.output / 'changed.json'
    if not change_evidence.exists():
        first = chapters[0]['id']
        second_child = project / 'chapters' / (chapters[1]['id'] + '.vox')
        second_before = (second_child / 'project.json').read_bytes()
        opened = request('chapter_open', chapter_id=first)
        child = Path(opened['path'])
        previous_ids = [s['id'] for s in opened['project']['segment_candidates']]
        document = opened['project']['draft']
        document['text'] = document['text'].replace('A quiet morning brings a new idea.', 'A quiet evening brings a new idea.')
        request('chapter_save', chapter_id=first, document=document)
        changed = request('chapter_batch')
        assert changed['chapter_batch']['status'] == 'completed', changed
        m = state(child)
        assert len(m['segment_candidates']) == 3
        assert all(item in [s['id'] for s in m['segment_candidates']] for item in previous_ids)
        assert (second_child / 'project.json').read_bytes() == second_before
        request('chapter_accept', chapter_id=first, audio_id=m['candidates'][-1]['id'])
        change_evidence.write_text(json.dumps(changed['chapter_batch'], indent=2))
    if not (args.output / 'chapters-export').exists():
        exported = request('chapter_export', destination=str(args.output / 'chapters-export'))
        assert all(c['srt'] and c['vtt'] for c in exported['manifest']['chapters'])

    # Real Echo independent editor exports PCM24; Vox converts only by explicit choice.
    chapter_id = chapters[0]['id']
    handoff = args.output / 'echo-handoff'
    child = project / 'chapters' / (chapter_id + '.vox')
    original_audio = state(child)['audio_head']
    if not handoff.exists():
        request('chapter_begin_echo', chapter_id=chapter_id, audio_id=original_audio, destination=str(handoff))
    echo_evidence = args.output / 'echo'
    if not echo_evidence.exists():
        subprocess.run(['python3', str(Path(__file__).with_name('test_echo_process.py')),
                        '--binary', str(args.echo), '--source', str(handoff / 'source.wav'),
                        '--evidence', str(echo_evidence)], check=True, timeout=120)
    if not any(c['origin'] == 'echo_file_return' for c in state(child)['candidates'] + state(child)['audio']):
        request('chapter_review_echo', chapter_id=chapter_id, handoff=str(handoff / 'handoff.json'),
                file=str(echo_evidence / 'return-0.wav'), allow_pcm24=True)
        m = state(child)
        assert m['audio_head'] == original_audio
        request('chapter_accept', chapter_id=chapter_id, audio_id=m['candidates'][-1]['id'])
    if not (args.output / 'echo-export').exists():
        exported = request('chapter_export', destination=str(args.output / 'echo-export'))
        edited = exported['manifest']['chapters'][0]
        assert edited['srt'] is None and edited['vtt'] is None
        assert edited['subtitles_unavailable_reason']
    before = (project / 'project.json').read_bytes()
    request('open')
    assert before == (project / 'project.json').read_bytes()
    report = {'project': str(project), 'cli': str(args.cli),
              'cli_sha256': hashlib.sha256(args.cli.read_bytes()).hexdigest(),
              'voice_design_jobs': [d['generation']['job_id'] for d in designs],
              'chapters': [c['id'] for c in chapters], 'resume_candidates': before_resume,
              'cancellation': str(cancellation), 'changed': str(change_evidence),
              'echo_report': str(echo_evidence / 'report.json'), 'reopen_unchanged': True}
    (args.output / 'report.json').write_text(json.dumps(report, indent=2))
    print(json.dumps({'passed': True, 'project': str(project)}), flush=True)


if __name__ == '__main__':
    main()
