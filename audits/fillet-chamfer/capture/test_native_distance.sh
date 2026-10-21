#!/usr/bin/env bash
set -euo pipefail
capture_sdk=$(realpath "${1:?usage: test_native_distance.sh OCCT_SDK_ROOT OUTPUT_DIRECTORY}")
mkdir -p "${2:?usage: test_native_distance.sh OCCT_SDK_ROOT OUTPUT_DIRECTORY}"
capture_output=$(realpath "$2")
capture_source=$(cd "$(dirname "$0")" && pwd)
g++ -std=c++17 -O2 -Wall -Wextra -Wpedantic \
  -isystem "$capture_sdk/usr/include/opencascade" \
  "$capture_source/test_native_distance.cpp" -L"$capture_sdk/usr/lib64" \
  -lTKDESTEP -lTKXSBase -lTKShHealing -lTKFillet -lTKPrim \
  -lTKTopAlgo -lTKGeomAlgo -lTKBRep -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o "$capture_output/test_native_distance"
LD_LIBRARY_PATH="$capture_sdk/usr/lib64${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" \
  "$capture_output/test_native_distance" "$capture_source/fixtures/curved-cap-chamfer.brep"
