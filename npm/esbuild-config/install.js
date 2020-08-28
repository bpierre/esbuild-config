// This is a slightly modified version of the esbuild install script:
// https://github.com/evanw/esbuild/blob/1336fbcf9bcca2f2708f5f575770f13a8440bde3/lib/install.ts

const fs = require('fs')
const os = require('os')
const path = require('path')
const zlib = require('zlib')
const https = require('https')
const child_process = require('child_process')

const version = require('./package.json').version
const binPath = path.join(__dirname, 'bin', 'esbuild-config')
const stampPath = path.join(__dirname, 'stamp.txt')

async function installBinaryFromPackage(name, fromPath, toPath) {
  // It turns out that some package managers (e.g. yarn) sometimes re-run the
  // postinstall script for this package after we have already been installed.
  // That means this script must be idempotent. Let's skip the install if it's
  // already happened.
  if (fs.existsSync(stampPath)) {
    return
  }

  // Try to install from the cache if possible
  const cachePath = getCachePath(name)
  try {
    // Copy from the cache
    fs.copyFileSync(cachePath, toPath)
    fs.chmodSync(toPath, 0o755)

    // Mark the cache entry as used for LRU
    const now = new Date()
    fs.utimesSync(cachePath, now, now)

    // Mark the operation as successful so this script is idempotent
    fs.writeFileSync(stampPath, '')
    return
  } catch {}

  // Next, try to install using npm. This should handle various tricky cases
  // such as environments where requests to npmjs.org will hang (in which case
  // there is probably a proxy and/or a custom registry configured instead).
  let buffer
  let didFail = false
  try {
    buffer = installUsingNPM(name, fromPath)
  } catch (err) {
    didFail = true
    console.error(`Trying to install "${name}" using npm`)
    console.error(
      `Failed to install "${name}" using npm: ${(err && err.message) || err}`
    )
  }

  // If that fails, the user could have npm configured incorrectly or could not
  // have npm installed. Try downloading directly from npm as a last resort.
  if (!buffer) {
    const url = `https://registry.npmjs.org/${name}/-/${name}-${version}.tgz`
    console.error(`Trying to download ${JSON.stringify(url)}`)
    try {
      buffer = extractFileFromTarGzip(await fetch(url), fromPath)
    } catch (err) {
      console.error(
        `Failed to download ${JSON.stringify(url)}: ${
          (err && err.message) || err
        }`
      )
    }
  }

  // Give up if none of that worked
  if (!buffer) {
    console.error(`Install unsuccessful`)
    process.exit(1)
  }

  // Write out the binary executable that was extracted from the package
  fs.writeFileSync(toPath, buffer, { mode: 0o755 })

  // Mark the operation as successful so this script is idempotent
  fs.writeFileSync(stampPath, '')

  // Also try to cache the file to speed up future installs
  try {
    fs.mkdirSync(path.dirname(cachePath), { recursive: true })
    fs.copyFileSync(toPath, cachePath)
    cleanCacheLRU(cachePath)
  } catch {}

  if (didFail) console.error(`Install successful`)
}

function getCachePath(name) {
  const home = os.homedir()
  const common = ['esbuild-config', 'bin', `${name}@${version}`]
  if (process.platform === 'darwin')
    return path.join(home, 'Library', 'Caches', ...common)
  if (process.platform === 'win32')
    return path.join(home, 'AppData', 'Local', 'Cache', ...common)
  return path.join(home, '.cache', ...common)
}

function cleanCacheLRU(fileToKeep) {
  // Gather all entries in the cache
  const dir = path.dirname(fileToKeep)
  const entries = []
  for (const entry of fs.readdirSync(dir)) {
    const entryPath = path.join(dir, entry)
    try {
      const stats = fs.statSync(entryPath)
      entries.push({ path: entryPath, mtime: stats.mtime })
    } catch {}
  }

  // Only keep the most recent entries
  entries.sort((a, b) => +b.mtime - +a.mtime)
  for (const entry of entries.slice(5)) {
    try {
      fs.unlinkSync(entry.path)
    } catch {}
  }
}

