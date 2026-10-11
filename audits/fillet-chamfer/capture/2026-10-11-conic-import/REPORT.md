# Nonelliptic conic import replay

F8/F9/G1/G2 retain their native successes. Truck now converts their parabolas
correctly and proceeds to input-consistency validation. All four still fail there,
with finite extrusion support domains under investigation. No blend success or
full parity is assigned.

The actual parser and modeling conversion source hashes match `41e25f28`. The
runtime HEAD predates that commit because validation was performed first; dirty
files, actual kernel sources, executables and fixtures are recorded in the manifest.
Every listed artifact and the archive are hashed. Kernel tests and exact geometry
checks are documented in
`truck-benchmarks/results/2026-10-11-conic-import/REPORT.md`.
