# SPDX-License-Identifier: Apache-2.0
"""Verify the common renderer snapshot and consumer paths; never regenerates pins."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import tomllib

def digest(data):
    return hashlib.sha256(data).hexdigest()

def consumer_kit(manifest):
    manifest = Path(manifest).resolve()
    data = tomllib.loads(manifest.read_text(encoding='utf-8'))
    renderer = (manifest.parent / data['patch']['crates-io']['iced_tiny_skia']['path']).resolve()
    kit = renderer.parent
    regression = data['dev-dependencies']['sk-iced-renderer-regression']
    if regression['version'] != '=0.1.0' or (manifest.parent / regression['path']).resolve() != kit / 'regression':
        raise RuntimeError('Renderer and regression dependency must select the same companion kit')
    if renderer != kit / 'iced_tiny_skia':
        raise RuntimeError('Unexpected renderer companion layout')
    return kit

def verify(kit, upstream=None):
    kit = Path(kit).resolve()
    record = json.loads((kit / 'SHA256SUMS.json').read_text())
    expected = record['files']
    actual = {p.relative_to(kit).as_posix() for p in kit.rglob('*') if p.is_file() and '__pycache__' not in p.parts and p.name != 'SHA256SUMS.json'}
    if actual != set(expected):
        raise RuntimeError('Shared renderer file inventory changed; review and update the common snapshot')
    for name, sha in expected.items():
        path = (kit / name).resolve()
        if not path.is_relative_to(kit) or path.is_symlink() or digest(path.read_bytes()) != sha:
            raise RuntimeError('Shared renderer source hash mismatch: ' + name)
    spec = json.loads((kit / 'patches/manifest.json').read_text())
    patch = (kit / 'patches' / spec['patch']).read_bytes()
    if digest(patch) != spec['patch_sha256']:
        raise RuntimeError('Shared renderer patch hash mismatch')
    for name, pins in spec['files'].items():
        if digest((kit / 'iced_tiny_skia' / name).read_bytes()) != pins['after']:
            raise RuntimeError('Renderer does not match common patch: ' + name)
    if upstream is not None:
        upstream = Path(upstream)
        lines = patch.decode().splitlines(keepends=True)
        i = 0
        while i < len(lines):
            if not lines[i].startswith('--- '): raise RuntimeError('Invalid patch header')
            name = lines[i+1][6:].strip(); i += 2
            pins = spec['files'][name]
            data = (upstream / name).read_bytes() if pins['before'] else b''
            if pins['before'] and digest(data) != pins['before']: raise RuntimeError('Pristine source hash mismatch: ' + name)
            source, result, cursor = data.decode().splitlines(keepends=True), [], 0
            while i < len(lines) and lines[i].startswith('@@ '):
                match = re.match(r'@@ -(\d+)(?:,\d+)? \+\d+(?:,\d+)? @@', lines[i])
                if not match: raise RuntimeError('Invalid patch hunk')
                start = max(0,int(match[1])-1); result.extend(source[cursor:start]); cursor=start; i+=1
                while i < len(lines) and not lines[i].startswith(('@@ ','--- ')):
                    kind, content = lines[i][0], lines[i][1:]
                    if kind in ' -':
                        if cursor >= len(source) or source[cursor] != content: raise RuntimeError('Patch context mismatch: ' + name)
                        cursor += 1
                    if kind in ' +': result.append(content)
                    if kind not in ' +-': raise RuntimeError('Invalid patch line')
                    i += 1
            result.extend(source[cursor:])
            if digest(''.join(result).encode()) != pins['after']: raise RuntimeError('Patch result mismatch: ' + name)
    return spec

if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--consumer',type=Path)
    parser.add_argument('--upstream',type=Path)
    args=parser.parse_args()
    kit=consumer_kit(args.consumer) if args.consumer else Path(__file__).resolve().parents[1]
    verify(kit,args.upstream)
    print('Verified shared renderer, patch and regression snapshot:',kit)
