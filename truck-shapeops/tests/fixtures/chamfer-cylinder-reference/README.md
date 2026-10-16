# Native half-cylinder reference fixture

Byte-preserved OCCT 7.9.3 input and schema-2 task from the shared-chamfer-ends
proof committed in71edbef1, native-evidence.zip members final/capture/002.input.*.
The four-face solid is a radius5,height10 half-cylinder with a two-edge top cap.
The original task assigns radial0.7 on the planar cap and vertical0.3 on the
cylindrical side. Tests swap those distances when identifying the cylinder as
the reference, using its independently known axis(0,0,z) and radius5. Tests also
construct a distance-angle request and transformed/reversed variants explicitly;
these synthetic variants are not claimed to be original native capture bytes.

Input volume is pi*5²*10/2. A radial inset a and vertical setback b remove
pi/2*(5*a*b-a²*b/3), independently of either kernel's result.

SHA256:
- half-cylinder.step: 5613afaf32a0f8592d249939898a6ab744ff21ccd74e04ddf3ce57f0ebb6743b
- half-cylinder.json: 0842ca9edaff20c05769c7fc52f74958b86599e6377d808c17393e481963bb6d
