#!/usr/bin/env bash
set -euo pipefail
out=target/h1-local-witness-20261103
sdk=target/occt-sdk/usr
mkdir -p "$out"
python3 audits/fillet-chamfer/oracles/h1_local_report.py --snapshot
export LD_LIBRARY_PATH="$PWD/$sdk/lib64${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
g++ -std=c++17 -O2 -Wall -Wextra -isystem "$sdk/include/opencascade" audits/fillet-chamfer/oracles/h1_local_witness.cpp -L"$sdk/lib64" -lTKDESTEP -lTKXSBase -lTKShHealing -lTKTopAlgo -lTKGeomAlgo -lTKBRep -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel -o "$out/h1_witness"
ldd "$out/h1_witness" > "$out/libraries.txt"
"$out/h1_witness" target/h1-implementation-20261103/captures/H1/001.result.brep target/h1-implementation-20261103/captures/H1/001.truck.step 24 > "$out/localized.log"
g++ -std=c++17 -O2 -Wall -Wextra -isystem "$sdk/include/opencascade" audits/fillet-chamfer/oracles/h1_edge_witness.cpp -L"$sdk/lib64" -lTKDESTEP -lTKXSBase -lTKShHealing -lTKTopAlgo -lTKGeomAlgo -lTKBRep -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel -o "$out/h1_edge_witness"
python3 audits/fillet-chamfer/oracles/h1_local_report.py > "$out/summary.log"
