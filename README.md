# truck

A B-rep CAD kernel in Rust, forked from [ricosjp/truck](https://github.com/ricosjp/truck) at
`efe39930` (2026-08-31). This fork has one consumer, our parametric CAD application, and
changes for it. We rewrite parts and change APIs whenever that makes the kernel faster,
simpler or more robust. It does not track upstream.

[AGENTS.md](AGENTS.md) holds the priorities and rules for working here. Read it before
changing anything.

## Crates

| Crate | Contents |
| --- | --- |
| `truck-base` | Basic structs and traits: cgmath, curve and surface traits, tolerance, diagnostics |
| `truck-geotrait` | Geometric traits: `ParametricCurve`, `ParametricSurface`, and so on |
| `truck-geometry` | Knot vectors, B-splines, NURBS and derived geometry |
| `truck-topology` | Vertices, edges, wires, faces, shells and solids |
| `truck-modeling` | Modeling builders and the curve and surface enums |
| `truck-shapeops` | Booleans, fillets, chamfers and local operations |
| `truck-meshalgo` | Tessellation and mesh algorithms |
| `truck-polymesh` | Polygon mesh data structure |
| `truck-stepio` | STEP import and export |
| `truck-hlr` | Hidden-line projection of a solid onto a plane |
| `truck-drafting` | 2D drafting on truck geometry and topology |
| `truck-assembly` | Assembly structures |
| `truck-platform` | Graphics utilities on wgpu |
| `truck-rendimpl` | Rendering of shapes and meshes on `truck-platform` |
| `truck-js` | WebAssembly bindings |
| `truck-derivers` | Derive macros |
| `truck-benchmarks` | CPU benchmarks of public operations |

## Documentation

- [DIAGNOSTICS.md](DIAGNOSTICS.md): error codes and the `try_*` APIs.
- [BENCHMARKS.md](BENCHMARKS.md): how to measure performance, and reference numbers.
- [CHANGELOG.md](CHANGELOG.md): every change, with breaking changes marked.
- Crate docs: `cargo doc --open -p <crate>`.

## Building and testing

```bash
git submodule update --init   # STEP fixtures in resources/
cargo test -p truck-shapeops
```

## License

Apache License 2.0, inherited from upstream. See [LICENSE](LICENSE).
