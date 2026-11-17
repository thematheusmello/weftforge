What this change does: The corrected external supervisor now measures final output volume after ROOT-RESULT persistence, then samples its final clock. Its terminal acceptance uses those measured values and reports the final volume.

R1 is resolved in supervisor SHA256 b62b1d86c4b05a54bc37e9a20d49822ddab4374a9abddf67901913e4be35afbf. The exact delta adds the final volume sample before the clock, uses that sample in the gate and records it in stdout. Static AST checks confirm this ordering; no helper or supervisor was imported or executed by this review. The original R1 review/receipt and historical PREPARATION remain unchanged, as does the old supervisor/case.

No remaining blockers found to root issuing ONE exact13-key numerical GO for this frozen case. The final61-file freeze, five repository bindings, protocol, preflight, build receipt and actual own producer hash remain exact. No executable packet source changed, no ROOT-GO exists and no numerical/output/root-witness namespace has started. The earlier19-check source review and nine final binding checks remain the review basis; they were not repeated as a new source test cycle.

Root must bind the GO to freeze5ad83bba74240553cee9d7b7e5d05ce82ee9e748e4b084f14ab9d02fdb9f823c, source9d4b629c077f06c348f6615fe90df7bbeacedb24, build binding461cb838c010a05d5d93555be1f3820151db6f82c251674619046246c858b585 and experiment WF-20261117-MATERIAL-POLLING-HARBOR-A8-001; the exact schema requires base only, one init-only initialization with zero steps, one fresh actual-run initialization/dynamics run, zero retries and zero continuations. Use only the corrected supervisor above. This review itself authorizes no execution.

The420-second acceptance point now follows controller completion, receipt/hash guards, persisted root witness, persisted ROOT-RESULT and final output-volume traversal. Retain final stdout plus the process exit:0 means operational completion and strict quality pass;2 means operational completion with strict quality failure;1 means operational rejection. Pending/persisted files alone cannot establish final acceptance. The original181-state physical/assembly screens and256MiB numerical caps remain unchanged; no prefix or old-case regrade is allowed.

Verdict: Ready for root's one exact13-key numerical GO under the unchanged scope.

Not checked: No execution, packet imports, model/init/step/analysis, build/test, UI, network or push occurred. G002 closure and idle scheduling are root-provided state.
Risk: Actual full-case timing and strict quality are still unproven; this is no garment/native/reference acceptance. Metadata freshness/xtask and linker/SDK/sysroot qualification remain pending.
