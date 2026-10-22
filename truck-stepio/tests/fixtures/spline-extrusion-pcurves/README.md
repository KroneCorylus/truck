# C1 extruded spline parameter curve

Unmodified native STEP input for chamfer/equal_dist_sequence/C1, capture006.
Source: audits/fillet-chamfer/capture/2026-10-22-angle-seams/artifacts.zip,
member chamfer/equal_dist_sequence/C1/capture/006.input.step.
SHA256: 865feac228f1d3302e9cd9868815412351af61aa0155b5750668d29468b9afd2.

The regression checks exact conversion of the spline parameter trace and bounds
of its extruded face. Other edges in this native input remain inconsistent with
their supports at the kernel's strict tolerance. This fixture does not assert
that the complete solid is geometrically consistent or that its next chamfer
constructs. No imported geometry or tolerance is changed.
