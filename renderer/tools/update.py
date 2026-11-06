# SPDX-License-Identifier: Apache-2.0
"""Explicitly regenerate reviewed companion pins against a pristine registry package."""
import argparse,difflib,hashlib,json
from pathlib import Path

def sha(data): return hashlib.sha256(data).hexdigest()
def update(kit,upstream):
    kit=Path(kit).resolve();upstream=Path(upstream).resolve()
    spec=json.loads((kit/'patches/manifest.json').read_text())
    original=json.loads((upstream/'.cargo-checksum.json').read_text())
    if original['package']!=spec['upstream_sha256']: raise RuntimeError('Unexpected upstream package')
    for name,digest in original['files'].items():
        if sha((upstream/name).read_bytes())!=digest: raise RuntimeError('Modified pristine source: '+name)
    for p in kit.rglob('*'):
        if p.is_file() and '__pycache__' not in p.parts:
            if 'target' in p.relative_to(kit).parts or p.suffix in ['.exe','.dll','.pdb','.pyc']: raise RuntimeError('Build artifact in companion: '+str(p))
            p.write_bytes(p.read_bytes().replace(b'\r\n',b'\n'))
    patch='';files={}
    for p in sorted((kit/'iced_tiny_skia').rglob('*')):
        if not p.is_file(): continue
        name=p.relative_to(kit/'iced_tiny_skia').as_posix()
        before=(upstream/name).read_bytes() if (upstream/name).is_file() else b'';after=p.read_bytes()
        if before==after:continue
        files[name]={'before':sha(before) if before else None,'after':sha(after)}
        patch+=''.join(difflib.unified_diff(before.decode().splitlines(True),after.decode().splitlines(True),fromfile='a/'+name,tofile='b/'+name))
    spec['files']=files;spec['patch_sha256']=sha(patch.encode())
    (kit/'patches'/spec['patch']).write_bytes(patch.encode())
    (kit/'patches/manifest.json').write_bytes((json.dumps(spec,indent=2)+'\n').encode())
    pins={p.relative_to(kit).as_posix():sha(p.read_bytes()) for p in sorted(kit.rglob('*')) if p.is_file() and p.name!='SHA256SUMS.json' and '__pycache__' not in p.parts}
    (kit/'SHA256SUMS.json').write_bytes((json.dumps({'format':1,'files':pins},indent=2)+'\n').encode())
if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--upstream',required=True,type=Path);args=parser.parse_args()
    update(Path(__file__).resolve().parents[1],args.upstream)
    print('Updated reviewed common patch and snapshot; run both consumer suites before committing.')
