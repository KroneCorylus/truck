# Operation diagnostics

Use the diagnostic APIs when a caller needs to distinguish failure conditions. Most
operations also have a form without `try_` that returns `Option` and discards the
diagnostic. The modeling, topology, geometry, drafting and mesh error enums expose `code()`.
Import `truck_base::diagnostics::CodedError` for a common error-code interface.

## Rust

```rust
use truck_base::diagnostics::Code;
use truck_shapeops::try_subtract;

// target, cutter and tolerance are supplied by the application.
match try_subtract(&target, &cutter, tolerance) {
    Ok(solid) => accept(solid),
    Err(error) => match error.code {
        Code::InvalidTolerance => request_valid_tolerance(error.context.value.as_deref()),
        Code::InvalidInputTopology => reject_operand(error.context.operand),
        _ => report_error(error),
    },
}
```

`Diagnostic` implements `std::error::Error` and `serde::Serialize`. Its fields are:

| Field | Meaning |
| --- | --- |
| `code` | Typed `Code` in Rust; stable `TRUCK_...` string when serialized |
| `operation` | Public operation, such as `subtract` or `fillet_solid_along_wire` |
| `stage` | Where the failure was observed, such as `validate_input`, `find_contact`, or `trim_faces` |
| `message` | Human explanation; never parse it for application logic |
| `context` | Optional operand, shell, face, edge, selection, dependency and parameter locations |
| `cause` | Optional immediate underlying error code and message |

Original underlying Rust errors are retained through `Error::source()` where available.
Third-party errors without a stable code still retain their explanation and Rust source.

Codes are explicit constants, independent of enum ordinals, display text and debug output.
The app matches on them, so changing what a code means is a breaking change (see
[AGENTS.md](AGENTS.md)). New codes and context fields can be added;
`Code` is non-exhaustive, and consumers must provide a fallback. Stages describe implementation
progress and may become more precise; branch on codes and relevant typed context instead.

Locations are zero-based and refer to the current invocation, not persistent topology names.
`face_index` is within `shell_index` when supplied; otherwise it is in the flattened face
traversal. Face-pair errors identify `operand`, `face_index`, `related_operand` and
`related_index`. Generated topology errors describe the failed construction stage; they do
not invent an input face when correspondence is unavailable. STEP entity IDs serialize as
decimal **strings** to preserve all 64 bits in JavaScript. Parameter values serialize as text
so that `NaN` and infinities survive reporting. Raw pointer-based topology IDs are not used
as serialized entity references.

Geometry reports `TRUCK_GEOMETRY_NON_FINITE_KNOT_ARITHMETIC` when knot domains,
span reciprocals, first/second derivative coefficient bounds, or calculated basis
coefficients are non-finite. Use a representable parameter chart; `KnotVec::try_normalize` can normalize tiny finite domains before
spline construction. `TRUCK_GEOMETRY_COLLAPSED_KNOT_SPAN` reports normalization
that would merge distinct knots; failed normalization leaves its input unchanged.
`TRUCK_GEOMETRY_ZERO_RANGE` remains the zero-range condition.
The fallible basis API rejects non-finite parameters and computed basis coefficients.
The infallible basis evaluator preserves IEEE arithmetic results, including NaN or infinity
from non-finite parameters or overflowing extrapolation; it still panics on zero-range or
insufficient-knot structural errors. It does not convert arithmetic overflow into a new
exception inside generic infallible curve/surface evaluation. Call the fallible basis API
when arithmetic diagnostics are required, and validate computed geometry at solver or
operation boundaries. Constructor derivative bounds use degree-wide supports
and define a conservative supported-domain policy; a failed bound is not proof that
an exact derivative is unrepresentable. Cuts, insertion, unchecked construction and
parameter transforms do not revalidate these constructor bounds. This does not provide
universal panic-free spline evaluation or finite arbitrary-order derivatives and
computations with extreme control point values.

## APIs

| Area | Diagnostic API |
| --- | --- |
| Booleans | `try_and`, `try_or`, `try_subtract`, `try_subtract_with_effect`, `try_solid_components` |
| Modeling blends | `try_fillet_solid_edges`, `try_fillet_solid_edges_with_radii`, `try_fillet_solid_along_wire`, `try_chamfer_solid_edge`, `try_chamfer_solid_along_wire`, `try_chamfer_solid_along_wire_chord`, `try_chamfer_solid_along_wire_angle` |
| Generic shell blends | `try_fillet_edges`, `try_fillet_along_wire`, `try_chamfer_along_wire` |
| Local operations | `try_move_faces`, `try_offset_faces`, `try_replace_surfaces`, `try_delete_face`, `try_draft`, `try_shell`, `try_thicken`, `try_intersect_surfaces` |
| Meshing | `triangulation_with_diagnostics`, `robust_triangulation_with_diagnostics`, `try_triangulation` |
| STEP | `Table::from_step_with_diagnostics`, `record_diagnostics`, `to_compressed_shell_with_diagnostics`, `to_compressed_solid_with_diagnostics` |

