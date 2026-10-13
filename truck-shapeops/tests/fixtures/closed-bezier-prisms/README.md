# Closed Bézier prism STEP fixtures

Unmodified native input STEP bytes from `audits/fillet-chamfer/capture/2026-10-13-conical-domains/artifacts.zip`:

- `cubic.step`: `bugs/modalg_7/bug30940_2/capture/001.input.step`, SHA-256 `40658e7a0da09323750635035f260db205e9f1f911d598da0e4467a151e14ccf`.
- `quartic.step`: `bugs/modalg_7/bug30940_5/capture/001.input.step`, SHA-256 `b27a8a4598f5fba5a5c7d7afebfd7b1cd20007c6535d07d24246ba627eba2c4a`.

Both profiles close at a nonsmooth cusp and are extruded by height 10. The cubic control points are (0,0), (10,0), (0,10), (0,0). Its coordinates are x=30t(1-t)^2 and y=30t^2(1-t); the signed area integral `(x*y'-y*x')/2` is 15, so the volume is **150**.

The quartic control points are (0,0), (0,-10), (50,50), (-10,0), (0,0). Its coordinates are x=300t^2-640t^3+340t^4 and y=-40t+420t^2-720t^3+340t^4. The signed area is **720/7**, so the volume is **7200/7**. These expectations follow from the profile dimensions, not from an imported kernel result.

The raw inputs contain a repeated vertical seam on the extruded side face. Splitting that face requires distinct parameter branches at the coincident profile endpoints; the cusp does not make the profile periodic. Native top-rim selections include the nonsmooth cusp. Input recovery alone does not establish support for those fillets.
