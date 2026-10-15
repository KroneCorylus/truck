# Confirmed native rejection replay

Previously a native command rejection appeared as capture-incomplete because
there was no result JSON. The runner now requires a complete input JSON/STEP,
a successful input-export marker and native error return code 1. It replays that
input while retaining the native rejection as a separate native_result record.
No native shape comparison is attempted, even when Truck constructs successfully.
Missing logs, failed export, non-error Tcl returns, malformed JSON and missing
STEP input remain incomplete. No geometry, native plugin or Rust source changed.

Two new runner regressions fail on parent ab12f114; eight capture tests and
19 audit tests pass. Existing assertions and tolerances are unchanged. No speed
claim; native and Rust validation from the previous group is retained.

Real bugs/moddata_1/bug119 passes its native test because the radius-50 request
on a 100-cube is rejected. Truck independently rejects its four physical edges
with OutsideNeighbour and input_unchanged=true. This is rejection evidence, not
a recovered construction, geometric comparison or full-parity verdict.
The successful bug203 chamfer control retains the EXACT previous result and
sampled-comparison records. Both instrumented native entries pass.

Raw evidence: 27 byte-verified archive members.
Archive SHA256 79a0ab7b13a5cf905152e145e8ca87376ba3ed2a505bc5462d63f31b8ba136db.
metadata.json contains source hashes and both operation records. The raw manifest
inside the archive includes complete source/executable hashes and the tracked
proof-stage changes. A full clean capture follows the implementation commit.
