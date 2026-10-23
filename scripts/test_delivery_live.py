#!/usr/bin/env python3
"""Explicit isolated Echo -> local alignment -> compiled chapter delivery acceptance."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import wave


def tree_hashes(root):
    return {str(p.relative_to(root)): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in root.rglob('*') if p.is_file()}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cli', required=True, type=Path)
    parser.add_argument('--echo', required=True, type=Path)
    parser.add_argument('--fixture', required=True, type=Path,
                        help='Existing isolated two-chapter accepted production fixture; copied before use')
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    source_hashes = tree_hashes(args.fixture)
    project = args.output / 'Delivery.vox'
    shutil.copytree(args.fixture, project)
    evidence = args.output / 'steps.jsonl'

    def state(path=project):
        return json.loads((path / 'project.json').read_text())

    def request(action, path=project, expected_error=None, **fields):
        payload = {'action': action, 'path': str(path), **fields}
        if path.exists():
            payload.setdefault('version', state(path)['version'])
        done = subprocess.run([str(args.cli), 'request'], input=json.dumps(payload),
                              text=True, capture_output=True, timeout=900)
        replies = [json.loads(line) for line in done.stdout.splitlines() if line.strip()]
        reply = next((item for item in reversed(replies) if 'ok' in item), None)
        with evidence.open('a') as out:
            out.write(json.dumps({'action': action, 'reply': reply}) + '\n')
        if expected_error:
            assert reply and not reply['ok'] and reply['error'] == expected_error, reply
            print(action, expected_error, flush=True)
            return reply
        if not reply or not reply['ok']:
            raise RuntimeError((action, reply, done.stderr[-1000:]))
        print(action, 'ok', flush=True)
        return reply['result']

    request('open')
    chapters = [c for c in state()['chapters'] if not c['archived']]
    assert len(chapters) == 2, 'Use the isolated long-form acceptance fixture'
    chapter_ids = [c['id'] for c in chapters]
    child = project / 'chapters' / (chapter_ids[0] + '.vox')
    untouched = project / 'chapters' / (chapter_ids[1] + '.vox')
    untouched_before = (untouched / 'project.json').read_bytes()
    before_audio = state(child)['audio_head']
    handoff = args.output / 'echo-handoff'
    request('chapter_begin_echo', chapter_id=chapter_ids[0], audio_id=before_audio,
            destination=str(handoff))
    echo_evidence = args.output / 'echo'
    subprocess.run(['python3', str(Path(__file__).with_name('test_echo_process.py')),
                    '--binary', str(args.echo), '--source', str(handoff / 'source.wav'),
                    '--evidence', str(echo_evidence)], check=True, timeout=120)
    request('chapter_review_echo', chapter_id=chapter_ids[0], handoff=str(handoff / 'handoff.json'),
            file=str(echo_evidence / 'return-0.wav'), allow_pcm24=True)
    candidate = state(child)['candidates'][-1]['id']
    assert state(child)['audio_head'] == before_audio
    request('chapter_accept', chapter_id=chapter_ids[0], audio_id=candidate)
    preview = request('subtitle_preview', path=child, audio_id=candidate)
    assert preview['accepted_audio'] and preview['track'] is None
    assert preview['transcript'].strip()
    result = request('align_subtitles', path=child, audio_id=candidate,
                     spec={'transcript': preview['transcript'], 'language': 'English'})
    outcome = result['subtitle_outcome']
    assert outcome['state'] == 'completed', outcome
    subtitle_id = outcome['candidate_id']
    assert subtitle_id and candidate not in state(child).get('subtitle_selections', {})
    pending = request('subtitle_preview', path=child, audio_id=candidate, candidate_id=subtitle_id)
    assert pending['track']['source'] == 'forced_alignment' and pending['track']['cues']
    request('export_subtitles', path=child, audio_id=candidate, format='srt',
            destination=str(args.output / 'unaccepted.srt'), expected_error='subtitles_unavailable')
    assert not (args.output / 'unaccepted.srt').exists()
    request('accept_subtitles', path=child, id=subtitle_id)
    accepted = request('subtitle_preview', path=child, audio_id=candidate)
    assert accepted['track']['alignment_revision'] == subtitle_id
    for fmt in ('srt', 'vtt'):
        request('export_subtitles', path=child, audio_id=candidate, format=fmt,
                destination=str(args.output / ('edited.' + fmt)))
    separate = request('chapter_export', destination=str(args.output / 'chapters'))
    assert all(c['srt'] and c['vtt'] for c in separate['manifest']['chapters'])

    options = {'gap_ms': 350, 'audio_mode': 'preserve', 'subtitles': 'required'}
    preserve = request('chapter_compilation_preflight', chapter_ids=chapter_ids, options=options)
    assert not preserve['ready'], 'Echo stereo and synthetic mono require an explicit format choice'
    options['audio_mode'] = 'normalize_mono24000'
    ready = request('chapter_compilation_preflight', chapter_ids=chapter_ids, options=options)
    assert ready['ready'], ready
    destination = args.output / 'compiled'
    compiled = request('chapter_compile', chapter_ids=chapter_ids, options=options,
                       fingerprint=ready['fingerprint'], destination=str(destination))
    assert compiled['exported']
    with wave.open(str(destination / 'compilation.wav'), 'rb') as wav:
        assert wav.getframerate() == 24000 and wav.getnchannels() == 1 and wav.getsampwidth() == 2
        assert wav.getnframes() == ready['output']['frames']
    for name in ('manifest.json', 'chapters.json', 'chapters.ffmetadata', 'compilation.srt', 'compilation.vtt'):
        assert (destination / name).is_file(), name
    export_hashes = tree_hashes(destination)
    repeated = subprocess.run([str(args.cli), 'request'], text=True, capture_output=True,
        input=json.dumps({'action': 'chapter_compile', 'path': str(project),
            'version': state()['version'], 'chapter_ids': chapter_ids, 'options': options,
            'fingerprint': ready['fingerprint'], 'destination': str(destination)}), timeout=30)
    assert not json.loads(repeated.stdout.splitlines()[-1])['ok']
    assert tree_hashes(destination) == export_hashes
    # Parent metadata change invalidates the review fingerprint, even with a fresh version.
    title = chapters[1]['title']
    request('chapter_rename', chapter_id=chapter_ids[1], title=title + ' revised')
    request('chapter_compile', chapter_ids=chapter_ids, options=options,
            fingerprint=ready['fingerprint'], destination=str(args.output / 'stale'),
            expected_error='compilation_preflight_changed')
    assert not (args.output / 'stale').exists()
    request('chapter_rename', chapter_id=chapter_ids[1], title=title)
    assert (untouched / 'project.json').read_bytes() == untouched_before
    before_reopen = (child / 'project.json').read_bytes()
    request('open', path=child)
    assert (child / 'project.json').read_bytes() == before_reopen
    assert tree_hashes(args.fixture) == source_hashes
    report = {'passed': True, 'project': str(project), 'worker': str(args.cli),
        'worker_sha256': hashlib.sha256(args.cli.read_bytes()).hexdigest(),
        'alignment_revision': subtitle_id, 'track': accepted['track'],
        'alignment_receipt': next(s['generation'] for s in state(child)['subtitles'] if s['id'] == subtitle_id),
        'compilation': str(destination), 'preflight': ready,
        'echo_report': str(echo_evidence / 'report.json'), 'no_clobber': True,
        'stale_preflight_rejected': True, 'fixture_unchanged': True, 'other_chapter_unchanged': True}
    (args.output / 'report.json').write_text(json.dumps(report, indent=2))
    print(json.dumps({'passed': True, 'project': str(project)}), flush=True)


if __name__ == '__main__':
    main()
