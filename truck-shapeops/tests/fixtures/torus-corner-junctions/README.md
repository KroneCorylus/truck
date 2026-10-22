# Torus corner selections

Native blend/simple/E6, E8 and F3 inputs preserve their three physical selections.
E6/E8 use a 45-degree meridian and F3 uses a nonconvex 160-degree profile. The
regression scales geometry and radii by 0.01 and checks both selection orders,
closed material, STEP and input immutability. Original-scale production replay
and native sampled comparison remain separately required.

Independent material controls integrate distance to the inward outer torus solid
intersected with selected offset planes, then its radius-r dilation. The nonconvex
F3 profile subtracts its inner cavity above the unselected cap; its rounded wall
strip is integrated from the exact rolling-sphere envelope. A convex-only volume
calculation would omit this cavity. Native outputs have comparison discrepancies
and are not used as independent material-volume oracles.

F1 uses a separate mixed convex/concave construction. Its former rejection is
replaced by positive material, STEP and immutability checks; oversized convex
rejections remain unchanged. Dedicated mixed controls also verify radius and G1.
