# truck-stepio

[![Crates.io](https://img.shields.io/crates/v/truck-stepio.svg)](https://crates.io/crates/truck-stepio) [![Docs.rs](https://docs.rs/truck-stepio/badge.svg)](https://docs.rs/truck-stepio)

Reads/writes STEP files from/to truck.

## Sample Codes

### shape-to-step

convert from truck shape json to step file.

#### usage

```bash
shape-to-step <input shape file> [output shape file]
```

### step-to-mesh

Parse STEP data, extract shape, and meshing.

## Test fixture provenance

The `resources/step/occt-*.step` files were written with FreeCAD (Open CASCADE STEP
processor 7.8) for testing truck. `occt-assy.step` is an assembly in AP203; the others
are single solids in AP214. Other fixture sources are documented in `resources/Readme.md`.
