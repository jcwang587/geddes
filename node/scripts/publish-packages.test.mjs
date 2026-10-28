import assert from 'node:assert/strict'
import { createHash } from 'node:crypto'
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import test from 'node:test'
import { planPublishes, publishPackages } from './publish-packages.mjs'

function fixture(t) {
  const root = mkdtempSync(join(tmpdir(), 'geddes-publish-test-'))
  t.after(() => rmSync(root, { recursive: true, force: true }))
  const directory = join(root, 'packages')
  mkdirSync(directory)
  const names = ['geddes-win32-x64-msvc', 'geddes-win32-arm64-msvc', 'geddes-darwin-arm64',
    'geddes-linux-x64-gnu', 'geddes-linux-arm64-gnu', 'geddes']
  const packages = names.map((name, index) => ({ name, version: '1.0.1', filename: `${name}.tgz`,
    target: index === 5 ? null : `target-${index}` }))
  const metadata = new Map()
  for (const pkg of packages) {
    const bytes = Buffer.from(`tested tarball for ${pkg.name}`)
    writeFileSync(join(directory, pkg.filename), bytes)
    metadata.set(pkg.name, { name: pkg.name, versions: {}, 'dist-tags': { latest: '1.0.0' },
      integrity: `sha512-${createHash('sha512').update(bytes).digest('base64')}` })
  }
  writeFileSync(join(directory, 'packages.json'), JSON.stringify(packages))
  writeFileSync(join(root, 'package.json'), JSON.stringify({ napi: { targets: packages.slice(0, 5).map(p => p.target) } }))
  const fetchMetadata = async url => ({ status: 200, json: async () => metadata.get(decodeURIComponent(url.split('/').at(-1))) })
  const markPublished = index => {
    const item = metadata.get(packages[index].name)
    item.versions['1.0.1'] = { dist: { integrity: item.integrity } }
    item['dist-tags'].latest = '1.0.1'
    return item
  }
  return { directory, packages, metadata, fetchMetadata, markPublished }
}

test('recovery skips matching Windows tarballs and publishes the remaining four with wrapper last', async t => {
  const f = fixture(t)
  f.markPublished(0)
  f.markPublished(1)
  const published = []
  await publishPackages(f.directory, { fetchMetadata: f.fetchMetadata, publish: entry => published.push(entry.pkg.name) })
  assert.deepEqual(published, f.packages.slice(2).map(pkg => pkg.name))
})

test('an integrity mismatch in a later package aborts before any publish', async t => {
  const f = fixture(t)
  f.markPublished(4).versions['1.0.1'].dist.integrity = 'sha512-different'
  let published = false
  await assert.rejects(publishPackages(f.directory, { fetchMetadata: f.fetchMetadata, publish: () => { published = true } }), /differs from the tested tarball/)
  assert.equal(published, false)
})

test('matching bytes with an incorrect latest tag require explicit repair', async t => {
  const f = fixture(t)
  f.markPublished(0)['dist-tags'].latest = '0.0.1-security'
  await assert.rejects(planPublishes(f.packages, f.directory, f.fetchMetadata), /repair the tag/)
})

test('published versions without integrity cannot be skipped', async t => {
  const f = fixture(t)
  delete f.markPublished(0).versions['1.0.1'].dist.integrity
  await assert.rejects(planPublishes(f.packages, f.directory, f.fetchMetadata), /differs from the tested tarball/)
})

for (const status of [401, 403, 404, 500]) {
  test(`registry HTTP ${status} aborts instead of treating the version as absent`, async t => {
    const f = fixture(t)
    await assert.rejects(planPublishes(f.packages, f.directory, async () => ({ status })), new RegExp(`HTTP ${status}`))
  })
}

test('network failure and malformed registry data abort', async t => {
  const f = fixture(t)
  await assert.rejects(planPublishes(f.packages, f.directory, async () => { throw new Error('network unavailable') }), /network unavailable/)
  delete f.metadata.get(f.packages[0].name).versions
  await assert.rejects(planPublishes(f.packages, f.directory, f.fetchMetadata), /Invalid registry versions/)
})
