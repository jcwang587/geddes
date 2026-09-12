const assert = require('node:assert/strict')
const { execFileSync } = require('node:child_process')
const fs = require('node:fs')
const os = require('node:os')
const path = require('node:path')

assert(process.argv[2], 'Usage: npm run check:package -- PACKAGE_DIR')
assert(process.env.npm_execpath, 'Run this script through npm run check:package')
const packageDir = path.resolve(process.argv[2])
const packages = JSON.parse(fs.readFileSync(path.join(packageDir, 'packages.json'), 'utf8'))
const abi = `${process.platform}-${process.arch}${process.platform === 'win32' ? '-msvc' : process.platform === 'linux' ? '-gnu' : ''}`
const main = packages.find(pkg => pkg.target === null)
const native = packages.find(pkg => main && pkg.name === `${main.name}-${abi}`)
assert(main && native, `Missing release package for ${abi}`)
assert.equal(native.version, main.version)
const consumer = fs.mkdtempSync(path.join(os.tmpdir(), 'geddes-consumer-'))

try {
  fs.writeFileSync(path.join(consumer, 'package.json'), JSON.stringify({ private: true }))
  const env = { ...process.env, npm_config_cache: path.join(consumer, 'cache') }
  delete env.NODE_PATH
  delete env.NAPI_RS_NATIVE_LIBRARY_PATH
  delete env.GEDDES_BINDING
  execFileSync(process.execPath, [process.env.npm_execpath, 'install',
    '--offline', '--ignore-scripts', '--no-audit', '--no-fund', '--package-lock=false',
    path.join(packageDir, main.filename), path.join(packageDir, native.filename),
  ], { cwd: consumer, env, stdio: 'inherit' })

  const installed = path.join(consumer, 'node_modules', ...main.name.split('/'))
  const installedNative = path.join(consumer, 'node_modules', ...native.name.split('/'))
  const installedManifest = JSON.parse(fs.readFileSync(path.join(installed, 'package.json'), 'utf8'))
  assert.equal(installedManifest.version, main.version)
  assert.equal(installedManifest.optionalDependencies[native.name], native.version)
  assert(fs.existsSync(path.join(installed, installedManifest.types)), 'Missing TypeScript declarations')
  assert(fs.existsSync(path.join(installed, 'LICENSE')), 'Missing main package license')
  assert(fs.existsSync(path.join(installedNative, 'LICENSE')), 'Missing native package license')
  assert(!fs.readdirSync(installed).some(file => file.endsWith('.node')),
    'Test must load the installed platform package through the JS loader')

  execFileSync(process.execPath, [path.resolve(__dirname, '..', 'test.cjs')], {
    cwd: consumer,
    env: { ...env, GEDDES_BINDING: installed, NAPI_RS_ENFORCE_VERSION_CHECK: '1' },
    stdio: 'inherit',
  })
  console.log(`Installed package passed on ${abi}, Node ${process.version}`)
} finally {
  fs.rmSync(consumer, { recursive: true, force: true })
}
