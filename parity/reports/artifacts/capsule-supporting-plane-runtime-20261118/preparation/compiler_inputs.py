"""Fixed compiler-input inventory for this one full Harbor producer; no Cargo invocation."""
import hashlib
import os
from pathlib import Path
import tomllib

LOCAL_ROOTS=('crates/weftforge-core','crates/weftforge-sim','vendor/earcutr','.local/harbor-capsule-supporting-plane-preparation-20261118/producer')


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



STUDIO_INPUT_ROOTS = ("apps", "crates", "vendor", "xtask", ".cargo")
STUDIO_INPUT_FILES = ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "build.rs")


def compiler_input_roster(base, names):
    base = Path(base)
    result = {}
    for name in names:
        root = base / name
        if root.is_symlink():
            raise ValueError("Compiler input symlink refused: " + str(root))
        if not root.exists():
            continue
        paths = sorted(root.rglob("*")) if root.is_dir() else [root]
        for path in paths:
            if path.is_symlink():
                raise ValueError("Compiler input symlink refused: " + str(path))
            if path.is_dir():
                continue
            if not path.is_file():
                raise ValueError("Nonregular compiler input refused: " + str(path))
            stat = path.stat()
            result[str(path.relative_to(base))] = {
                "sha256":sha(path), "bytes":stat.st_size, "mode":stat.st_mode & 0o777,
            }
    return result


def studio_input_roster(wt):
    # Inventory whole source trees, including automatic build/test/include inputs.
    return compiler_input_roster(wt, (*STUDIO_INPUT_ROOTS, *STUDIO_INPUT_FILES))


def validate_compile_coverage(studio, tested, producer, approved_producer):
    expected = {name:entry for name,entry in tested.items()
                if name in STUDIO_INPUT_FILES or
                any(name == root or name.startswith(root + "/") for root in STUDIO_INPUT_ROOTS)}
    if studio != expected:
        raise ValueError("Studio compiler/include/manifest roster is not covered by actual F-tested bytes/modes")
    if producer != approved_producer:
        raise ValueError("Standalone producer compiler roster differs from its approved derivative bytes/modes")


def dependency_inventory(wt,cargo_home):
    lock=tomllib.loads((Path(wt)/'.local/harbor-capsule-supporting-plane-preparation-20261118/producer/Cargo.lock').read_text());result=[]
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
