// Run after building the binding, or set GEDDES_BINDING to a compiled .node file.
const assert = require('node:assert/strict')
const fs = require('node:fs')
const path = require('node:path')
const geddes = require(process.env.GEDDES_BINDING || './index.js')
const corpus = path.resolve(__dirname, '../tests/data/formats')
const manifest = JSON.parse(fs.readFileSync(path.join(corpus, 'manifest.json'), 'utf8'))

for (const fixture of manifest.fixtures) {
  const source = path.resolve(corpus, fixture.path)
  const rows = fs.readFileSync(path.resolve(corpus, fixture.reference), 'utf8')
    .trim().split('\n').slice(1).map(line => line.split(',').map(Number))
  const options = { scan: fixture.scan, block: fixture.block || undefined }
  for (const pattern of [geddes.read(source, options), geddes.readBytes(fs.readFileSync(source), source, options)]) {
    assert.deepEqual(Object.keys(pattern).sort(), ['x', 'y'])
    assert.equal(pattern.x.length, rows.length, fixture.id)
    assert.equal(pattern.y.length, rows.length, fixture.id)
    for (let i = 0; i < rows.length; i++) {
      for (const [column, expected] of [['x', rows[i][0]], ['y', rows[i][1]]]) {
        assert.ok(Math.abs(pattern[column][i] - expected) <= fixture.atol + fixture.rtol * Math.abs(expected),
          `${fixture.id}: ${column}[${i}]`)
      }
    }
  }
}
assert.throws(() => geddes.readBytes(Buffer.from('10 1\n11 2\n'), 'a.xy', { scan: 1 }))
console.log(`Node: ${manifest.fixtures.length} fixtures match every x/y value from paths and bytes`)
