#!/usr/bin/env python3
"""Exercise offline subtitle correction/import on a copy of a real delivery fixture."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import wave


def hashes(root):
    return {str(p.relative_to(root)): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in root.rglob('*') if p.is_file()}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cli', required=True, type=Path)
    parser.add_argument('--fixture', required=True, type=Path,
                        help='Isolated accepted two-chapter delivery fixture; copied before use')
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    before_fixture = hashes(args.fixture)
    collection = args.output / 'Refinement.vox'
    shutil.copytree(args.fixture, collection)
    evidence = args.output / 'steps.jsonl'

    def state(path):
        return json.loads((path / 'project.json').read_text())

    def request(action, path, failure=False, **fields):
        payload = {'action': action, 'path': str(path), 'version': state(path)['version'], **fields}
        done = subprocess.run([str(args.cli), 'request'], input=json.dumps(payload),
                              text=True, capture_output=True, timeout=90)
        replies = [json.loads(line) for line in done.stdout.splitlines() if line.strip()]
        reply = next((item for item in reversed(replies) if 'ok' in item), None)
        with evidence.open('a') as out:
            out.write(json.dumps({'action': action, 'reply': reply}) + '\n')
        assert reply and reply['ok'] != failure, (action, reply, done.stderr[-1000:])
        print(action, reply.get('error', 'ok'), flush=True)
        return reply if failure else reply['result']

    chapters = [c for c in state(collection)['chapters'] if not c['archived']]
    assert len(chapters) == 2
    ids = [c['id'] for c in chapters]
    child = collection / 'chapters' / (ids[0] + '.vox')
    other = collection / 'chapters' / (ids[1] + '.vox')
    before_other = hashes(other)
    baseline = state(child)
    audio_id = baseline['audio_head']
    initial = request('subtitle_preview', child, audio_id=audio_id)
    assert initial['track']['source'] == 'forced_alignment' and initial['accepted_audio']
    current_cues = initial['track']['cues']
    assert len(current_cues) >= 2
    edited_cues = json.loads(json.dumps(current_cues))
    edited_cues[0]['text'] = edited_cues[0]['text'].rstrip('.!') + '!'
    edited_cues[0]['start_ms'] += 10
    old_version = baseline['version']
    edited = request('subtitle_edit', child, audio_id=audio_id, cues=edited_cues)
    edit_id = edited['subtitle_candidate_id']
    candidate = request('subtitle_preview', child, audio_id=audio_id, candidate_id=edit_id)
    assert candidate['track']['source'] == 'manual_revision'
    assert candidate['track']['revision_id'] == edit_id
    assert candidate['track']['cues'] == edited_cues
    assert request('subtitle_preview', child, audio_id=audio_id)['track']['cues'] == current_cues
    request('subtitle_edit', child, failure=True, version=old_version,
            audio_id=audio_id, cues=edited_cues)
    unchanged = (child / 'project.json').read_bytes()
    invalid = json.loads(json.dumps(edited_cues))
    invalid[0]['end_ms'] = initial['duration_ms'] + 1
    request('subtitle_edit', child, failure=True, audio_id=audio_id, cues=invalid)
    assert (child / 'project.json').read_bytes() == unchanged
    request('accept_subtitles', child, id=edit_id)

    # A saved child correction must retain its own parent evidence after discard.
    parent_cues = json.loads(json.dumps(edited_cues))
    parent_cues[1]['start_ms'] += 5
    parent = request('subtitle_edit', child, audio_id=audio_id, cues=parent_cues)
    parent_id = parent['subtitle_candidate_id']
    nested_cues = json.loads(json.dumps(parent_cues))
    nested_cues[0]['end_ms'] -= 5
    nested = request('subtitle_edit', child, audio_id=audio_id,
                     candidate_id=parent_id, cues=nested_cues)
    nested_id = nested['subtitle_candidate_id']
    request('discard_subtitles', child, id=parent_id)
    assert request('subtitle_preview', child, audio_id=audio_id,
                   candidate_id=nested_id)['track']['cues'] == nested_cues
    request('discard_subtitles', child, id=nested_id)

    for fmt in ('srt', 'vtt'):
        destination = args.output / ('corrected.' + fmt)
        request('export_subtitles', child, audio_id=audio_id,
                destination=str(destination), format=fmt)
        imported = request('subtitle_import', child, audio_id=audio_id,
                           file=str(destination), format=fmt, language='English')
        import_id = imported['subtitle_candidate_id']
        preview = request('subtitle_preview', child, audio_id=audio_id, candidate_id=import_id)
        assert preview['track']['source'] == 'imported_subtitles'
        assert preview['track']['cues'] == edited_cues
        request('accept_subtitles', child, id=import_id)
    imported_version = state(child)
    assert imported_version['subtitle_selections'][audio_id] == import_id

    # Preview metadata follows the exact reviewed track, not the source default.
    language_candidate = request('subtitle_import', child, audio_id=audio_id,
                                 file=str(args.output / 'corrected.srt'),
                                 format='srt', language='Spanish')['subtitle_candidate_id']
    assert request('subtitle_preview', child, audio_id=audio_id,
                   candidate_id=language_candidate)['language'] == 'Spanish'
    request('discard_subtitles', child, id=language_candidate)

    invalid_file = args.output / 'invalid.srt'
    invalid_file.write_text('1\n00:00:02,000 --> 00:00:01,000\nInvalid time\n')
    before_invalid = (child / 'project.json').read_bytes()
    request('subtitle_import', child, failure=True, audio_id=audio_id,
            file=str(invalid_file), format='srt')
    assert (child / 'project.json').read_bytes() == before_invalid

    options = {'gap_ms': 350, 'audio_mode': 'normalize_mono24000', 'subtitles': 'required'}
    ready = request('chapter_compilation_preflight', collection, chapter_ids=ids, options=options)
    assert ready['ready'], ready
    first_preview = next(item for item in ready['chapters'] if item['chapter_id'] == ids[0])
    assert first_preview['subtitle_revision'] == import_id
    assert first_preview.get('alignment_revision') is None
    compiled = args.output / 'compiled'
    request('chapter_compile', collection, chapter_ids=ids, options=options,
            fingerprint=ready['fingerprint'], destination=str(compiled))
    assert edited_cues[0]['text'] in (compiled / 'compilation.srt').read_text()
    with wave.open(str(compiled / 'compilation.wav'), 'rb') as wav:
        assert wav.getnframes() == ready['output']['frames']
    request('chapter_export', collection, destination=str(args.output / 'chapters'))

    # Existing genuine alignment remains selectable; imported text never overwrites it.
    old_alignment = initial['track']['alignment_revision']
    request('accept_subtitles', child, id=old_alignment)
    assert request('subtitle_preview', child, audio_id=audio_id)['track']['cues'] == current_cues
    request('accept_subtitles', child, id=import_id)
    saved = (child / 'project.json').read_bytes()
    request('open', child)
    assert (child / 'project.json').read_bytes() == saved
    final = state(child)
    for key in ('audio', 'candidates', 'audio_head', 'sources', 'source_head', 'draft', 'subtitle_runs'):
        assert final.get(key) == baseline.get(key), key
    assert hashes(other) == before_other
    assert hashes(args.fixture) == before_fixture
    report = {'passed': True, 'worker': str(args.cli),
              'worker_sha256': hashlib.sha256(args.cli.read_bytes()).hexdigest(),
              'collection': str(collection), 'edited_child': str(child), 'audio_id': audio_id,
              'manual_revision': edit_id, 'imported_revision': import_id,
              'runtime_jobs_unchanged': True, 'source_audio_unchanged': True,
              'fixture_unchanged': True, 'other_chapter_unchanged': True,
              'parent_candidate_discard_safe': True, 'stale_version_rejected': True,
              'invalid_inputs_preserved_manifest': True, 'compilation': str(compiled),
              'output': ready['output']}
    (args.output / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({'passed': True, 'report': str(args.output / 'report.json')}))


if __name__ == '__main__':
    main()
