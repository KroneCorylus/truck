#!/usr/bin/env bash
set -euo pipefail
capture_root=$(cd "$(dirname "$0")/../../.." && pwd)
capture_sdk=$(realpath "${1:?usage: build.sh OCCT_SDK_ROOT OUTPUT_DIRECTORY}")
mkdir -p "${2:?usage: build.sh OCCT_SDK_ROOT OUTPUT_DIRECTORY}"
capture_output=$(realpath "$2")
capture_source="$capture_root/audits/fillet-chamfer/capture"
capture_libraries=(-L"$capture_sdk/usr/lib64" -lTKDESTEP -lTKXSBase -lTKShHealing
    -lTKTopAlgo -lTKGeomAlgo -lTKBRep -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel)
g++ -std=c++17 -O2 -Wall -Wextra -Wpedantic -fPIC -shared \
    -isystem "$capture_sdk/usr/include/opencascade" -isystem /usr/include/tcl9.0 \
    "$capture_source/blend_capture.cpp" -L"$capture_sdk/usr/lib64" -lTKDraw -lTKTopTest \
    "${capture_libraries[@]}" -o "$capture_output/libTKBlendAudit.so"
g++ -std=c++17 -O2 -Wall -Wextra -Wpedantic \
    -isystem "$capture_sdk/usr/include/opencascade" "$capture_source/compare_shapes.cpp" \
    "${capture_libraries[@]}" -o "$capture_output/compare_shapes"
printf 'BLENDAUDIT: TKBlendAudit\n' > "$capture_output/BlendAudit"
