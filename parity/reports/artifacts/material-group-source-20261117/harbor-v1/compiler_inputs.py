"""Fixed compiler-input inventory for this one full Harbor producer; no Cargo invocation."""
import hashlib
import os
from pathlib import Path
import tomllib

LOCAL_ROOTS=('crates/weftforge-core','crates/weftforge-sim','vendor/earcutr','.local/harbor-material-groups-preparation-20261116/producer')


def sha(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def tree(path):
    path=Path(path)
    if path.is_symlink():raise ValueError(f'compiler-input root symlink refused: {path}')
    if not path.exists():return None
    result={}
    for p in sorted(path.rglob('*')):
        if p.is_symlink():raise ValueError(f'compiler-input symlink refused: {p}')
        if p.is_file():result[str(p.relative_to(path))]=sha(p)
    return result


def local_inventory(wt):
    wt=Path(wt)
    return {'roots':{name:tree(wt/name) for name in LOCAL_ROOTS},
            'automatic_build_rs':{name+'/build.rs':sha(wt/name/'build.rs') if (wt/name/'build.rs').is_file() else None for name in LOCAL_ROOTS}}


def dependency_inventory(wt,cargo_home):
    lock=tomllib.loads((Path(wt)/'.local/harbor-material-groups-preparation-20261116/producer/Cargo.lock').read_text());result=[]
    for package in lock['package']:
        if 'source' not in package:continue
        if package['source']!='registry+https://github.com/rust-lang/crates.io-index':raise ValueError('unexpected dependency source')
        candidates=sorted((Path(cargo_home)/'registry/src').glob('*/'+package['name']+'-'+package['version']))
        if len(candidates)!=1:raise ValueError(f'exact cached dependency source is ambiguous/missing: {package["name"]}')
        source=candidates[0]
        result.append({'name':package['name'],'version':package['version'],'lock_checksum':package['checksum'],'source_path':str(source),'files':tree(source)})
    return result


def cargo_config_inventory(wt,cargo_home):
    paths=set()
    for base in [Path(wt),*Path(wt).parents]:
        paths.update([base/'.cargo/config',base/'.cargo/config.toml'])
    paths.update([Path(cargo_home)/'config',Path(cargo_home)/'config.toml'])
    return {str(p):sha(p) if p.is_file() else None for p in sorted(paths)}


def ambient_build_env():
    return {k:v for k,v in sorted(os.environ.items()) if k.startswith(('CARGO_BUILD_','CARGO_PROFILE_','CARGO_TARGET_'))}
