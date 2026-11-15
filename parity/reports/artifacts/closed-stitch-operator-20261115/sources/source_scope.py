from pathlib import Path
import hashlib
import subprocess

BASE = 'c17d303e099836f022fe8f8222bc61b1cb33c3cd'
CANDIDATE = '35afac604a93174fcdc2a5538799432aa6d69286'
SIM = Path('crates/weftforge-sim/src')


def old(revision, path):
    return subprocess.check_output(['git', 'show', f'{revision}:{path}'])


constraints_path = SIM / 'constraints.rs'
current = constraints_path.read_bytes()
start = current.index(b'    if target_distance == 0. && s.compliance > 0. {')
end = current.index(b'    let distance = length(difference);', start)
normalized = (current[:start] + current[end:]).replace(
    b'    lambda: &mut Vec3,', b'    lambda: &mut f64,'
).replace(b'        &mut lambda[0],', b'        lambda,')
assert normalized == old(BASE, constraints_path)
candidate = old(CANDIDATE, constraints_path)
marker = b'pub(crate) fn solve_stitch('
assert current[current.index(marker):] == candidate[candidate.index(marker):]
print('PASS: solve_stitch exactly reuses candidate 35af source.')
print('PASS: removing only the new exact-zero/positive-compliance branch and normalizing its private lambda signature/index recovers the entire base constraints.rs byte-for-byte.')
print('Scope: all positive-target and hard radial projection arithmetic, duplicate merging, denominator/distance guards, shared project, membrane and bending code are unchanged.')

lib_path = SIM / 'lib.rs'
assert lib_path.read_bytes().replace(
    b'let mut seam_lambda = vec![[0.; 3]; self.input.stitches.len()];',
    b'let mut seam_lambda = vec![0.; self.input.stitches.len()];'
) == old(BASE, lib_path)
print('PASS: lib.rs differs from base only at the private seam-lambda allocation; reset, cancellation, transaction acceptance, integration and quality limits are byte-identical.')
for path in sorted(SIM.glob('*.rs')):
    if path.name not in {'constraints.rs', 'lib.rs', 'tests.rs'}:
        assert path.read_bytes() == old(BASE, path), path
        print(f'PASS: unchanged {path.name} sha256={hashlib.sha256(path.read_bytes()).hexdigest()}')

tests_path = SIM / 'tests.rs'
tests = tests_path.read_bytes()
start = tests.index(b'// This read-only adapter keeps the frozen N32 observer')
end = tests.index(b'#[test]\nfn analytic_dihedral_gradient_matches_independent_finite_difference()', start)
normalized = tests[:start] + tests[end:]
for name in (b'stitch', b's'):
    normalized = normalized.replace(
        b'constraints::solve_stitch(' + name + b', 0., &mut x, &[1.; 4], 0.01, &mut [0.; 3]);',
        b'constraints::solve_stitch(' + name + b', 0., &mut x, &[1.; 4], 0.01, &mut 0.);'
    )
assert normalized == old(BASE, tests_path)
print('PASS: all existing tests are byte-identical except the two necessary private lambda variables, including current canonical diagnostic and full capsule tests.')
new_block = tests[start:end]
added_start = new_block.index(b'#[test]\nfn closed_elastic_stitch_two_free_masses_preserve_reactions_under_rigid_transforms()')
added_end = new_block.index(b'#[test]\nfn closed_elastic_stitch_merges_repeated_and_cross_side_vertices()', added_start)
normalized = (new_block[:added_start] + new_block[added_end:]).replace(
    b'            d.capsule_surface_uncertain_pairs,\n', b''
)
original = old(CANDIDATE, tests_path)
original_start = original.index(b'// This read-only adapter keeps the frozen N32 observer')
original_end = original.index(b'#[test]\nfn analytic_dihedral_gradient_matches_independent_finite_difference()', original_start)
assert normalized == original[original_start:original_end]
print('PASS: all candidate analytic tests/tolerances are unchanged except inclusion of current capsule diagnostic state; one two-mass/rigid-transform fixture is added.')
changed = set(subprocess.check_output(['git', 'diff', BASE, '--name-only'], text=True).splitlines())
untracked = set(subprocess.check_output(['git', 'ls-files', '--others', '--exclude-standard'], text=True).splitlines())
assert changed | untracked == {str(SIM / n) for n in ['constraints.rs', 'lib.rs', 'tests.rs']} | {
    'docs/physics.md', 'spec/features/SIM-001-closed-stitch-coordinates.md'
}, (changed, untracked)
print('PASS: file ownership is limited to the three solver files, physics note, and prospective spec.')
