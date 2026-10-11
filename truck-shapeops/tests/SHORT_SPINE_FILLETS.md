# Fillets wider than the spine

A fillet's radius is a transverse dimension. A short straight spine is valid when
its cylindrical strip can be trimmed by the end faces. The previous convex path
required the full rolling ball to fit behind distant faces even at these planar
terminations. It rejected a radius-0.5 fillet on a length-0.2 spine before constructing
the trimmed patch.

Single-edge terminal centers now need to stay within the convex input planes. The
two exact rational end arcs supply the control rows of the cylindrical patch. Every
control weight must be positive and finite, and every Euclidean control point must
lie in every input half-space, within the existing numerical tolerance. The rational
convex-hull property then bounds the complete patch, including oblique elliptical
terminations. Spherical/miter corner center checks are retained. This does not add
support for collapsed edges, disappearing faces or arbitrary nonconvex collisions.

The regression selects the z-axis edge of a `100 × 100 × h` box for `h=0.2,1,10`
and radius `r=0.5,10`. It repeats each with either sign of the shear
`z'=z+s*x-s*y/2`, `s=±0.4`, and without shear, before/after a rigid motion:
36 operations. The shear makes the two caps oblique while retaining vertical side
planes. Its determinant is one, so the independent volume remains

```
V = h * [10000 - (1-π/4)*r²].
```

The exact fillet satisfies `(x-r)²+(y-r)²=r²`, with normal proportional to
`(x-r,y-r,0)`, and stays between `0 <= z-s*x+s*y/2 <= h`. The tests sample these
constraints, validate native geometry, topology, closed meshes, face history,
STEP and unchanged inputs. Radius 101 still rejects as `OutsideNeighbour` without
mutation. Modeling/mesh tolerance is 0.001; STEP fitting tolerance is 0.00005.

The paired OCCT 7.9.3 probe performs the six native unsheared tasks. All return
valid single solids with matching sampled dimensions and analytic volume (largest
recorded integration deviation approximately `1.2e-10`). Its source and output are
under `audits/fillet-chamfer/probes/short_spine_fillets*`. OCCT shear, transform,
mesh, STEP and editing behavior are not established by that probe.
