#!/usr/bin/env python3
"""Verify saved oracle evidence; standard library only, no candidate execution."""
import argparse
from hashlib import sha256
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parent
PRIVATE_PATH = re.compile(r'/Users/[A-Za-z0-9_.-]+/|/home/[A-Za-z0-9_.-]+/|[A-Za-z]:\\Users\\[^\\\s]+\\')
CREDENTIAL = re.compile(r'sk-[A-Za-z0-9]{20,}|AKIA[A-Z0-9]{16}|-----BEGIN [A-Z ]*PRIVATE KEY-----')
TEXT_SUFFIXES = {'.json', '.md', '.py', '.txt', '.weftforge', '.dxf', '.autosave'}


def digest(path):
    return sha256(path.read_bytes()).hexdigest()


def self_test():
    assert PRIVATE_PATH.search('/Users/' + 'example' + '/secret/file')
    assert PRIVATE_PATH.search('/home/' + 'example' + '/secret/file')
    assert not PRIVATE_PATH.search('${WORKSPACE}/relative/file')
    assert CREDENTIAL.search('sk-' + 'a' * 24)
    assert not CREDENTIAL.search('fixture stable ID 23')
    assert sha256(b'abc').hexdigest() == 'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad'


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--require-git-tracked', action='store_true')
    parser.add_argument('--allow-missing-external', action='store_true', help='Preparation only: explicitly report the root-owned image not merged yet.')
    args = parser.parse_args()
    self_test()
    manifest = json.loads((ROOT/'bundle-manifest.json').read_bytes())
    errors, pending = [], []
    expected = {**manifest['files'], **manifest['external_files']}
    for name, info in expected.items():
        path = ROOT/name
        if not path.is_file():
            if name in manifest['external_files'] and args.allow_missing_external:
                pending.append(name)
            else:
                errors.append('missing ' + name)
            continue
        if path.is_symlink():
            errors.append('symlink is not original delivery: ' + name)
        if digest(path) != info['sha256'] or path.stat().st_size != info['bytes']:
            errors.append('hash/size mismatch: ' + name)
        if path.suffix.lower() in TEXT_SUFFIXES:
            try:
                text = path.read_text(encoding='utf-8')
                if PRIVATE_PATH.search(text) or CREDENTIAL.search(text):
                    errors.append('privacy pattern: ' + name)
            except UnicodeError:
                errors.append('unexpected non-text file: ' + name)
    actual = {str(p.relative_to(ROOT)) for p in ROOT.rglob('*') if p.is_file() and '__pycache__' not in p.parts}
    extra = actual - set(expected) - {'bundle-manifest.json'}
    if extra:
        errors.append('unlisted files: ' + ', '.join(sorted(extra)))
    for relative, wanted in [('freeze.json','20fc2dc605374c5ee9b4056c3716e18156266515db2a82bbc02f99288f7e6560'),('supplement-cancel-draft/freeze.json','a93edd4e0275c752a65a6bb3d041828c580440b7314934eec7b5d301e4011d79')]:
        if digest(ROOT/relative) != wanted:
            errors.append('original freeze changed: ' + relative)
        freeze = json.loads((ROOT/relative).read_bytes())
        for name, wanted_hash in freeze['files'].items():
            if digest((ROOT/relative).parent/name) != wanted_hash:
                errors.append('frozen file changed: ' + str(Path(relative).parent/name))
    if args.require_git_tracked:
        repo = subprocess.check_output(['git','-C',str(ROOT),'rev-parse','--show-toplevel'],text=True).strip()
        names = [str((ROOT/name).relative_to(repo)) for name in sorted(actual)]
        check = subprocess.run(['git','-C',repo,'ls-files','--error-unmatch','--',*names],capture_output=True,text=True)
        if check.returncode:
            errors.append('some evidence files are not Git-tracked: ' + check.stderr.strip())
    result = {'ok':not errors,'complete':not errors and not pending,'file_count':len(actual),'pending_external_files':pending,'errors':errors,'privacy_scope':'Text pattern scan plus separately recorded visual review; no absolute personal path or credential pattern found when errors is empty.','git_tracking_required':args.require_git_tracked}
    print(json.dumps(result,indent=2))
    return 0 if not errors else 1


if __name__ == '__main__':
    raise SystemExit(main())
