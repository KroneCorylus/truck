# Separate fillet contours

`cylinder-cone-stack.step` contains unmodified native input bytes from
`feat/featrf/A4/capture/001.input.step` in
`audits/fillet-chamfer/capture/2026-10-14-cusp-contacts/artifacts.zip`.
SHA-256: `30ee8adf04a55b8a080efecc79ceed25c0c0476e4e910299be18983e854247a8`.

The stack is a radius-3, height-5 cylinder, a height-3 cone frustum
(radius 2 to 1.5), then a radius-1, height-2 cylinder. Two independent
reentrant rims at (radius 2, z=5) and (radius 1, z=8) receive radius-0.3
fillets. A5, A6 and B4 repeat this same initial blend before later rib features.

The input volume is 56.25π. Revolving each added circular profile region
using π∮ρ²dz gives added volumes 0.16698658092757426 and
0.12948648959397593, for **177.01105983494742** in total. The test computes
this integral from the dimensions; no native output is used as its oracle.
