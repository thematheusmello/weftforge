# Historical GUI export regression logs

These are existing captured headless test outputs, extracted directly from the
investigator's retained command-execution records. No tests were rerun and no
native application was operated while preparing this supplement.

`red-export-cancel.log` records the original Cancel defect: revision 4 became 5,
width 0.4 m became 0.45 m and resolution 0.07 m became 0.08 m before any successful
export. It ran the 5f4124c production ordering with an uncommitted regression and
a test-only chooser hook. It is not evidence that the old packaged native app
was launched. The first failed assertion stopped the remaining planned cases.
No frozen instrumented-source or test-executable digest was recorded.

`green-app-tests.log` records all 23 passing app tests on the completed working
tree subsequently committed as 409d845. It includes the 55-case actual App::ui
regression and the ready-rest/current-drape/stale-mesh OBJ check. The 55 cases
are a matrix inside one Rust test, not 55 independently named tests. Formatting
and strict workspace Clippy also passed. Source attribution, commands, times,
exit codes, original private record/text hashes and public hashes are explicit
in `execution-provenance.json`.

Only exact absolute workspace and dedicated Cargo target prefixes are replaced
with literal `${WORKSPACE}` and `${CARGO_TARGET_DIR}`. All other log bytes are
preserved. `green-fmt.log` is empty because the successful check emitted nothing;
its exit code and execution record are in the provenance. Use `manifest.json`
for public file checksums. This is historical implementation evidence; it does
not alter the later frozen native acceptance or establish reference parity.
