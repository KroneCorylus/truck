"""Run with FreeCADCmd; matches Truck feature fixtures in millimetres. See BENCHMARKS.md."""
import json
import math
import statistics
import time
import os
from pathlib import Path

import FreeCAD as App
import Part
import MeshPart


def measure(name, run, expected=None, samples=20, validate=None):
    if os.environ.get("CAD_BENCH_FILTER") and not name.startswith(os.environ["CAD_BENCH_FILTER"]):
        return
    result = run()
    if validate is None:
        assert result.isValid() and len(result.Solids) == 1, name
        if expected is not None:
            assert abs(result.Volume - expected) < max(1e-6, expected * 1e-8), name
    else:
        assert validate(result), name
    times = []
    for _ in range(samples):
        start = time.perf_counter()
        next_result = run()
        times.append((time.perf_counter() - start) * 1000)
        result = next_result
    print("BENCH " + json.dumps(dict(name=name, median_ms=statistics.median(times),
          min_ms=min(times), max_ms=max(times), samples=samples,
          volume=getattr(result, "Volume", None), facets=getattr(result, "CountFacets", None),
          faces=len(result.Faces) if hasattr(result, "Faces") else None)), flush=True)


def main():
    print("VERSION " + json.dumps(dict(freecad=App.Version(), occ=Part.OCC_VERSION)), flush=True)
    for holes in [1, 10, 30, 100]:
        cols = math.ceil(math.sqrt(holes))
        rows = math.ceil(holes / cols)
        plate = Part.makeBox(cols * 4, rows * 4, 2)
        cutters = [Part.makeCylinder(1, 4, App.Vector((i % cols) * 4 + 2,
                   (i // cols) * 4 + 2, -1)) for i in range(holes)]
        compound = Part.makeCompound(cutters)
        expected = cols * rows * 32 - holes * math.pi * 2
        measure(f"subtract_batch/{holes}", lambda: plate.cut(compound), expected)

        def sequential():
            result = plate
            for cutter in cutters:
                result = result.cut(cutter)
            return result

        measure(f"subtract_sequential/{holes}", sequential, expected)

    # Same 60 x 60 x 10 plate and radius-33, depth-5 pocket as the app corpus.
    plate = Part.makeBox(60, 60, 10)
    cutter = Part.makeCylinder(33, 5, App.Vector(30, 30, 5))
    segment = 33**2 * math.acos(30 / 33) - 30 * math.sqrt(33**2 - 30**2)
    expected = 36000 - 5 * (math.pi * 33**2 - 4 * segment)
    measure("circle_remove/kernel", lambda: plate.cut(cutter), expected)


    def build_pocket():
        return Part.makeBox(60, 60, 10).cut(
            Part.makeCylinder(33, 5, App.Vector(30, 30, 5)))


    measure("circle_remove/build_geometry", build_pocket, expected)


    def polygon(edges, radius, z):
        points = [App.Vector(radius * math.cos(math.tau * i / edges),
                             radius * math.sin(math.tau * i / edges), z) for i in range(edges)]
        return Part.Wire(Part.makePolygon(points + points[:1]).Edges)


    for edges in [4, 32, 128]:
        face = Part.Face(polygon(edges, 2, 0))
        area = edges * 2 * math.sin(math.tau / edges)
        measure(f"extrude/{edges}", lambda: face.extrude(App.Vector(0, 0, 5)), area * 5)
        wire = polygon(edges, 0.5, 0)
        wire.rotate(App.Vector(), App.Vector(1, 0, 0), -90)
        wire.translate(App.Vector(2, 0, 0))
        face = Part.Face(wire)
        measure(f"revolve/{edges}", lambda: face.revolve(App.Vector(), App.Vector(0, 0, 1), 360),
                edges * 0.125 * math.sin(math.tau / edges) * 4 * math.pi)

    for sections in [2, 8, 32]:
        wires = [polygon(8, 1 + 0.2 * math.sin(i / (sections - 1) * math.pi),
                         i / (sections - 1) * 5) for i in range(sections)]
        measure(f"loft/{sections}", lambda: Part.makeLoft(wires, True, False))

    for segments in [1, 8, 32]:
        wire = Part.Wire([Part.makeCircle(0.2)])
        path = Part.Wire(Part.makePolygon([App.Vector(0, 0, i) for i in range(segments + 1)]).Edges)
        measure(f"sweep_path/{segments}", lambda: path.makePipeShell([wire], True, False),
                math.pi * 0.2**2 * segments)

    cube = Part.makeBox(4, 4, 4)
    for count in [1, 12]:
        edges = cube.Edges[:count]
        measure(f"fillet/{count}", lambda: cube.makeFillet(0.2, edges))
        measure(f"chamfer/{count}", lambda: cube.makeChamfer(0.2, edges))
    top = max(cube.Faces, key=lambda f: f.CenterOfMass.z)
    measure("shell_box", lambda: cube.makeThickness([top], -0.2, 1e-6), 64 - 3.6**2 * 3.8)

    a = Part.makeCylinder(1, 2)
    b = Part.makeCylinder(1, 2, App.Vector(0.8, 0, 0.5))
    measure("intersect_cylinders", lambda: a.common(b))
    measure("union_cylinders", lambda: a.fuse(b))
    a, b = a.toNurbs(), b.toNurbs()
    measure("intersect_nurbs_cylinders", lambda: a.common(b))


    def perforated_plate(holes):
        cols = math.ceil(math.sqrt(holes))
        rows = math.ceil(holes / cols)
        points = [App.Vector(0, 0, 0), App.Vector(cols * 4, 0, 0),
                  App.Vector(cols * 4, rows * 4, 0), App.Vector(0, rows * 4, 0)]
        outer = Part.Wire(Part.makePolygon(points + points[:1]).Edges)
        wires = [outer]
        for i in range(holes):
            hole = Part.Wire([Part.makeCircle(1, App.Vector((i % cols) * 4 + 2, (i // cols) * 4 + 2, 0))])
            hole.reverse()
            wires.append(hole)
        return Part.Face(wires).extrude(App.Vector(0, 0, 2))


    for holes in [1, 10, 30, 100]:
        solid = perforated_plate(holes)
        # Copy geometry so OCCT cannot reuse a cached triangulation from the preflight.
        measure(f"plate/{holes}", lambda: MeshPart.meshFromShape(Shape=solid.copy(True, False), LinearDeflection=0.01, AngularDeflection=math.pi, Relative=False),
                validate=lambda mesh: mesh.CountPoints > 0 and mesh.CountFacets > 0)
        if holes > 30:
            continue
        matrix = App.Matrix()
        matrix.move(App.Vector(1, 2, 3))
        def translated():
            result = solid.copy(True, False)
            result.translate(App.Vector(1, 2, 3))
            return result
        measure(f"transform/{holes}", translated, solid.Volume)

    for tol in [0.1, 0.01, 0.001]:
        solid = Part.makeCylinder(1, 2)
        measure(f"cylinder_tolerance/{tol}", lambda: MeshPart.meshFromShape(Shape=solid.copy(True, False), LinearDeflection=tol, AngularDeflection=math.pi, Relative=False),
                validate=lambda mesh: mesh.CountPoints > 0 and mesh.CountFacets > 0)

    import TechDraw
    for holes in [1, 10, 30]:
        solid = perforated_plate(holes)
        measure(f"hidden_lines/{holes}", lambda: TechDraw.project(solid, App.Vector(1, 1, 1)),
                validate=lambda result: any(not s.isNull() for s in result[:2]) and any(not s.isNull() for s in result[2:]))

    document = App.newDocument("BenchDraft")
    body = document.addObject("PartDesign::Body", "Body")
    base = body.newObject("PartDesign::Feature", "Base")
    base.Shape = cube
    draft = body.newObject("PartDesign::Draft", "Draft")
    draft.NeutralPlane = (base, ["Face5"])
    draft.Angle = math.degrees(0.05)
    draft.Refine = False
    for count in [1, 4]:
        draft.Base = (base, [f"Face{i + 1}" for i in range(count)])
        def run_draft():
            draft.touch()
            document.recompute()
            return draft.Shape
        measure(f"draft_box/{count}", run_draft)
    App.closeDocument(document.Name)
    if not os.environ.get("CAD_BENCH_FILTER") or os.environ["CAD_BENCH_FILTER"].startswith("pattern100"):
        import Sketcher
        document = App.newDocument("PatternBench")
        body = document.addObject("PartDesign::Body", "Body")
        sketch = body.newObject("Sketcher::SketchObject", "PlateSketch")
        points = [App.Vector(0, 0, 0), App.Vector(40, 0, 0), App.Vector(40, 40, 0), App.Vector(0, 40, 0)]
        sketch.addGeometry([Part.LineSegment(points[i], points[(i + 1) % 4]) for i in range(4)], False)
        pad = body.newObject("PartDesign::Pad", "Pad")
        pad.Profile, pad.Length = sketch, 2
        document.recompute()
        circles = body.newObject("Sketcher::SketchObject", "HoleSketch")
        circles.Placement.Base = App.Vector(0, 0, 2)
        circles.addGeometry(Part.Circle(App.Vector(2, 2, 0), App.Vector(0, 0, 1), 1), False)
        pocket = body.newObject("PartDesign::Pocket", "Pocket")
        pocket.Profile, pocket.Length = circles, 2
        document.recompute()
        x = body.newObject("PartDesign::LinearPattern", "X")
        x.Direction, x.Length, x.Occurrences = (circles, ["H_Axis"]), 36, 10
        y = body.newObject("PartDesign::LinearPattern", "Y")
        y.Direction, y.Length, y.Occurrences = (circles, ["V_Axis"]), 36, 10
        pattern = body.newObject("PartDesign::MultiTransform", "Pattern")
        pattern.Originals, pattern.Transformations, pattern.Refine = [pocket], [x, y], False
        def rebuild_pattern():
            for obj in [sketch, pad, circles, pocket, x, y, pattern]:
                obj.touch()
            document.recompute()
            return pattern.Shape
        measure("pattern100/recompute", rebuild_pattern, 3200 - 200 * math.pi, samples=10)
        def rebuild_pattern_meshes():
            shape = rebuild_pattern()
            meshes = [MeshPart.meshFromShape(Shape=obj.Shape.copy(True, False),
                      LinearDeflection=0.05, AngularDeflection=math.pi, Relative=False)
                      for obj in [pad, pocket, pattern]]
            assert all(mesh.CountFacets > 0 for mesh in meshes)
            return shape
        measure("pattern100/recompute_meshes", rebuild_pattern_meshes, 3200 - 200 * math.pi, samples=10)
        App.closeDocument(document.Name)

    directory = Path(__file__).resolve().parents[2] / "target" / "comparison"
    for holes in [1, 10, 30]:
        source = directory / f"shared-{holes}.step"
        if not source.exists():
            raise RuntimeError(f"Run Truck import_file benchmarks first: missing {source}")
        solid = Part.read(str(source))
        measure(f"import_file/{holes}", lambda: Part.read(str(source)), solid.Volume)
        output = directory / f"freecad-output-{holes}.step"
        def export_file():
            solid.exportStep(str(output))
            return output
        measure(f"export_file/{holes}", export_file, validate=lambda p: p.stat().st_size > 0)
        assert Part.read(str(output)).isValid()
    print("COMPLETE", flush=True)
    print("API " + json.dumps([name for name in dir(cube) if any(
        word in name.lower() for word in ["draft", "step", "project", "string"])]), flush=True)


try:
    main()
except Exception:
    import traceback
    print("FAILED " + traceback.format_exc(), flush=True)
