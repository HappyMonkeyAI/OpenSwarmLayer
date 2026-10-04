'use strict';
const fs = require('node:fs/promises');
const os = require('node:os');
const path = require('node:path');
const crypto = require('node:crypto');
const { spawn } = require('node:child_process');
const release = require('../release.json');

const HELP = `OpenSwarmLayer ${release.tag} - technical test beta; not ready for production.

Usage: openswarmlayer install [--format exe|deb]
       openswarmlayer download [--format exe|deb|appimage]
       openswarmlayer --help

Windows x64 defaults to the normal unsigned EXE installer.
Linux x64 defaults to DEB with apt/sudo; installation requires glibc 2.39+.
AppImage is download-only; make it executable and run it yourself.
No app is installed when you merely install this npm package or request help.
The wrapper requires Node.js 20+ and downloads pinned native GitHub release files.
`;

function parseArgs(args) {
  if (!args.length || (args.length === 1 && ['--help', '-h'].includes(args[0]))) return { action: 'help' };
  if (!['install', 'download'].includes(args[0])) throw new Error('Choose install or download. Use --help for instructions.');
  let format;
  for (let i = 1; i < args.length; i++) {
    if (args[i] !== '--format' || format || !['exe', 'deb', 'appimage'].includes(args[i + 1])) {
      throw new Error('Unknown or incomplete argument. Use --help for instructions.');
    }
    format = args[++i];
  }
  if (args[0] === 'install' && format === 'appimage') throw new Error('Use download --format appimage, then run the portable app manually.');
  return { action: args[0], format };
}

function selectAsset(format, platform = process.platform, arch = process.arch) {
  if (arch !== 'x64' || !['win32', 'linux'].includes(platform)) throw new Error('Only Windows x64 and Linux x64 releases are available.');
  format = format || (platform === 'win32' ? 'exe' : 'deb');
  const asset = release.assets[format];
  if (!asset || asset.platform !== platform) throw new Error('That package format is not available for this operating system.');
  return asset;
}

function checkLinuxInstall(runtimeGlibc) {
  if (!/^\d+\.\d+$/.test(runtimeGlibc || '')) throw new Error('Linux installation requires a glibc system (2.39+); musl is unsupported.');
  const [major, minor] = runtimeGlibc.split('.').map(Number);
  if (major < 2 || (major === 2 && minor < 39)) throw new Error(`This native beta requires glibc 2.39+, but this system has ${runtimeGlibc}.`);
}

function checkedUrl(value) {
  const url = new URL(value);
  const allowedHosts = ['github.com', 'release-assets.githubusercontent.com', 'objects.githubusercontent.com'];
  if (url.protocol !== 'https:' || url.username || url.password || (url.port && url.port !== '443') || !allowedHosts.includes(url.hostname)) {
    throw new Error('Release download redirected outside the allowed HTTPS hosts.');
  }
  return url;
}

async function releaseResponse(url, fetchImpl = fetch) {
  for (let redirects = 0; redirects <= 5; redirects++) {
    const current = checkedUrl(url);
    const response = await fetchImpl(current, {
      redirect: 'manual', signal: AbortSignal.timeout(600000),
      headers: { 'User-Agent': 'OpenSwarmLayer-beta-installer', 'Accept-Encoding': 'identity' },
    });
    if ([301, 302, 303, 307, 308].includes(response.status)) {
      const location = response.headers.get('location');
      if (response.body) await response.body.cancel();
      if (!location) throw new Error('Release redirect has no destination.');
      url = new URL(location, current).href;
      continue;
    }
    if (!response.ok) { if (response.body) await response.body.cancel(); throw new Error(`Release download returned HTTP ${response.status}.`); }
    return response;
  }
  throw new Error('Too many release download redirects.');
}

async function downloadAsset(asset, { fetchImpl = fetch, tempRoot = os.tmpdir() } = {}) {
  if (!/^OpenSwarmLayer-[A-Za-z0-9._-]+$/.test(asset.name) || !/^[a-f0-9]{64}$/.test(asset.sha256) || !Number.isSafeInteger(asset.size) || asset.size <= 0 || asset.size > 256 * 1024 * 1024) {
    throw new Error('Invalid pinned release metadata.');
  }
  const folder = await fs.mkdtemp(path.join(tempRoot, 'OpenSwarmLayer-npx-'));
  const partial = path.join(folder, `${asset.name}.partial`);
  const output = path.join(folder, asset.name);
  let file;
  try {
    const url = `https://github.com/SPhillips1337/OpenSwarmLayer/releases/download/${release.tag}/${asset.name}`;
    const response = await releaseResponse(url, fetchImpl);
    if (!response.body) throw new Error('Release download has no body.');
    file = await fs.open(partial, 'wx', 0o600);
    const digest = crypto.createHash('sha256');
    let size = 0;
    for await (const bytes of response.body) {
      size += bytes.length;
      if (size > asset.size) throw new Error('Release download exceeds its pinned size.');
      digest.update(bytes);
      // FileHandle.write can write fewer bytes than requested.
      let offset = 0;
      while (offset < bytes.length) {
        const written = await file.write(bytes, offset, bytes.length - offset);
        if (!written.bytesWritten) throw new Error('Could not write the downloaded package.');
        offset += written.bytesWritten;
      }
    }
    if (size !== asset.size || digest.digest('hex') !== asset.sha256) throw new Error('Release size/checksum mismatch; installation refused.');
    await file.close(); file = undefined;
    await fs.rename(partial, output);
    return { file: output, folder };
  } catch (error) {
    if (file) await file.close();
    await fs.unlink(partial).catch(() => {});
    await fs.rmdir(folder).catch(() => {});
    throw error;
  }
}

function runProcess(command, args) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, { stdio: 'inherit', shell: false });
    child.once('error', reject);
    child.once('close', (code, signal) => code === 0 ? resolve() : reject(new Error(`Installer exited ${signal || code}.`)));
  });
}

async function main(args, dependencies = {}) {
  const options = parseArgs(args);
  const log = dependencies.log || console.log;
  if (options.action === 'help') { log(HELP); return; }
  const platform = dependencies.platform || process.platform;
  const asset = selectAsset(options.format, platform, dependencies.arch || process.arch);
  if (options.action === 'install' && platform === 'linux') {
    checkLinuxInstall(dependencies.glibc || process.report.getReport().header.glibcVersionRuntime);
  }
  log(`OpenSwarmLayer ${release.tag} - technical test beta; not ready for production.`);
  log(platform === 'win32' ? 'Unsigned Windows package. Close the app before installing.' : 'Experimental Linux package; requires glibc 2.39+. Native desktop acceptance remains incomplete.');
  const downloaded = await (dependencies.download || downloadAsset)(asset);
  log(`SHA-256 verified: ${asset.name}`);
  if (options.action === 'download') { log(`Verified package saved to ${downloaded.file}`); return downloaded.file; }
  try {
    const run = dependencies.run || runProcess;
    if (platform === 'win32') await run(downloaded.file, []);
    else {
      await fs.chmod(downloaded.folder, 0o755);
      await fs.chmod(downloaded.file, 0o644);
      const root = process.getuid && process.getuid() === 0;
      await run(root ? 'apt-get' : 'sudo', root ? ['install', downloaded.file] : ['apt-get', 'install', downloaded.file]);
    }
    log('Installer finished. Launch OpenSwarmLayer to begin your test.');
  } finally {
    await fs.unlink(downloaded.file);
    await fs.rmdir(downloaded.folder);
  }
}

module.exports = { main, parseArgs, selectAsset, checkLinuxInstall, checkedUrl, releaseResponse, downloadAsset };
