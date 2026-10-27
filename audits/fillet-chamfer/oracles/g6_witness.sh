#!/usr/bin/env bash
# Run from the repository root after building the release replay example.
# Finite trimmed-face/edge samples plus local refinement are not global bounds.
set -euo pipefail
sdk=${OCCT_SDK:-../truck/target/occt-sdk/usr}
out=${G6_WITNESS_OUT:-target/g6-investigation/review-witness}
mkdir -p "$out"
export LD_LIBRARY_PATH="$sdk/lib64${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
g++ -std=c++17 -O2 -g -Wall -Wextra -isystem "$sdk/include/opencascade" \
  audits/fillet-chamfer/oracles/g6_corner_witness.cpp -L"$sdk/lib64" \
  -lTKFillet -lTKDESTEP -lTKXSBase -lTKShHealing -lTKTopAlgo -lTKGeomAlgo \
  -lTKBRep -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel -o "$out/g6_witness"
g++ -std=c++17 -O2 -Wall -Wextra -isystem "$sdk/include/opencascade" \
  audits/fillet-chamfer/oracles/g2_corner_witness.cpp -L"$sdk/lib64" \
  -lTKDESTEP -lTKXSBase -lTKShHealing -lTKTopAlgo -lTKGeomAlgo \
  -lTKBRep -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel -o "$out/g2_witness"
for case in G6 G2; do
  mkdir -p "$out/$case"
  for ext in input.json input.step result.brep; do
    cp "target/g6-investigation/replays/$case/001.$ext" "$out/$case/001.$ext"
  done
  target/release/examples/replay_occt_blend "$out/$case/001" > "$out/$case/replay.log"
  ../truck/target/occt-distance-tree-plugin/compare_shapes \
    "$out/$case/001.result.brep" "$out/$case/001.truck.step" "$out/$case/comparison.json"
done
"$out/g6_witness" "$out/G6/001.result.brep" "$out/G6/001.truck.step" 20 > "$out/G6/localized.log"
"$out/g2_witness" "$out/G2/001.result.brep" "$out/G2/001.truck.step" > "$out/G2/material.log"
cat > "$out/native.tcl" <<EOF_TCL
pload MODELING
beziercurve w1 5 0 0 0 20 0 0 20 5 0 25 10 0 10 20 0
mkedge w1 w1
polyline w2 10 20 0 0 0 0
wire w w1 w2
mkplane w w
prism s w 0 2 10
tscale s 0 0 0 10
explode s E
blend result s 10 s_4 10 s_1 10 s_6
save result $out/draw-native.brep
exit
EOF_TCL
cat > "$out/native.gdb" <<'EOF_GDB'
set pagination off
set breakpoint pending on
set debuginfod enabled off
break ChFi3d_FilBuilder::PerformThreeCorner(int)
commands
silent
printf "TRACE PerformThreeCorner\n"
bt 4
continue
end
break GeomFill_ConstrainedFilling::Init(opencascade::handle<GeomFill_Boundary> const&, opencascade::handle<GeomFill_Boundary> const&, opencascade::handle<GeomFill_Boundary> const&, bool)
commands
silent
printf "TRACE three-boundary Coons filling\n"
bt 5
continue
end
break GeomFill_ConstrainedFilling::Init(opencascade::handle<GeomFill_Boundary> const&, opencascade::handle<GeomFill_Boundary> const&, opencascade::handle<GeomFill_Boundary> const&, opencascade::handle<GeomFill_Boundary> const&, bool)
commands
silent
printf "TRACE four-boundary Coons filling\n"
bt 5
continue
end
break BRepBlend_AppSurface::BRepBlend_AppSurface(opencascade::handle<Approx_SweepFunction> const&, double, double, double, double, double, GeomAbs_Shape, int, int)
commands
silent
printf "TRACE sweep approximation\n"
bt 6
continue
end
run
EOF_GDB
# Trace the original DRAW construction only after the spatial witness above.
gdb -batch -x "$out/native.gdb" --args DRAWEXE -b -f "$out/native.tcl" > "$out/native-trace.log" 2>&1
python3 audits/fillet-chamfer/oracles/g6_material.py > "$out/flux.log"
sha256sum "$out/G6/001.truck.step" "$out/G6/001.result.brep" \
  "$out/G2/001.truck.step" "$out/G2/001.result.brep" \
  target/release/examples/replay_occt_blend "$out/g6_witness" \
  truck-shapeops/src/fillet/planar_fillet.rs \
  truck-shapeops/tests/quartic_remote_junction.rs \
  audits/fillet-chamfer/oracles/g6_material.py \
  audits/fillet-chamfer/oracles/g6_witness.sh \
  audits/fillet-chamfer/oracles/g6_corner_witness.cpp > "$out/identities.txt"
