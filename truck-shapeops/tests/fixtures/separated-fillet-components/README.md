# Separated fillet components

Unmodified STEP input and native contour selection from OCCT `blend/simple/Q2`,
capture 001 in the 2026-10-22 open-chamfer-terminals audit. Three selected edges
form two components: a pair meeting at a planar corner and a separate curved rim.
The native result volume is 2.1864163191624213.

The replay regression checks both selection orders, input immutability, closed
material and STEP. The procedural companion covers two radii, two placements,
both face orders and three selection permutations with an independent exact
volume. For its height-two extrusion of a 3-by-4 rectangle and a radius-one
semicircle, input volume is 24+pi. The corner removal is
6*r^2*(1-pi/4)-r^3*(5/3-pi/2), including the overlap of two quarter-cylinder
cuts. The half-toroidal rim removes
pi/2*(r^2*(2-pi/2)+r^3*(pi/2-5/3)).

Exact rim contacts and face history are checked. An oversized cut must fail
without mutating the input. Existing generated-face interference guards remain.
