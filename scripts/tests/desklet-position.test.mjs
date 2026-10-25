import { readFile } from 'node:fs/promises';
import { test } from 'node:test';
import assert from 'node:assert/strict';
import ts from 'typescript';

const source = await readFile(new URL('../../src/deskletPosition.ts', import.meta.url), 'utf8');
const { outputText } = ts.transpileModule(source, { compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 } });
const { parseDeskletPosition, visibleDeskletPosition } = await import(`data:text/javascript;base64,${Buffer.from(outputText).toString('base64')}`);
const size = { width: 620, height: 460 };
const main = { x: 0, y: 50, width: 3000, height: 1900 };

test('valid saved positions including negative display coordinates survive storage', () => {
  const position = { x: -1600, y: -450 };
  assert.deepEqual(parseDeskletPosition(JSON.parse(JSON.stringify(position))), position);
  for (const value of [null, {}, { x: '100', y: 20 }, { x: 10.5, y: 20 }, { x: Infinity, y: 20 }]) {
    assert.equal(parseDeskletPosition(value), null);
  }
});

test('visible positions remain exact on either Retina or secondary display', () => {
  assert.deepEqual(visibleDeskletPosition({ x: 1200, y: 400 }, size, [main]), { x: 1200, y: 400 });
  const left = { x: -2560, y: 50, width: 2560, height: 1400 };
  assert.deepEqual(visibleDeskletPosition({ x: -2000, y: 100 }, size, [main, left]), { x: -2000, y: 100 });
});

test('a disconnected display restores the entire window inside the nearest work area', () => {
  assert.deepEqual(visibleDeskletPosition({ x: -2000, y: 100 }, size, [main]), { x: 0, y: 100 });
  assert.deepEqual(visibleDeskletPosition({ x: 2900, y: 1800 }, size, [main]), { x: 2380, y: 1490 });
});

test('oversized windows keep their header accessible without negative clamp ranges', () => {
  const small = { x: 100, y: 80, width: 300, height: 200 };
  assert.deepEqual(visibleDeskletPosition({ x: 500, y: 500 }, size, [small]), { x: 100, y: 80 });
});
