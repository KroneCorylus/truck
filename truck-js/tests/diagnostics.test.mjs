import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);
const Truck = require(process.env.TRUCK_WASM_PKG || '../pkg-node/truck_js.js');
const cube = () => {
  const vertex = Truck.vertex(0, 0, 0);
  const edge = Truck.try_tsweep(vertex.upcast(), [1, 0, 0]);
  const face = Truck.try_tsweep(edge, [0, 1, 0]);
  return Truck.try_tsweep(face, [0, 0, 1]).into_solid();
};
const errorCode = (code, operation) => error => {
  assert.ok(error instanceof Error);
  assert.equal(error.code, code);
  assert.equal(error.operation, operation);
  assert.equal(typeof error.stage, 'string');
  assert.equal(typeof error.context, 'object');
  return true;
};

test('boolean failure can be handled without parsing messages', () => {
  const a = cube(), b = cube();
  assert.throws(() => Truck.try_and(a, b, NaN), error => {
    errorCode('TRUCK_INVALID_TOLERANCE', 'and')(error);
    assert.equal(error.context.parameter, 'tol');
    assert.equal(error.context.value, 'NaN');
    return true;
  });
  assert.equal(Truck.and(a, b, NaN), undefined);
});

test('builder input validation and topology causes are structured', () => {
  const v = Truck.vertex(0, 0, 0);
  assert.throws(() => Truck.try_line(v, v), error => {
    errorCode('TRUCK_INVALID_INPUT_TOPOLOGY', 'line')(error);
    assert.equal(error.cause.code, 'TRUCK_TOPOLOGY_SAME_VERTEX');
    return true;
  });
  const shape = v.upcast();
  assert.throws(() => Truck.try_translated(shape, [1, 2]), errorCode('TRUCK_INVALID_PARAMETER', 'translated'));
  assert.throws(() => Truck.try_rsweep(shape, [0, 0, 0], [0, 0, 1], 7, 0), errorCode('TRUCK_INVALID_PARAMETER', 'rsweep'));
});

test('JSON and STEP parse failures expose codes', () => {
  assert.throws(() => Truck.Solid.try_from_json(new TextEncoder().encode('{')), errorCode('TRUCK_INVALID_JSON', 'from_json'));
  assert.throws(() => Truck.Table.try_from_step('not STEP'), errorCode('TRUCK_STEP_SYNTAX', 'step_parse'));
});

test('partial STEP conversion reports both omission and dependency', () => {
  const input = readFileSync(new URL('../../resources/step/occt-cube.step', import.meta.url), 'utf8')
    .replace("#32 = PLANE('',#33);", "#32 = HOGE_SURFACE('',#33);");
  const table = Truck.Table.try_from_step(input);
  const report = table.get_shape_with_diagnostics(table.shell_indices()[0]);
  assert.equal(report.is_complete(), false);
  const [error] = report.diagnostics();
  assert.equal(error.code, 'TRUCK_STEP_UNSUPPORTED_ENTITY');
  assert.equal(error.context.entity_id, '17');
  assert.equal(error.context.related_entity_id, '32');
  assert.throws(() => table.get_shape_with_diagnostics(9007199254740993n), error => {
    errorCode('TRUCK_STEP_MISSING_REFERENCE', 'get_shape')(error);
    assert.equal(error.context.entity_id, '9007199254740993');
    return true;
  });
});

test('mesh reports and strict meshing expose validation failures', () => {
  const solid = cube();
  const report = solid.to_polygon_with_diagnostics(0.01);
  assert.equal(report.is_complete(), true);
  assert.deepEqual(report.diagnostics(), []);
  assert.throws(() => solid.try_to_polygon(0), errorCode('TRUCK_INVALID_TOLERANCE', 'triangulation'));
});

test('malformed mesh input retains its coded source', () => {
  const invalid = new TextEncoder().encode('v nope 0 0\n');
  assert.throws(() => Truck.PolygonMesh.try_from_obj(invalid), error => {
    errorCode('TRUCK_MESH_IO', 'from_obj')(error);
    assert.equal(error.cause.code, 'TRUCK_MESH_FROM_IO');
    return true;
  });
});
