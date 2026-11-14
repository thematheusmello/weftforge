# Independent DXF writer and native export evidence

The saved outputs pass the bounded FILE-005-v1 profile through ordinary ezdxf 1.4.5 reading, zero audit errors and fixes, raw-tag structure checks, exact source-derived binary64 geometry, and independent B-spline evaluation. The two native outputs and the final CLI export of the actual saved native source are byte-identical (`ca94b0235ba944c0879b51b260b600ee3a10c6bd539112b213f6b59f9a719ca7`). The known rectangle measures 125 × 250 mm numerically. The empty source retains full required infrastructure with no model-space geometry.

The native saved snapshots prove the declared cancel/success/repeat/reopen consequences. A separately frozen supplement proves the specific pending-field Cancel case: the captured field remains 0.055 with 907 vertices and no Unsaved marker; the saved Redo consequence is revision 24/resolution 0.05. The independent image review is in `reports/cancel-draft-visual-review.json`. Action observations, process identities and their limits remain in the original parent logs. Neither a screenshot nor a saved document alone proves hidden history.

This evidence establishes no reference parity, other CAD consumer behavior, DXF import, apparel interchange, garment round trip, native write-failure/overwrite-warning behavior, or printing/printed scale. GitHub Actions and Pages remain disabled until the full project is done.

## Original evidence and provenance

Original freezes, evaluators, fixtures, expected documents, reports and saved exports are preserved byte for byte. `publication-provenance.json` maps their private workspace-relative sources and hashes. Four producer/wrapper log copies replace only the personal workspace prefix with the literal `${WORKSPACE}`; the original private hashes and derived public hashes are recorded. Candidate binaries, the private verification environment, wheels, SDK evidence and raw capture metadata are excluded.

The root capture owner must merge the original reviewed image at `captures/dxf-pending-cancel-20261114T135220Z.png` with SHA256 `7968db1c1df076df4c50ca3cee3d12c90430eb0db0653520ed983980b502eded`. It is an explicit external file in the manifest until that merge. The oracle publisher did not copy the capture subset. A complete replay requires this original image. Additional reviewed capture subsets live outside this oracle root and are registered separately.

`native/source-after-cancel.weftforge.autosave` is intentionally included because the original primary summary references its hash. It is original public-source JSON, byte-identical to `native/source-after.weftforge`; the alias is documented in provenance. It must be explicitly Git-tracked despite the repository ignore pattern. Other volatile scratch files are excluded.

`bundle-manifest.json` inventories every selected file and the single external image by SHA256 and size. The manifest excludes its own bytes to avoid a self-reference. Verify it before replay; root registry/map checks from a clean checkout establish tracked publication. Frozen criteria retain their original preparation command paths, so use the portable commands below without editing those criteria.

## Check the saved bundle

From the repository root, after the capture owner has merged the original image:

```sh
python3 parity/reports/artifacts/dxf-20261114/native/verify_bundle.py --require-git-tracked
```

For a standalone extracted bundle, omit `--require-git-tracked`. During preparation only, `--allow-missing-external` reports the outstanding capture and leaves `complete` false. A missing external image is never a complete delivery.

## Fresh independent consumer replay

Tested original environment: Python 3.13.0, ezdxf 1.4.5 from official PyPI, MIT license; exact distribution and resolved dependency versions are recorded in `reports/consumer.json`, `reports/pip-install.json` and `reports/ezdxf-license.txt`. These are verification dependencies only. A new platform may select a different wheel for the same pinned package version; it does not retroactively change the original distribution record.

From the repository root, create a fresh private environment and send new reports outside the immutable evidence tree. Invoke the venv interpreter directly; resolving its symlink can bypass the venv, which is the preserved initial wrapper failure in `headless/pre-ui-fix/oracle-run-results.public.json`.

```sh
task_bundle_root="$(pwd)/parity/reports/artifacts/dxf-20261114/native"
task_replay_dir="$(mktemp -d)"
python3.13 -m venv "$task_replay_dir/venv"
"$task_replay_dir/venv/bin/python" -m pip install --index-url https://pypi.org/simple -r "$task_bundle_root/requirements-verification.txt"
python3 "$task_bundle_root/verify_bundle.py"

for task_case in mixed mixed-placement-resolution rectangle empty native-expected; do
  "$task_replay_dir/venv/bin/python" "$task_bundle_root/evaluate.py" check \
    --source "$task_bundle_root/input/$task_case.weftforge" \
    --dxf "$task_bundle_root/headless/pre-ui-fix/$task_case.dxf" \
    --report "$task_replay_dir/headless-$task_case.json"
done

"$task_replay_dir/venv/bin/python" "$task_bundle_root/evaluate.py" check \
  --source "$task_bundle_root/input/mixed.weftforge" \
  --dxf "$task_bundle_root/headless/pre-ui-fix/mixed-repeat.dxf" \
  --report "$task_replay_dir/headless-repeat.json"

"$task_replay_dir/venv/bin/python" "$task_bundle_root/evaluate.py" native-snapshots \
  --directory "$task_bundle_root/native" --report "$task_replay_dir/native-snapshots.json"

for task_native_file in native-mixed native-mixed-repeat; do
  "$task_replay_dir/venv/bin/python" "$task_bundle_root/evaluate.py" check \
    --source "$task_bundle_root/native/source-after.weftforge" \
    --dxf "$task_bundle_root/native/$task_native_file.dxf" \
    --report "$task_replay_dir/$task_native_file.json"
done

"$task_replay_dir/venv/bin/python" "$task_bundle_root/evaluate.py" check \
  --source "$task_bundle_root/native/source-after.weftforge" \
  --dxf "$task_bundle_root/headless/final/from-native-snapshot.dxf" \
  --report "$task_replay_dir/final-cli-from-native.json"

python3 "$task_bundle_root/supplement-cancel-draft/evaluate_cancel.py" \
  --report "$task_replay_dir/cancel-draft.json"
python3 "$task_bundle_root/verify_bundle.py"
```

Each consumer report must have `accepted: true`, empty raw/consumer/audit error and audit fix lists, and unchanged original DXF bytes. The independent spline tolerance is 1e-8 mm per coordinate at the seven predeclared parameters; exact scaled binary64 controls and knots remain mandatory. Every source and output hash is recorded. A consumer failure on a new environment remains a failure; do not recover, save/rewrite the export or adjust the frozen oracle.

The original pretests can also be replayed to new temporary report paths using `evaluate.py self-test` and `supplement-cancel-draft/evaluate_cancel.py --self-test`. They include 34 primary and 11 supplemental negative witnesses, including the original precommit-on-Cancel/lost-Redo result.

Publication is complete only after all selected files, the original capture, and registry/map references are Git-tracked and verified from a clean checkout. Run the repository's required integrated checks there; this saved-output replay starts no candidate app, MCP server or Cargo build.
