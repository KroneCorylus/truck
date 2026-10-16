# Native frustum reference fixtures

Byte-preserved OCCT 7.9.3 inputs and schema-2 tasks from the cone-reference native
controls, archive members fixture-origin/capture/001.input.* (narrowing) and
fixture-origin/capture/005.input.* (widening). Full circular frusta have height 10 and
bottom/top radii 5/3 or 3/5. Original tasks assign planar setback 0.3 and cone
setback 0.7. Tests explicitly synthesize cone references, angle requests,
transforms and reversed samples; those variants are not original capture bytes.

For top radius R, slope k=dr/dz, planar inset a and cone setback q, let
b=q/sqrt(1+k*k). The removed volume is pi*(R*a*b-k*a*b*b/3-a*a*b/3).
The input volume is pi*H/3*(Rbottom²+Rbottom*Rtop+Rtop²).

SHA256:
- narrowing.json: fad15ca57e4d5c269b8aaaeda46ce66e7b45ae370f14769216192e974c959598
- narrowing.step: 6dc585330860d91c35823ae7480f0f7fde25127f13104e0865903307f32c099c
- widening.json: cf82eafa1b99a835c86d4ea439cab804f77cee2fa37ad6a63815e62739deb957
- widening.step: 9644754f6058ed331284d8697d8e1bafc44de792952fc413c967ff2d492183fb
