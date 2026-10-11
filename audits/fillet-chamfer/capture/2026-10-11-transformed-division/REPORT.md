# Transformed division preprocessing replay

All five instrumented native tasks remain successful. F8, F9, G1 and G2 now
return an unsupported-conic import error instead of the small-local-tolerance
panic. Z1 proceeds further and exposes an odd-divisor index error in closed-face
splitting. These results remain failures before Truck blending, not parity passes.

This replay validated the sampling prototype before commit `dc055215`. The exact
runtime HEAD, dirty files, actual kernel sources, replay/native executables and
artifacts are hashed in the manifest. Final validation added documentation,
expanded tests, the exact-chord underflow guard and the existing Clippy test-helper
update. Its report is
`truck-benchmarks/results/2026-10-11-transformed-division/REPORT.md`.

The original baseline is retained. The archive contains every manifest-listed
artifact, including the native physical fixtures and failure-stage records.
