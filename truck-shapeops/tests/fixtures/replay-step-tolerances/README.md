# Fillet replay export tolerance

Original native OCCT 7.9.3 `blend/simple/D4` capture 001 from the clean
`2026-10-15-two-edge-face-trimming` corpus (kernel c26b6780). Both files preserve
native capture bytes. The case builds a radius-150 sphere sector, clipped at
latitudes +/-60 degrees and swept through 90 degrees, then applies radius 10.
The native result volume 3435781.0157892024 is a reference measurement, not an
independent analytic fillet oracle. The fitted STEP follows the exporter's
robust-tessellation contract. This fixture catches replay's accidental reduction
of fillet export tolerance from 5e-5 to 1e-7, which exceeded its fitting budget.

SHA-256:

- `D4.json`: `65a0d620b36c23d4c49faa67853e150711338811c7897b1b1dc81408cfbec3bf`
- `D4.step`: `e3216147c32cb505b53a58872bcd2bd46a02a59edbd60695f97381af300b1471`
