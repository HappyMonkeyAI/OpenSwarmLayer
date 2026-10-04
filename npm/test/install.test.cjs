'use strict';
const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const os = require('node:os');
const path = require('node:path');
const crypto = require('node:crypto');
const { main, parseArgs, selectAsset, checkLinuxInstall, checkedUrl, releaseResponse, downloadAsset } = require('../lib/install.cjs');

const bytes = Buffer.from('independent native package fixture');
const asset = { name: 'OpenSwarmLayer-fixture.deb', platform: 'linux', size: bytes.length, sha256: crypto.createHash('sha256').update(bytes).digest('hex') };
async function fixture(fn) {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'OpenSwarmLayer-npm-test-'));
  try { await fn(root); } finally { await fs.rm(root, { recursive: true }); }
}

test('arguments and unsupported platforms fail before downloads', () => {
  assert.deepEqual(parseArgs([]), { action: 'help' });
  assert.deepEqual(parseArgs(['download', '--format', 'appimage']), { action: 'download', format: 'appimage' });
  for (const args of [['install', '--format'], ['install', '--format', 'appimage'], ['install', '--version', '../../other'], ['install', '--format', 'deb', '--format', 'exe']]) assert.throws(() => parseArgs(args));
  assert.equal(selectAsset(undefined, 'win32', 'x64').platform, 'win32');
  assert.equal(selectAsset(undefined, 'linux', 'x64').platform, 'linux');
  for (const args of [['exe', 'linux', 'x64'], ['deb', 'win32', 'x64'], ['deb', 'darwin', 'x64'], ['deb', 'linux', 'arm64']]) assert.throws(() => selectAsset(...args));
  checkLinuxInstall('2.39'); checkLinuxInstall('2.40');
  for (const version of ['2.38', '1.99', '', undefined, 'musl']) assert.throws(() => checkLinuxInstall(version));
});

test('HTTPS destinations exclude credentials and unrelated hosts', () => {
  checkedUrl('https://github.com/example');
  checkedUrl('https://release-assets.githubusercontent.com/file');
  for (const url of ['http://github.com/file', 'https://github.com.evil.test/file', 'https://user:password@github.com/file', 'https://github.com:8443/file']) assert.throws(() => checkedUrl(url));
});

test('redirect handling permits the release CDN and bounds loops', async () => {
  let calls = 0;
  const response = await releaseResponse('https://github.com/file', async () => {
    calls++;
    return calls === 1 ? new Response(null, { status: 302, headers: { location: 'https://release-assets.githubusercontent.com/file' } }) : new Response(bytes);
  });
  assert.deepEqual(Buffer.from(await response.arrayBuffer()), bytes);
  assert.equal(calls, 2);
  await assert.rejects(releaseResponse('https://github.com/file', async () => new Response(null, { status: 302, headers: { location: 'https://evil.test/file' } })), /allowed HTTPS/);
  await assert.rejects(releaseResponse('https://github.com/file', async () => new Response(null, { status: 302, headers: { location: 'https://github.com/file' } })), /Too many/);
});

test('streamed download verifies exact bytes and removes partial staging', async () => fixture(async (root) => {
  const body = new ReadableStream({ start(controller) { controller.enqueue(bytes.subarray(0, 5)); controller.enqueue(bytes.subarray(5)); controller.close(); } });
  const downloaded = await downloadAsset(asset, { tempRoot: root, fetchImpl: async () => new Response(body) });
  assert.deepEqual(await fs.readFile(downloaded.file), bytes);
  assert.deepEqual(await fs.readdir(downloaded.folder), [asset.name]);
}));

test('corrupt, short, oversized and unavailable downloads never retain files', async () => fixture(async (root) => {
  for (const payload of [Buffer.alloc(bytes.length), bytes.subarray(0, -1), Buffer.concat([bytes, Buffer.from('extra')])]) {
    await assert.rejects(downloadAsset(asset, { tempRoot: root, fetchImpl: async () => new Response(payload) }));
    assert.deepEqual(await fs.readdir(root), []);
  }
  await assert.rejects(downloadAsset(asset, { tempRoot: root, fetchImpl: async () => new Response(null, { status: 404 }) }), /HTTP 404/);
  assert.deepEqual(await fs.readdir(root), []);
}));

test('interrupted streams remove unverified output', async () => fixture(async (root) => {
  const body = new ReadableStream({ start(controller) { controller.enqueue(bytes.subarray(0, 5)); controller.error(new Error('transport interrupted')); } });
  await assert.rejects(downloadAsset(asset, { tempRoot: root, fetchImpl: async () => new Response(body) }), /transport interrupted/);
  assert.deepEqual(await fs.readdir(root), []);
}));

test('invalid pinned metadata cannot escape temporary paths', async () => fixture(async (root) => {
  for (const mutation of [{ name: '../escape' }, { name: '..' }, { sha256: 'bad' }, { size: Number.MAX_SAFE_INTEGER }]) {
    await assert.rejects(downloadAsset({ ...asset, ...mutation }, { tempRoot: root, fetchImpl: async () => { throw new Error('must not fetch'); } }), /metadata/);
    assert.deepEqual(await fs.readdir(root), []);
  }
}));

test('help and incompatible install do not download or execute', async () => {
  const no = async () => { throw new Error('must not run'); };
  await main([], { log() {}, download: no, run: no });
  await assert.rejects(main(['install'], { platform: 'linux', arch: 'x64', glibc: '2.38', log() {}, download: no, run: no }), /glibc 2.39/);
});

test('download-only retains verified output without invoking installer', async () => fixture(async (root) => {
  const output = await main(['download'], { platform: 'win32', arch: 'x64', log() {}, download: () => downloadAsset(asset, { tempRoot: root, fetchImpl: async () => new Response(bytes) }), run: async () => { throw new Error('must not install'); } });
  assert.deepEqual(await fs.readFile(output), bytes);
}));

test('native installer receives verified file and failures clean it up', async () => fixture(async (root) => {
  let invoked = 0;
  for (const failure of [false, true]) {
    const invocation = main(['install'], {
      platform: 'win32', arch: 'x64', log() {},
      download: () => downloadAsset(asset, { tempRoot: root, fetchImpl: async () => new Response(bytes) }),
      run: async (file, args) => {
        invoked++; assert.deepEqual(args, []); assert.deepEqual(await fs.readFile(file), bytes);
        if (failure) throw new Error('installer cancelled');
      },
    });
    if (failure) await assert.rejects(invocation, /cancelled/); else await invocation;
    assert.deepEqual(await fs.readdir(root), []);
  }
  assert.equal(invoked, 2);
}));

test('Linux install uses argument arrays and cleans verified package', async () => fixture(async (root) => {
  await main(['install'], {
    platform: 'linux', arch: 'x64', glibc: '2.39', log() {},
    download: () => downloadAsset(asset, { tempRoot: root, fetchImpl: async () => new Response(bytes) }),
    run: async (command, args) => {
      assert.ok(['sudo', 'apt-get'].includes(command));
      assert.equal(args.at(-2), 'install');
      assert.deepEqual(await fs.readFile(args.at(-1)), bytes);
    },
  });
  assert.deepEqual(await fs.readdir(root), []);
}));
