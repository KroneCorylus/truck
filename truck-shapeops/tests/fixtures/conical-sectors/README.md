# Native conical sector

`H6.step` preserves the raw native input bytes from
`audits/fillet-chamfer/capture/2026-10-13-cone-pole-meshing/artifacts.zip`,
member `blend/simple/H6/capture/001.input.step`. H7 and H9 have identical
geometry records; their export timestamps differ.
SHA-256: `0f1722b346a2680aa6560e5483922945196fc0a391a21e80bc676b76e45a9097`.

The profile `(0,250), (200,50), (50,50), (50,-50), (200,-50),
(0,-250)` in radius/height coordinates revolves through 270 degrees. Its
independent volume is `(3*pi/4)*(2*200^3/3 + 50^2*100)` =
13155419.236907259.

H6 rounds the lower radius-50 cylinder rim at z=-50 with radius 10. The
added cross-section is a 10-by-10 square minus a quarter circle. Its area
is `A = 10^2*(1-pi/4)` and its radial centroid offset from the original
corner is `x = 10*(10-3*pi)/(12-3*pi)`. Revolving that area through
270 degrees adds `(3*pi/2)*(50+x)*A`, giving 13160701.562723719.
The rolling circle center is `(radius,z)=(60,-40)`.
