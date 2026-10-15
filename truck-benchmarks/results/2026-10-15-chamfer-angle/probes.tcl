# Dimension-defined controls outside corpus counts.
# Transformed ordinals were checked against inverse-transformed edge endpoints and plane normals.
wedge p 1 1 5 5
nexplode p e
renamevar p edges
nexplode edges f
chamf result edges p_12 edges_5 A 0.1 30
chamf result edges p_12 edges_6 A 0.1 30
chamf result edges p_7 edges_2 A 0.1 5
chamf result edges p_7 edges_6 A 0.1 5
wedge p 1 1 5 5
trotate p 0 0 0 1 2 3 37
ttranslate p 12 -9 7
nexplode p e
renamevar p edges
nexplode edges f
chamf result edges p_12 edges_4 A 0.1 30
chamf result edges p_12 edges_5 A 0.1 30
chamf result edges p_6 edges_2 A 0.1 5
chamf result edges p_6 edges_5 A 0.1 5
