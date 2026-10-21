# Native cone reference-face inconsistency

Six native-only controls start from a fresh unit cone (base radius1, height1).
Each operation selects its base circle and asks for two support distances.
Every operation builds and passes native BREP validity. Their contact circles
are measured directly from analytic native curves, without the sampled distance
comparator or Truck import/export.

The two equivalent ways to request a plane setback0.1 and cone setback0.2 are:

- Reference plane, distances(0.1,0.2): measured setbacks(0.1,0.2).
- Reference cone, distances(0.2,0.1): measured plane setback0.1, cone setback
  0.1093836321356054, instead of0.2.

The reciprocal request shows the analogous discrepancy:

- Reference cone, distances(0.1,0.2): measured cone setback0.1 and plane setback0.2.
- Reference plane, distances(0.2,0.1): measured cone setback0.1 and plane setback
  0.10938363213560542, instead of0.2.

Both equal-distance controls produce0.1 setbacks on both supports. Setbacks are
Euclidean distances from the original base circle in a meridian section; both
supports are straight in that section. Thus chord and support distance coincide.
The reference-face description changes no intended physical distance in each
pair. This demonstrates a native reference-face invariance failure for these
inputs, rather than a numerical tolerance issue.

The native contact circles of corpus dist_dist/C6 are consistent with the second
control (cone contact height0.077345908, radius0.922654092). This is a lead for
that family, not proof that every cone or corpus mismatch has the same cause.
No native library, capture reference, production solver, tolerance, or corpus
pass/fail classification changes in this checkpoint.

cone-reference-diagnosis.zip includes source, compiler command, run output and
12 input/result BREP files; cone-reference-diagnosis.json records measurements
and member hashes. Compile the archived source from the repository root with
the recorded SDK command, create target/native-cone-distance-controls, and run
target/probe-native-cone-distances with LD_LIBRARY_PATH=target/occt-sdk/usr/lib64.
