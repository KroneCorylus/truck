# Equal-radius corners with four or more incident edges

`high_valence_fillets.rs` checks a convex planar corner whose incident edges are all
selected. An exact spherical transition exists if all support planes, offset inward by
the radius, meet at one point. The solver uses all planes and rejects an inconsistent
system before constructing contact vertices. Existing cylindrical strips supply the
spherical patch boundary. No new surface representation or public API is required.

The tests use regular pyramids with 4, 5, 6 and 8 sides, base apothem 5 and height 10,
radii 0.2 and 1.2, in original and rigidly transformed coordinates. They check analytic
volume, geometric consistency, closed topology and meshes, requested sphere center and
radius, matching normals along every cylinder/sphere boundary, face-history counts,
reversed-selection equality, unchanged input and prepared STEP round trips. A Boolean
cut through the four-sided spherical cap checks downstream editing and another STEP
round trip. A rectangular pyramid without a common offset center and an excessive
radius check diagnostic failure without input mutation.

`fillet_edges.rs::all_pyramid_edges_with_four_edge_spherical_vertex` additionally
selects all eight edges of a square pyramid. It replaces the former rejection
expectation with tangency, topology, mesh, STEP, immutable-input and independent
Steiner-volume checks. The inner parallel pyramid is homothetic to the original,
so its volume, area and edge lengths scale by the third, second and first powers
of `1-r/inradius`, respectively. The radius-r parallel body's volume is
`V_inner + r A_inner + (r²/2) Σ(length × exterior_angle) + 4πr³/3`.

## Independent volume

For a regular n-sided pyramid, let `a` be the base apothem, `H` the height and `r` the
fillet radius. Define:

```
α = π/n                     s = a/H
q = sqrt(1+s²)              k = s/q
b = 1/sqrt(1+(s/cos α)²)    β = acos(cos(2α)/q² + k²)
c = H-r/k                  t = c+r k
m = k/cos(β/2)
I₁ = 2m sin(β/2)           I₂ = m²(β+sin β)/2
γ = 2 asin(s tan α b)
```

`c` is the sphere-center height, `k` the side-face normal's vertical component,
`b` the magnitude of a downward ridge unit vector's vertical component, and `β` the
angle between adjacent side normals. Direct integration gives the non-base areas:

```
A_plane    = n t² s tan α q
A_cylinder = n r (c β + r I₁)/b
A_sphere   = r²(2π - n γ)
```

The spherical normal polygon has area `2π-nγ` by angle defect. Its vertical flux is
`n b β/2`, from the vector area of its great-circle boundary. The closed boundary's
total vertical flux is zero, so the flat base area is

```
A_base = k A_plane + n r (c I₁ + r I₂)/b + r² n b β/2.
```

Relative to the sphere center, every non-base boundary point has `p·normal = r`;
the base has `p·normal = c`. The divergence theorem therefore gives the exact volume
`[r(A_plane+A_cylinder+A_sphere)+c A_base]/3`. These expressions do not inspect output
patches or use OCCT's volume. The downstream cut subtracts a spherical cap of height
`h = 0.2r`, whose exact volume is `πh²(r-h/3)`.

## Remaining scope

This construction requires a common center and all incident edges selected. It does
not supply a general curved corner patch for unequal radii, inconsistent offsets or
partially selected high-valence junctions. OCCT's paired probe uses a different corner
patch on these pyramids; equal topology counts do not establish identical geometry.
