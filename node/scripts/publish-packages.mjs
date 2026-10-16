import assert from 'node:assert/strict'
import { execFileSync } from 'node:child_process'
import { readFileSync } from 'node:fs'
import { join, resolve } from 'node:path'

assert(process.argv[2], 'Usage: npm run publish:packages -- PACKAGE_DIR')
assert(process.env.npm_execpath, 'Run this script through npm run publish:packages')
const directory = resolve(process.argv[2])
const packages = JSON.parse(readFileSync(join(directory, 'packages.json'), 'utf8'))
const { napi } = JSON.parse(readFileSync(resolve(directory, '..', 'package.json'), 'utf8'))
assert(packages.length === napi.targets.length + 1 && packages.at(-1).target === null,
  `Expected ${napi.targets.length} native packages followed by the JS package`)
for (const pkg of packages) {
  execFileSync(process.execPath, [process.env.npm_execpath, 'publish', join(directory, pkg.filename),
    '--ignore-scripts', '--provenance', '--access', 'public',
    '--tag', pkg.version.includes('-') ? 'next' : 'latest',
  ], { stdio: 'inherit' })
}
