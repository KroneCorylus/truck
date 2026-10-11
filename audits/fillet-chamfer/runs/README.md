# Native OCCT corpus execution

All 1,026 inventoried entries were attempted with installed OCCT 7.9.3 DRAW,
using the pinned checkout's unchanged Tcl harness and tests. The runtime is the
installed binary, not a build of the locally modified OCCT checkout. Manifests
record binary/library hashes, harness hashes, case/helper/parse-rule hashes,
runtime configuration, process status, timeout and wall time.

| Upstream harness outcome | Entries |
| --- | ---: |
| OK | 295 |
| BAD (known problem) | 11 |
| FAILED | 19 |
| SKIPPED (missing data) | 701 |
| Crash / timeout / harness startup error | 0 |

The available-fixture selection contains 313 entries: 292 OK, eight BAD, 13 FAILED.
The 713 entries with missing static references contain three OK, three BAD, six
FAILED and 701 SKIPPED. Static dependencies do not determine runtime outcomes:
some references are conditional or belong to unused rendering backends.

All 19 FAILED outcomes are in rendering/viewer suites. The logs report unavailable
GLES drivers, missing image support or fallback fonts, and an unset rendering
resource environment variable. They are retained as failures, not relabeled as
successful blends. BAD is OCCT's TODO-aware harness verdict, not proof that the
requested blend works; some TODO patterns also match incomplete fixture setup.
Likewise, OK proves only that the upstream checks and their rule set passed. It
does not independently certify dimensions, validity or a Truck equivalent.

Every case-level full Truck parity verdict remains unverified. Captured physical
replays now cover operations from 148 upstream entries; their import, construction,
mesh and sampled STEP observations are attached separately. See
`../capture/baseline/REPORT.md`. Independent paired probes supply additional evidence. Missing fixtures remain needed for
701 native cases, including all 279 dedicated chamfer entries. There is ample
primitive-based coverage to continue pairing and implementation meanwhile.

## Reproduce

From the Truck repository, with DRAWEXE, sway, Xwayland and the local OCCT SDK:

```sh
python3 audits/fillet-chamfer/run_occt.py \
  --occt ../investigations/sequential-holes-2026-10-10/occt \
  --library-path target/occt-sdk/usr/lib64 \
  --output target/occt-new-run --workers 2 --timeout 120
```

The default selects cases without known missing static references. Add
`--include-missing` to attempt the entire corpus, or repeat `--case 'group/grid/glob'`
to select entries. Existing output directories are rejected. Each case gets its
own DRAW process and working directory. A separate headless compositor provides
off-screen Xwayland views; neither the user's display nor its control socket is
inherited. Upstream screenshot and geometry checks run unchanged. A timeout kills
the case's process group. The runner does not modify source checkouts.

These recorded runs used two workers and a 120-second per-process timeout. Wall
times include process startup, rendering and contention; they are not comparative
kernel performance measurements. The initial batch-mode smoke failed at its
screenshot step; switching to an isolated virtual viewer resolved that failure
without removing or replacing any test command.

## Evidence and catalog integration

`2026-10-11-available` and `2026-10-11-missing` contain manifests and compressed raw
artifacts. Each archive stores the original console, DRAW log and invocation Tcl
script. Images and intermediate models remain under the corresponding local
`target/occt-audit-*` directory and are not archived. For example:

```sh
unzip -p audits/fillet-chamfer/runs/2026-10-11-available/artifacts.zip \
  bugs/modalg_7/bug28828_8/draw.log
```

`build_catalog.py` checks source and artifact hashes before attaching results.
Native runs cannot set Truck execution or parity fields. Repeated runs retain
history; directories sort chronologically and the last result supplies the displayed
native status. The catalog still shows fixture inventory separately from runtime
outcomes. The runner/parser and generator have 16 passing unit tests, including
known-failure versus pass, exit-code/timeout precedence, stale sources, missing or
changed logs, archived evidence and rejection of native-to-parity promotion.
