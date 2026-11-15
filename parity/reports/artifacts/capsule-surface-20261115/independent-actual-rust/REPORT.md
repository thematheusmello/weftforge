# Actual Rust extension of the independent static review

Actual unmodified `capsule::closest()` at commit
24dba7715233a7423088772d49c32a632d901a0c passed all83 frozen static cases.
All9 uncertain classifications match the earlier review. There were no failed
per-case bounds, convex-coordinate checks or uncertainty-agreement checks.
The original prospective tolerances were copied unchanged before execution.

This closes the earlier Python-transcription-only limitation for these83 static
inputs. It does not add response stepping, garment dynamics, convergence,
continuous contact, runtime performance or reference-parity evidence.

The prior case generator was reused verbatim from its retained source. Its full
inputs were frozen in inputs.json, and reconstructed transcription results were
asserted identical to every previous row before the Rust compile/run. Frozen
expected.json contains only the earlier independent100-digit quadratic oracle
distance, each original numerical tolerance and prior uncertainty flag.
METHOD.md, PLAN.json, source inputs and harness were hash-bound prospectively.

The tiny standalone harness declares Vec3 and includes byte-identical committed
math.rs and capsule.rs. It invokes only closest(), then emits one JSON line per
input. It changes no engine code and imports no Cargo dependencies. Pinned
compiler: rustc1.94.1(e408947bfd200af42db322daf0fadfe7e26d3bd1), host
aarch64-apple-darwin, LLVM21.1.8. Compiler executable SHA256:
b69cbcffa5e990057035d53ccc8163e8392ce784f6abb114c19ceb61528ba9f6.

Compilation command, run from this new directory with default unoptimized code
generation:

    rustup run 1.94.1 rustc --edition=2024 --crate-name capsule_static_check harness.rs -o capsule-static-check

Execution command:

    ./capsule-static-check

Exactly one compile and one static executable run completed, both exit0.
Compiler stdout/stderr and executable stderr are empty. Actual83-line stdout is
retained verbatim in actual.stdout. Measured compile/run wall intervals were
.808540416998/.243835415924s; these include subprocess overhead and are not a
solver or contact-performance benchmark.

The largest exact-case oracle difference divided by local scale is
1.5274253848741944e-16, at ordinary_44. Values need not be bit-identical to the
Python transcription: the largest absolute Rust/transcription difference is
3.814697265625e-6m in the synthetic1e10-scaled cases. Every actual Rust result
passes that case's original scale-dependent tolerance; no tolerance was changed.
The uncertainty lower-bound checks and exact-case absolute-error checks remain
distinct, as originally declared. Full actual rows, oracle values, original
tolerances and pass/failure arrays are in RESULT.json.

Artifact SHA256 bindings:

| Artifact | SHA256 |
| --- | --- |
| Exact committed math.rs | 4f344da57b5bcb7fb994bb67ae40f3909b7e23814969971dbe457c9094d1b424 |
| Exact committed capsule.rs | 67bed0887b2a0644dbab7c7003228427c0f9540e5d09a1170961f46cd14e9de7 |
| harness.rs | ed15519d991bdc7c160f7551165e7703533f7804aa783f713825836d54a3759e |
| capsule-static-check executable | 73c0178e95f9aa9032a9a6658d84591580abe6ce5ac3a53f89d57e353aac6031 |
| actual.stdout | ea523d2287b248ea925c8fc08df93f87f1a18f9e3d0964f8574001a1beda05b5 |
| inputs.json | 6ab7b275df6bc0966feeda0538c9e88cc08cea81b7e1f19238b9042cb0790121 |
| expected.json | 39365c2f42c286cb7ccd565fbc34ea7c2c9cf914de2b67935808c70da136697e |
| PLAN.json | a3e2f0304ce6dc810550ac8f4ca24f59ee5e07eb5eefcc2e2d5cc39ce8a09bcb |
| RESULT.json | 87492f50e8f369ee7c6cd7752fdbfa9743c0fa62224a16229fd55f854a28788d |

Committed module bytes, include copies, harness, input/expected files and pinned
compiler executable were verified before/after. The old9-file independent review
remains immutable, with MANIFEST SHA256
1041f3bcee5c3c313c72c590b455fc124bcba2ef268251ccef5e406c91ca4e8a.
This new directory is private ignored analysis, not published delivery.

Skipped: Cargo/workspace tests, projection/response execution, garment dynamics,
UI and publication.
Risk:83 static closest-feature cases cannot establish full solver behavior,
whole-garment convergence or reference parity.