Meshing and STEP use `Report<T> { value, diagnostics }`. `is_complete()` means no omissions
were reported, not that geometry is globally valid or a mesh is watertight.
`into_complete()` returns the value only when there are no omissions; otherwise it returns
all diagnostics. `try_triangulation` is a convenience that rejects incomplete tessellation
with the first failed face. Use the report API to inspect all omitted faces.

Keep STEP parsing diagnostics and conversion diagnostics: an unreadable record anywhere in
the table and an omitted face in a selected shell are different observations. Unsupported
entity counts remain available through `Table::unsupported()`. `Table::errors`,
`Converted<T>` and `Skipped` carry the same observations without codes; `Skipped::code()` and
`into_diagnostic()` expose coded reasons without matching text.

An empty intersection is success. A disjoint or contact-only subtraction can succeed with
`removed_material: false`. An empty surface-intersection list is also success. Calculation
failure is a separate error. No diagnostic API silently changes tolerance, retries an
operation, repairs geometry, or replaces a failed result with the input.

## JavaScript / WebAssembly

The `try_*` bindings return the usual value on success and throw a native JavaScript `Error`
with the diagnostic fields on failure.

```js
try {
  const result = Truck.try_subtract(target, cutter, tolerance);
  useSolid(result);
} catch (error) {
  switch (error.code) {
    case "TRUCK_INVALID_TOLERANCE":
      showParameterError(error.context.parameter, error.context.value);
      break;
    default:
      showOperationError(error.code, error.message);
  }
}

const table = Truck.Table.try_from_step(stepText);
const recordIssues = table.diagnostics();
const imported = table.get_shape_with_diagnostics(table.shell_indices()[0]);
if (!imported.is_complete()) showImportIssues(imported.diagnostics());
```

Available diagnostic bindings include booleans; `try_line`, `try_circle_arc`, `try_bezier`,
validated transforms and sweeps; `attach_plane_with_diagnostics`; shape `try_from_json`,
`try_into_solid`, `try_to_polygon`, `to_polygon_with_diagnostics`; OBJ/STL `try_*` codecs;
and STEP parsing/conversion reports. `MeshReport.mesh()` and `StepShapeReport.shape()` expose
the partial or complete value. The report's `diagnostics()` method returns plain objects.

Run the Node integration tests against generated Node bindings:

```sh
RUSTFLAGS='--cfg=getrandom_backend="wasm_js"' cargo build -p truck-js --target wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/debug/truck_js.wasm --target nodejs --out-dir /tmp/truck-wasm-pkg
TRUCK_WASM_PKG=/tmp/truck-wasm-pkg/truck_js.js node --test truck-js/tests/diagnostics.test.mjs
```

## Scope and interpretation

These diagnostics preserve observed failures. They do not add an exact geometric validity
checker or expand the supported geometry of an operation. Some numerical helpers still
return only `Option`; their callers report a construction/projection/division failure and
its stage rather than claiming an unproven mathematical cause. For example, failure to lift
an intersection is not reported as proof that surfaces do not meet. Only diagnostics with
an identified source include a `cause`.

Low-level optional queries, explicitly panicking constructors and the `Option` forms of
operations have their own documented contracts. Application callbacks and custom curve/surface implementations
must still satisfy their documented contracts; diagnostic APIs do not catch arbitrary panics.

## Operation code catalog