function fetch(url) {
  return new Promise((resolve, reject) => {
    https
      .get(url, (res) => {
        if (
          (res.statusCode === 301 || res.statusCode === 302) &&
          res.headers.location
        )
          return fetch(res.headers.location).then(resolve, reject)
        if (res.statusCode !== 200)
          return reject(new Error(`Server responded with ${res.statusCode}`))
        let chunks = []
        res.on('data', (chunk) => chunks.push(chunk))
        res.on('end', () => resolve(Buffer.concat(chunks)))
      })
      .on('error', reject)
  })
}

function extractFileFromTarGzip(buffer, file) {
  try {
    buffer = zlib.unzipSync(buffer)
  } catch (err) {
    throw new Error(
      `Invalid gzip data in archive: ${(err && err.message) || err}`
    )
  }
  let str = (i, n) =>
    String.fromCharCode(...buffer.subarray(i, i + n)).replace(/\0.*$/, '')
  let offset = 0
  file = `package/${file}`
  while (offset < buffer.length) {
    let name = str(offset, 100)
    let size = parseInt(str(offset + 124, 12), 8)
    offset += 512
    if (!isNaN(size)) {
      if (name === file) return buffer.subarray(offset, offset + size)
      offset += (size + 511) & ~511
    }
  }
  throw new Error(`Could not find ${JSON.stringify(file)} in archive`)
}

function installUsingNPM(name, file) {
  const installDir = path.join(__dirname, '.install')
  fs.mkdirSync(installDir)
  fs.writeFileSync(path.join(installDir, 'package.json'), '{}')

  // Erase "npm_config_global" so that "npm install --global esbuild-config"
  // works. Otherwise this nested "npm install" will also be global, and the
  // install will deadlock waiting for the global installation lock.
  const env = { ...process.env, npm_config_global: undefined }

  child_process.execSync(
    `npm install --loglevel=error --prefer-offline --no-audit --progress=false ${name}@${version}`,
    { cwd: installDir, stdio: 'pipe', env }
  )
  const buffer = fs.readFileSync(
    path.join(installDir, 'node_modules', name, file)
  )
  removeRecursive(installDir)
  return buffer
}

function removeRecursive(dir) {
  for (const entry of fs.readdirSync(dir)) {
    const entryPath = path.join(dir, entry)
    let stats
    try {
      stats = fs.lstatSync(entryPath)
    } catch (e) {
      continue // Guard against https://github.com/nodejs/node/issues/4760
    }
    if (stats.isDirectory()) removeRecursive(entryPath)
    else fs.unlinkSync(entryPath)
  }
  fs.rmdirSync(dir)
}

function installOnUnix(name) {
  installBinaryFromPackage(name, 'bin/esbuild-config', binPath).catch((e) =>
    setImmediate(() => {
      throw e
    })
  )
}

function installOnWindows(name) {
  fs.writeFileSync(
    binPath,
    `#!/usr/bin/env node
const path = require('path');
const esbuild_config_exe = path.join(__dirname, '..', 'esbuild-config.exe');
const child_process = require('child_process');
child_process.spawnSync(esbuild_config_exe, process.argv.slice(2), { stdio: 'inherit' });
`
  )
  const exePath = path.join(__dirname, 'esbuild-config.exe')
  installBinaryFromPackage(name, 'esbuild-config.exe', exePath).catch((e) =>
    setImmediate(() => {
      throw e
    })
  )
}

const key = `${process.platform} ${os.arch()} ${os.endianness()}`
const knownWindowsPackages = {
  'win32 x64 LE': 'esbuild-config-windows-64',
}
const knownUnixlikePackages = {
  'darwin x64 LE': 'esbuild-config-darwin-64',
  'linux x64 LE': 'esbuild-config-linux-64',
}

// Pick a package to install
if (key in knownWindowsPackages) {
  installOnWindows(knownWindowsPackages[key])
} else if (key in knownUnixlikePackages) {
  installOnUnix(knownUnixlikePackages[key])
} else {
  console.error(`Unsupported platform: ${key}`)
  process.exit(1)
}
