# G2 after validated F5 producer

Scratch preparation only; no Cargo or checkout edit. The running checkout's planar_fillet.rs is byte-identical to frozen F5 final-kernel.rs (af6c0b5132d7a129e5f354f0d394507171ad1f7ea5b248895a1023346361cfc8), so the incremental G2 kernel patch needs no contextual rebase. Both patches passed git apply --check against the current isolated checkout. No full kernel copy is required or proposed.

kernel.patch changes only truck-shapeops/src/fillet/planar_fillet.rs. It retains the existing F5 physical parameter chart, derivative chain, finite guards and contact_curve_plane domain correction. It adds only the remote two-selected-edge branch and its helpers, fitted-end handling and deferred shared intersection edges. No planar.rs, mesher, existing test or benchmark is changed. regressions.patch adds one procedural test and two independent oracle files; its documented invocation now names the installed oracle source.

Eligibility remains a single clamped quadratic Bezier extrusion, reflection symmetry about the chord midpoint, planar cap parallel to the generating plane, extrusion perpendicular to chord, nonfolding offset, bounded original slab/ruling, and E-first angular branch. G6 quartic/H1 cubic are excluded. Sampled branch witnesses are not a continuous uniqueness certificate. Native imported G2 representation remains unverified. Upstream projection evaluator behavior still depends on D's contract.

## Execution after current validation finishes

1. Apply regressions.patch only and compile the new test with default features (STEP enabled).
2. Run the unchanged positive test against F5 without G2; record construction rejection as parent proof. A compilation/fixture failure is not parent proof. Preserve source and library hashes.
3. Apply kernel.patch. Compile then run extrusion_remote_junction unchanged. A separate radius .5 identity/order0 diagnostic may precede the full run, but cannot replace it. All 24 positive operations and six immutable rejection operations must pass, including the exact material/STEP thresholds.
4. Run existing extrusion/fillet controls and full shapeops/replay validation. Do not broaden basis domains or weaken oracle assertions to force a result. Claim native G2 support only after its captured input is separately tested.

Commands (from isolated checkout):

    RAYON_NUM_THREADS=1 CARGO_BUILD_JOBS=12 cargo test -p truck-shapeops --test extrusion_remote_junction --no-run
    RAYON_NUM_THREADS=1 CARGO_BUILD_JOBS=12 cargo test -p truck-shapeops --test extrusion_remote_junction -- --nocapture

## Archived F5 integration recipe and scope proof

The earlier package target/b-f5-producer-integration-2026-10-24 contains producer.patch, tests-native.patch and provenance.json. At preparation, both planar source files in checkout5a4e3bc6 were byte-identical to 9e3ce08a. producer.patch was generated from those exact files to the independently hashed planar.rs.candidate and final-kernel.rs. It touches only those two files. tests-native.patch adds the unchanged paused procedural regression, native F5 STEP/intent/README, and appends only the native extrusion test module to the existing replay example; paused procedural/example sources equal original staged sources. Both scoped patches passed git apply --check before coordinator applied them.

The coordinator subsequently reports producer/native tests passing with the finite-triangle/bulk mesher. No such runtime claim is inferred from patch validation. Current mesher, direct-firstpass cap test and finest-loft test are coordinator-owned and absent from these G2 patches. Old full F5 integration and midpoint-and-accuracy patches must not be replayed: they contain superseded mesher/tests. The normalized-chart fixture defect remains unresolved. Physical chart success does not erase it. Independent F5 material source/method/controls remain target/extrusion-corner-volume.cpp, target/extrusion-corner-volume-method.txt, target/extrusion-corner-volume-controls.txt.