| Code | Observed condition |
| --- | --- |
| `TRUCK_INVALID_TOLERANCE` | The tolerance must be finite and positive. |
| `TRUCK_INVALID_PARAMETER` | The parameter is invalid. |
| `TRUCK_SELECTION_UNKNOWN_EDGE` | The selected edge does not belong to the input. |
| `TRUCK_SELECTION_UNKNOWN_FACE` | The selected face does not belong to the input. |
| `TRUCK_SELECTION_EMPTY` | This operation requires a nonempty selection. |
| `TRUCK_SELECTION_DUPLICATE` | The selection contains duplicate entities. |
| `TRUCK_SELECTION_MULTIPLE_SHELLS` | The selected edges must belong to one shell. |
| `TRUCK_INVALID_INPUT_TOPOLOGY` | The input topology is invalid. |
| `TRUCK_INVALID_OUTPUT_TOPOLOGY` | The constructed topology is invalid. |
| `TRUCK_TESSELLATION_FAILED` | A face could not be tessellated. |
| `TRUCK_DEGENERATE_MESH` | The tessellation has zero or nonfinite signed volume. |
| `TRUCK_SHELLS_INTERSECT` | The sampled shell boundaries intersect or touch. |
| `TRUCK_INCONSISTENT_SHELL_NESTING` | The sampled shell containment is inconsistent. |
| `TRUCK_INCONSISTENT_SHELL_ORIENTATION` | Shell orientation does not alternate with nesting. |
| `TRUCK_CLASSIFICATION_FAILED` | The sampled geometry could not be classified. |
| `TRUCK_UNBOUNDED_RESULT` | The requested result has no supported bounded representation. |
| `TRUCK_INTERSECTION_FAILED` | Intersection construction failed. |
| `TRUCK_FACE_DIVISION_FAILED` | A face could not be divided along its intersection loops. |
| `TRUCK_CURVE_FITTING_FAILED` | An intersection curve could not be fitted. |
| `TRUCK_PROJECTION_FAILED` | A point could not be projected onto the required geometry. |
| `TRUCK_DEGENERATE_CURVE` | The constructed curve has too few distinct points. |
| `TRUCK_FILLET_NON_PLANAR_FACE` | This fillet operation requires planar faces. |
| `TRUCK_UNSUPPORTED_GEOMETRY` | The geometry is outside the supported scope of this operation. |
| `TRUCK_UNSUPPORTED_TOPOLOGY` | The topology is outside the supported scope of this operation. |
| `TRUCK_BLEND_CONSTRUCTION_FAILED` | Blend construction failed; the numerical or geometric constraint could not be resolved. |
| `TRUCK_NO_INTERSECTION` | No usable intersection was found in the sampled domains. |
| `TRUCK_OUTSIDE_NEIGHBOUR` | The moved boundary leaves its neighbouring face. |
| `TRUCK_NO_OFFSET` | The requested surface offset cannot be constructed. |
| `TRUCK_CONCAVE_EDGE` | This operation does not support crossing a concave edge. |
| `TRUCK_STEP_SYNTAX` | The STEP exchange structure could not be parsed. |
| `TRUCK_STEP_NO_DATA` | The STEP file has no DATA section. |
| `TRUCK_STEP_UNSUPPORTED_ENTITY` | The STEP entity type is not implemented. |
| `TRUCK_STEP_UNREADABLE_ENTITY` | The STEP entity could not be deserialized. |
| `TRUCK_STEP_MISSING_REFERENCE` | A referenced STEP entity could not be resolved. |
| `TRUCK_STEP_SKIPPED_DEPENDENCY` | A required STEP entity was skipped. |
| `TRUCK_STEP_CONVERSION_FAILED` | STEP geometry conversion failed. |
| `TRUCK_INCOMPLETE_RESULT` | The operation omitted geometry; inspect the reported diagnostics. |
| `TRUCK_INVALID_JSON` | The shape JSON could not be read. |
| `TRUCK_MESH_IO` | Mesh input or output failed. |

Crate-specific causes use the `TRUCK_TOPOLOGY_`, `TRUCK_GEOMETRY_`, `TRUCK_MODELING_`,
`TRUCK_DRAFTING_` and `TRUCK_MESH_` namespaces. Transparent wrappers preserve the underlying
code. Their `code()` methods are the authoritative mappings.

Explicit chamfer corner preferences are passed as the third argument of
`try_chamfer_solid_edges_with_distances`; pass `&[]` for the existing policy.
`ChamferCornerPreference { vertex, edge }` names original topology, with one
selected incident cap edge per tied vertex. The bounded fully selected convex
one-cylinder family requires orthogonal planar trihedra and equal cap setbacks
`d = 2h`, where `h` is the equal axis setback. Opposite cap choices are independent.
Selection and preference order do not choose the pivot. Missing preferences retain
rejection. Uniform and wire APIs retain their existing behavior.

Invalid, duplicate (including identical), stale, unselected, nonincident, axis,
non-tied or unused preferences return `InvalidParameter` at
`validate_corner_preference`, `corner_preference_network`,
`nonplanar_corner_preference`, `select_corner_preference` or
`unused_corner_preference`. Preference-bearing calls
never retry through a route that ignores the preference. Geometry/extent failures
from the simultaneous construction retain their diagnostic and leave input unchanged.

`replay_occt_blend captured-prefix [preference-file]` optionally takes an explicit
JSON record containing `input_step` (the exact STEP input text), `input_json` (the
exact captured input JSON text), and `preferences`. Each preference contains a `vertex`
coordinate triple and an `edge` descriptor with `points`, using the existing
geometric selection matcher. Both descriptors must resolve uniquely. The input
identity must match; there is no implicit annotation lookup. Reports mark annotated
calls with `explicit_corner_preferences`; compare these separately from unannotated
baseline runs.
