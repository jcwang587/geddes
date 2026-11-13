import assert from 'node:assert/strict'
import { execFileSync } from 'node:child_process'
import { copyFileSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { NapiCli, parseTriple } from '@napi-rs/cli'

const nodeDir = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const [artifactArg, outputArg, localTarget] = process.argv.slice(2)
assert(artifactArg && outputArg, 'Usage: npm run pack:release -- ARTIFACT_DIR OUTPUT_DIR [LOCAL_TARGET]')
assert(process.env.npm_execpath, 'Run this script through npm run pack:release')
const artifactDir = resolve(artifactArg)
const outputDir = resolve(outputArg)
const pkg = JSON.parse(readFileSync(join(nodeDir, 'package.json'), 'utf8'))
const targets = localTarget ? [localTarget] : pkg.napi.targets
assert(targets.every(target => pkg.napi.targets.includes(target)), 'Unknown native target')
// Users install the main package alone. It lists one platform package per
// target as an optional dependency, named by napi.packageName under its scope.
assert.match(pkg.napi.packageName ?? '', /^@[^/]+\/[^/]+$/, 'napi.packageName must be a scoped package name')
const platformPackage = target => `${pkg.napi.packageName}-${parseTriple(target).platformArchABI}`
const staging = mkdtempSync(join(tmpdir(), 'geddes-pack-'))
mkdirSync(outputDir, { recursive: true })

try {
  pkg.optionalDependencies = Object.fromEntries(pkg.napi.targets.map(target => [platformPackage(target), pkg.version]))
  writeFileSync(join(staging, 'package.json'), JSON.stringify(pkg, null, 2) + '\n')
  copyFileSync(join(nodeDir, '..', 'README.md'), join(staging, 'README.md'))
  copyFileSync(join(nodeDir, '..', 'LICENSE'), join(staging, 'LICENSE'))
  await new NapiCli().createNpmDirs({ cwd: staging })

  const packages = []
  let wrapper
  let types
  function pack(directory, target) {
    const result = JSON.parse(execFileSync(process.execPath, [process.env.npm_execpath,
      'pack', '--json', '--ignore-scripts', '--pack-destination', outputDir,
    ], { cwd: directory, encoding: 'utf8' }))[0]
    assert(result.files.some(file => file.path === 'LICENSE'), 'Package is missing LICENSE')
    packages.push({ target, name: result.name, version: result.version, filename: result.filename })
  }

  for (const target of targets) {
    const abi = parseTriple(target).platformArchABI
    const built = join(artifactDir, `bindings-${target}`)
    const generatedWrapper = readFileSync(join(built, 'index.js'), 'utf8')
    const generatedTypes = readFileSync(join(built, 'index.d.ts'), 'utf8')
    if (wrapper !== undefined) {
      assert.equal(generatedWrapper, wrapper, `Generated loader differs for ${target}`)
      assert.equal(generatedTypes, types, `Generated types differ for ${target}`)
    }
    wrapper = generatedWrapper
    types = generatedTypes
    const nativeDir = join(staging, 'npm', abi)
    const manifest = JSON.parse(readFileSync(join(nativeDir, 'package.json'), 'utf8'))
    assert.equal(manifest.name, platformPackage(target), `Unexpected package name for ${target}`)
    assert.equal(manifest.publishConfig?.access, 'public', `${manifest.name} must be published with public access`)
    // napi's readme calls this the binary of napi.packageName, which is no
    // package: name the package users install.
    writeFileSync(join(nativeDir, 'README.md'), `# \`${manifest.name}\`\n\n` +
      `This is the **${target}** binary for \`${pkg.name}\`. Install \`${pkg.name}\`, ` +
      `and npm installs this package with it on that platform.\n`)
    const binary = `${pkg.napi.binaryName}.${abi}.node`
    copyFileSync(join(built, binary), join(nativeDir, binary))
    copyFileSync(join(staging, 'LICENSE'), join(nativeDir, 'LICENSE'))
    pack(nativeDir, target)
  }

  for (const name of Object.keys(pkg.optionalDependencies)) {
    assert(wrapper.includes(`require('${name}')`), `Generated loader does not require ${name}`)
  }
  writeFileSync(join(staging, 'index.js'), wrapper)
  writeFileSync(join(staging, 'index.d.ts'), types)
  pack(staging, null)
  writeFileSync(join(outputDir, 'packages.json'), JSON.stringify(packages, null, 2) + '\n')
  console.log(`Packed ${packages.length} release packages in ${outputDir}`)
} finally {
  rmSync(staging, { recursive: true, force: true })
}
