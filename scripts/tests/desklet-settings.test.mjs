import { readFile } from 'node:fs/promises';
import { test } from 'node:test';
import assert from 'node:assert/strict';
import ts from 'typescript';

const source = await readFile(new URL('../../src/deskletSettings.ts', import.meta.url), 'utf8');
const { outputText } = ts.transpileModule(source, { compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 } });
const { parseDeskletSettings, newVmTarget } = await import(`data:text/javascript;base64,${Buffer.from(outputText).toString('base64')}`);

test('existing single VM migrates without losing full transparency or interval', () => {
  const settings = parseDeskletSettings({ name: 'Windows', address: '192.0.2.10', interval: 15, opacity: 0 });
  assert.equal(settings.opacity, 0);
  assert.equal(settings.vms.length, 1);
  assert.equal(settings.vms[0].name, 'Windows');
  assert.equal(settings.vms[0].address, '192.0.2.10');
  assert.equal(settings.vms[0].interval, 15);
  assert.ok(settings.vms[0].id);
});

test('multiple VMs keep distinct identities and independent intervals across saves', () => {
  const vms = [
    { id: 'a', name: 'Windows', address: '192.0.2.10', interval: 3 },
    { id: 'b', name: 'Linux', address: '::1', interval: 3600 },
  ];
  assert.deepEqual(parseDeskletSettings(JSON.parse(JSON.stringify({ vms, opacity: 35 }))), { vms, opacity: 35 });
});

test('duplicate identities are repaired so both VMs receive independent status', () => {
  const settings = parseDeskletSettings({ vms: [
    { id: 'same', name: 'One', address: '127.0.0.1', interval: 10 },
    { id: 'same', name: 'Two', address: '::1', interval: 10 },
  ] });
  assert.equal(settings.vms.length, 2);
  assert.equal(new Set(settings.vms.map(vm => vm.id)).size, 2);
});

test('malformed stored entries cannot start uncontrolled polling', () => {
  assert.deepEqual(parseDeskletSettings(null), { vms: [], opacity: 100 });
  const settings = parseDeskletSettings({ opacity: -5, vms: [null, {},
    { address: '127.0.0.1', interval: 0 }, { address: '::1', interval: 3.5 },
    { address: ' ', interval: 10 }, { address: '127.0.0.1', interval: 3601 },
  ] });
  assert.deepEqual(settings, { vms: [], opacity: 100 });
});

test('new VM drafts have their own stable identity', () => {
  const first = newVmTarget();
  const second = newVmTarget();
  assert.notEqual(first.id, second.id);
  assert.equal(first.interval, 10);
  assert.equal(first.address, '');
});
