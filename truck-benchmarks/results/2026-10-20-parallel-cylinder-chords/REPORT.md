# Parallel cylinder chamfer chords and subdivided concave caps

Replay now converts each cylindrical chord to the core tangent setback R*tan(2*asin(D/(2R))) for straight spines between parallel cylinders. Equal and unequal distances preserve their support/reference assignment. Distance-angle requests between two cylinders remain explicitly unsupported, and oversized chords remain rejected.

Concave chamfers whose terminal walk crosses coplanar cap subdivisions now retain those internal seams at the original corner. The construction splits the two outer cap arcs at the contacts and adds the missing curved triangular cap patch. Existing faces remain in their original index order. This branch requires empty consumed-neighbor paths, a nonempty coplanar/co-oriented terminal fan, and a concave chamfer. Each new cap polygon and every modified planar boundary are validated. Fillets and noncoplanar terminal walks retain their prior routes.

Parent 0847f7b7 fails three new tests: equal chord conversion, unequal support selection, and the direct core regression with subdivided union caps. Two rejection guards pass. Parent output and test/fixture hashes are retained. The initial adapter-only candidate also failed the union construction and is retained as evidence. No failing fixture was removed or simplified.

The five new tests exercise 18 replay variants over equal/unequal cylinder intersections and a cylinder union with fragmented end caps, plus direct core construction and rejection guards. They cover both reference faces, reversed axes/samples/face order, scale and rigid transform. Assertions include exact contacts, oriented bevel support, independent material, shared solid/STEP checks and input immutability.

Independent section oracle: cylinder radii R1/R2, center distance d, common spine S=(x,y), x=(R1²-R2²+d²)/(2d), y=sqrt(R1²-x²). Rotate the two radii by opposite chord sweeps into their faces. The changed area is the absolute sum of the two signed circular-sector integrals and the straight bevel integral, divided by two. Add it to the union or subtract it from the intersection, then multiply by height. The fixture and validation sources retain the explicit formula.

Nine native controls and nine Truck outputs are valid single solids. All Truck outputs match independent contacts/material and native sampled geometry. Maximum Truck contact error 2.2397950889897277e-13; material error 2.7853275241795927e-10. Native input immutability holds for six controls. The three union controls change two endpoint vertex tolerances from approximately 1.0000016e-7 to 1e-6. The original capture assertion failed for that reason; its failure and raw before/after BREPs are preserved. This is not reported as native input immutability. All Truck input files remain byte-identical.

Full release validation: 400 shapeops and 98 meshalgo tests pass, with one existing ignore each; 54 replay tests and 13 capture tests pass. All-target release Clippy and single-threaded actions smoke pass. Replay built last. Archive contains 178 byte-verified members, SHA256 bed444ec845ebf765ff88d61e8dae2bfe7db8190c0c7a679d3c37d3f8ea0f63e.

A clean full corpus capture follows this implementation/evidence commit. Finite samples do not prove global boundary error or complete parity. No consumer change, push, PR, dependency, tolerance relaxation or performance claim.
