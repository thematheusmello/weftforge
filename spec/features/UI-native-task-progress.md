# Native task progress

Prospective implementation-only contract, written before source changes on
2026-11-15. This is original WeftForge behavior; no native observation or
reference parity is claimed.

Each mesh or simulation worker receives a monotonically allocated application
task ID when it is created. The ID identifies a worker in the current process
session, travels with its input document revision and job kind, and is neither
an operating-system thread handle nor a persisted document ID. Queued phases
have no task ID until they start. Opening or resetting a document does not restart
the session ID sequence.

A simulation worker starts at zero successful steps. Its atomic counter advances
once only after the existing transactional solver step returns success. Cloth
initialization, attempted/failed/cancelled steps, substeps and solver iterations
do not advance that counter. The fixed batch step count is its total. This is
completed worker work, not accepted display cloth, elapsed-time percentage,
convergence, garment quality or estimated time remaining. Mesh work has no
invented total or percentage and remains explicitly indeterminate.

The existing simulation transport shows the active phase and successful-step
count over its total, using the existing repaint and cancellation paths. An
optional, initially closed Task details disclosure shows the running task's ID,
kind and input revision and the most recently terminated task with its terminal
status. These are session diagnostics, not network telemetry or file logging.

Only a successful result from an uncancelled task at the current document
revision is Accepted. Cancelled, Failed and Discarded (obsolete input revision)
are distinct terminal statuses; none is described as accepted or completed.
Even a full successful-step count cannot make an unaccepted result Accepted.
Cancellation takes precedence when a task is also obsolete. The last terminal
record survives queued phases, new mesh scheduling, construction edits, clearing,
document load and Reset. It is replaced only when another task terminates.

Worker failure, cancellation and stale results retain the previous accepted
cloth and mesh. Existing explicit Reset/document-clear behavior still clears
simulation state, and accepted replacement meshing still resets cloth. No step
count, settings, forces, ordering, geometry, document schema, persistence,
core/solver API, automation API or dependency changes belong to this increment.

One focused deterministic native implementation test will run real workers and
check IDs/revisions, successful-step counts, terminal classification, retention
of accepted snapshots after cancellation/staleness/failure, and retained
correlation across Reset/edit/mesh scheduling. Existing layout and lifecycle
checks remain applicable. These tests do not establish actual window usability,
physical garment acceptance or reference parity.
