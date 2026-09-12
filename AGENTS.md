# Agent guide

## What this repository is

This is our fork of [ricosjp/truck](https://github.com/ricosjp/truck), and it has exactly one
consumer: our parametric CAD application, `test_cad` (`~/Development/test_cad`). It is not a
general-purpose library. There are no outside users to protect, so compatibility only matters
to the extent our app needs it.

The app pins a commit of this repository in its workspace `Cargo.toml`. Its core uses
`truck-modeling`, `truck-shapeops`, `truck-meshalgo`, `truck-polymesh` and `truck-stepio`.
Its UI renders with `truck-platform` and `truck-rendimpl`. `truck-hlr` was built for the
app's 2D drawings but is not wired in yet. The app rebuilds the whole B-rep from scratch on
every edit, so **kernel latency is UI latency**. When the kernel cannot do something, the app
reports the feature as failed. A missing or fragile capability is a broken product feature.

## Priorities

1. **Stable.** Give the same result for the same input. Never panic on input a user can
   produce. Failures return `None` or a `Diagnostic` (see `DIAGNOSTICS.md`). Only a violated
   internal invariant may panic.
2. **Fast.** Operations should stay interactive on real parts. Treat regressions in
   `truck-benchmarks` as bugs.
3. **Simple.** Aim for the smallest model that behaves correctly. Fewer code paths mean fewer
   places for bugs and slow cases to hide.

## You may rewrite and change the API

- Rewriting a module or a whole pipeline is welcome when it makes things faster, simpler or
  more robust. Existing structure is not correct just because it exists, and upstream's
  design is not a constraint. Do not add machinery to work around a design that should
  change.
- Public APIs may change when you find a better shape. Remove the old API instead of keeping
  deprecated aliases or compatibility wrappers. Before changing an API, search the app for
  its call sites so the migration note covers every use.
- Document every breaking change as a `CHANGELOG.md` line under `## Unreleased`, starting
  with `Breaking:`. Name the old API, the new API and how to migrate. Update any doc the change
  touches, such as `DIAGNOSTICS.md` or the crate docs.
- Upstream is where this code came from, not something we track. We do not merge from it,
  contribute to it, or keep our code compatible with it. If it fixes something we need, port
  the fix like any other change.
- For a large rewrite, first describe the new design and how you will prove it equivalent.
  Then build it.

## What does not bend

- **Tests hold behavior.** Every fix or new capability comes with a test that fails on the
  parent commit. A rewrite that keeps an API must pass that API's existing tests unchanged.
- **Never delete, ignore or loosen a test to make a change pass.** Sometimes an API change or
  an intended change in results means a test no longer fits. Then rewrite the test first, so
  it states the new contract and checks at least as much as before. After that, change the
  code until the test passes, and say in the change why the test changed.
- Check solids with the harness in `truck-shapeops/tests/common/mod.rs`: `assert_solid`
  checks topology, a closed tessellation, and signed volume against an exact value.
- **Performance claims come with numbers.** Measure before and after with
  `truck-benchmarks` (see `BENCHMARKS.md`) and report the median. Speed must not cost
  correctness: identical geometry, or an explained and tested difference.
- **Diagnostic codes are part of the API.** The app matches on them. Changing a code's
  meaning is a breaking change, covered by the rules above.
- **Dependencies need a justification.** Prefer fewer of them. A dependency is welcome
  when it makes the kernel measurably faster or more robust and is known to be stable and
  reliable: a mature math crate, or a proven C or Fortran library such as LAPACK. Justify
  it in the change with:
  - benchmark numbers showing the gain
  - its track record
  - a license compatible with Apache 2.0
  - proof that it builds on every platform the app builds for

  Keep it behind our own API so it can be replaced.
- **No `unsafe`, except at a foreign-library boundary.** Bindings to a C or Fortran library
  may use `unsafe` inside one small wrapper module that exposes a safe API. No other code
  uses `unsafe`.
- The crates use `#![deny(clippy::all, rust_2018_idioms)]` and deny warnings in release
  builds. Keep Clippy clean and format with the workspace `rustfmt.toml`.

## Working in the repo

```bash
git submodule update --init                      # STEP fixtures in resources/
cargo test -p truck-shapeops                     # test one crate (preferred while iterating)
cargo clippy -p truck-shapeops --all-targets     # lint one crate
RAYON_NUM_THREADS=1 cargo bench -p truck-benchmarks --bench actions -- --test   # check benchmarks still run
RAYON_NUM_THREADS=1 cargo bench -p truck-benchmarks --bench actions             # measure
```

- `truck-platform` and `truck-rendimpl` are on the app's display path, and their tests need a
  GPU (`cargo make gpu-test`). The app does not use `truck-js`: keep it compiling, but do not
  invest in it.
- Unit tests go next to the code in `mod tests`. Integration tests go in `<crate>/tests/`.
- Keep comments to a minimum: add one only when the code cannot make the point itself. Match
  the style of the surrounding code.

## Local-only directories (gitignored)

- `kernel-issues/` holds capability requests from the app. They describe a problem to
  solve, not a design to copy. `kernel-issues/responses/` records the agreed shape and
  scope, so read the response before working an issue. If the code contradicts it, update
  the response. If an issue or response sets other ground rules, this file wins.
- `WIP/` holds working plans and notes, such as `HOWTO-freecad.md` for checking STEP output
  in FreeCAD. Delete a plan once its work has landed.

## Commits

- Do not commit, push or open pull requests unless explicitly asked.
