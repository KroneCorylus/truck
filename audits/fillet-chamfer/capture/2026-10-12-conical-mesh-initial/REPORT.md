# Initial conical-mesh capture: STEP join regression

The full run at clean aa6cfa65 passes all 156 native entries. Its 161
captures comprise 72 operation-succeeded, 32 operation-failed, 25
input-import-failed, 23 input-inconsistent, eight unhandled intents and one
incomplete capture. There are no timeouts or panics. I3's mesh timeout is
resolved; all 72 constructed outputs preserve input and geometry consistency,
produce closed Truck meshes and pass Truck STEP preparation.

Native validation caught a regression missed by Truck's own STEP reader:
21 exported models are rejected because concatenated fitted cubic pieces
retain interior knots with multiplicity four. Removing the global-tolerance
optimization preserved tighter accuracy but also retained duplicate join
controls. Only 51 exports are accepted as valid single solids, and 50 meet
the combined limited sampled comparisons. This run is not an improvement
in overall interchange coverage and is retained as failure evidence.

The follow-up removes exactly duplicated join controls without changing
distinct endpoints or using an approximation tolerance. The strengthened
regression requires valid internal multiplicities as well as the original
position/derivative accuracy. A later clean capture records recovery.

All archived artifact hashes were checked. The manifest retains clean source,
binary, library and harness provenance. No full parity verdict is assigned.
