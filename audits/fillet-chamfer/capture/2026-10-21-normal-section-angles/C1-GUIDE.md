# C1 guide construction diagnosis

This explains a measured difference; it does not change the recorded comparison
failure, comparison threshold, requested selection, or native reference result.

On distance-angle sequence C1/001, a fresh ChFi3d_ChBuilder using the captured
input, selected edge, reference face, distance 0.01 and angle 30 degrees produces
the same sampled contact locations as the captured native result. Its internal
elementary guide differs from the original edge: 1,001 original edge samples
have a maximum nearest distance of 0.005971920944933963 to the internal guide.
An independent nearest search over 20,001 internal guide samples gives an upper
bound of 0.00597193016809689 at those original samples. These are sampled values,
not global error bounds. The exported internal guide is included in the archive.

At 15 interior samples of the reference contact edge, the native result is
0.010564840609682097 to 0.014949375874974911 from the original edge, but
0.009999817906598100 to 0.010000063448344145 from its internal guide. Truck's
contact is 0.009999996828580292 to 0.010000000099958752 from the original edge.
The dense guide search independently agrees with the native contact distances
to about 1e-8. The original input's native BREP-to-STEP and Truck import/export
transport comparisons both differ by at most 1.84e-14 at sampled points.

This evidence identifies native guide smoothing as a contributor to C1's
boundary difference. It does not prove that every other mismatch has this cause,
or establish global accuracy of either output. The new API's contract remains
a chord measured from the original guide. Do not distort that guide to make a
native comparison pass. C1 remains unresolved in the same limited comparison
count, pending an explicit policy for reference results with geometric drift.

Reproduce from the repository root using the archived C++ probe and compiler
command, changing only the source/output basenames for the non-dense probe.
Run with the captured 001.input.brep, 001.result.brep and 001.truck.step from
chamfer/dist_angle_sequence/C1/capture in artifacts.zip. Set LD_LIBRARY_PATH to
target/occt-sdk/usr/lib64. The dense probe also writes the recomputed native result
and its elementary guide to target/c1-recomputed-angle.brep and
target/c1-computed-guide.brep. No production or native-library change is involved.
