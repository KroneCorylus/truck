# Dimension-defined controls, separate from the unchanged native corpus.
box block 10 20 30
renamevar block edges
explode edges e
renamevar edges faces
explode faces f
chamf equal_result faces edges_1 2
chamf unequal_result faces edges_1 faces_3 2 3
chamf swapped_result faces edges_1 faces_1 2 3
chamf angle_result faces edges_1 faces_3 A 2 30
blend fillet_control faces 2 edges_1
box block 10 20 30
trotate block 0 0 0 1 2 3 37
ttranslate block 12 -9 7
renamevar block edges
explode edges e
renamevar edges faces
explode faces f
chamf moved_equal faces edges_1 2
chamf moved_unequal faces edges_1 faces_3 2 3
chamf moved_angle faces edges_1 faces_3 A 2 30
