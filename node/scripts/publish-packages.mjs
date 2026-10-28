import assert from 'node:assert/strict'
import { execFileSync } from 'node:child_process'
import { createHash } from 'node:crypto'
import { readFileSync } from 'node:fs'
import { join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

export async function planPublishes(packages, directory, fetchMetadata = fetch) {
  const plan = []
  for (const pkg of packages) {
    const tarball = join(directory, pkg.filename)
    const integrity = `sha512-${createHash('sha512').update(readFileSync(tarball)).digest('base64')}`
    const tag = pkg.version.includes('-') ? 'next' : 'latest'
    const response = await fetchMetadata(`https://registry.npmjs.org/${encodeURIComponent(pkg.name)}`, {
      headers: { accept: 'application/json' },
      signal: AbortSignal.timeout(30_000),
    })
    assert.equal(response.status, 200,
      `Cannot verify existing npm package ${pkg.name}: HTTP ${response.status}; resolve registry access before retrying`)
    const metadata = await response.json()
    assert.equal(metadata.name, pkg.name, `Unexpected registry package for ${pkg.name}`)
    assert(metadata.versions && typeof metadata.versions === 'object' && !Array.isArray(metadata.versions),
      `Invalid registry versions for ${pkg.name}`)
    const published = metadata.versions[pkg.version]
    if (published !== undefined) {
      assert.equal(published?.dist?.integrity, integrity,
        `${pkg.name}@${pkg.version} differs from the tested tarball; refusing to skip or overwrite it`)
      assert.equal(metadata['dist-tags']?.[tag], pkg.version,
        `${pkg.name}@${pkg.version} has matching bytes but the ${tag} tag differs; repair the tag before retrying`)
    }
    plan.push({ pkg, tarball, tag, skip: published !== undefined })
  }
  return plan
}

export async function publishPackages(directory, { fetchMetadata = fetch, publish } = {}) {
  const packages = JSON.parse(readFileSync(join(directory, 'packages.json'), 'utf8'))
  const { napi } = JSON.parse(readFileSync(resolve(directory, '..', 'package.json'), 'utf8'))
  assert(packages.length === napi.targets.length + 1 && packages.at(-1).target === null,
    `Expected ${napi.targets.length} native packages followed by the JS package`)
  // Verify every registry entry before any publication can have a side effect.
  const plan = await planPublishes(packages, directory, fetchMetadata)
  const publishTarball = publish ?? (({ tarball, tag }) => {
    execFileSync(process.execPath, [process.env.npm_execpath, 'publish', tarball,
      '--ignore-scripts', '--provenance', '--access', 'public', '--tag', tag,
    ], { stdio: 'inherit' })
  })
  for (const entry of plan) {
    if (entry.skip) {
      console.log(`Already published with matching integrity: ${entry.pkg.name}@${entry.pkg.version}`)
    } else {
      await publishTarball(entry)
    }
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  assert(process.argv[2], 'Usage: npm run publish:packages -- PACKAGE_DIR')
  assert(process.env.npm_execpath, 'Run this script through npm run publish:packages')
  await publishPackages(resolve(process.argv[2]))
}
