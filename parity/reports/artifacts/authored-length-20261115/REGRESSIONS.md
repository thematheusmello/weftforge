# Retained corrections and scope

The a820 implementation passed focused checks and the independent analytic CLI/MCP case. Source review then found native feedback disappeared from layout when clipped. An added headless egui scrolling test failed with content height 2198→2290px before the fix. The log is named for metadata 7976; that added test was not committed in 7976, so no pre-edit committed-test freeze is claimed. The product code under test was a820 plus the added regression.

At be8, feedback reserves three stable A/B/B−A rows in all states. Only uncached, visible or partially clipped blocks compute metrics. The same scrolling check holds 3787→3787→3787px and computes 2 of 25 pairs; long errors keep that footprint. Independent source review resolves the finding. Full 205 Rust / 54 Python gates follow.

Initial new-test mistakes are retained privately: exact equality was incorrectly used between independently budgeted group/interval integrals and decimal endpoints. Corrections use the already frozen 1e-8 m criterion; no production tolerance or independent expected value changed. Native AX text helpers initially counted duplicate parent/inline text and were corrected to Label nodes. An unstable test Cell API was replaced with stable get/set. These were test/API corrections, not claimed product defects or reference failures.

Native 001 stopped permanently when its recorder used an unavailable global require before input or screenshots. Native 002 keeps the same values, fixture and caps, but loads the same recorder via createRequire after offline self-check. Its screenshots are actual native output. Neither read-only case performs authoring, numeric input, Save, history, physics or export. Full trace/completion compliance remains actor reported.

The reference display observation is separately reviewed in spec/features/PAT-007-line-length-runtime-20261115.md. Unselected canvas labels and Select All aggregates are observed there; our three-decimal selected inspector is a different partial presentation. Exact arc/rounding/angle rules, full native authoring and reference parity remain unknown.
