// GEDDES_BINDING can point to an installed package entry point or a compiled .node file.
const assert = require('node:assert/strict')
const fs = require('node:fs')
const os = require('node:os')
const path = require('node:path')
const geddes = require(process.env.GEDDES_BINDING || './index.js')
const corpus = path.resolve(__dirname, '../tests/data/formats')
const manifest = JSON.parse(fs.readFileSync(path.join(corpus, 'manifest.json'), 'utf8'))

for (const fixture of manifest.fixtures) {
  const source = path.resolve(corpus, fixture.path)
  const rows = fs.readFileSync(path.resolve(corpus, fixture.reference), 'utf8')
    .trim().split('\n').slice(1).map(line => line.split(',').map(Number))
  const options = { index: fixture.scan, block: fixture.block || undefined }
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

// Defaults and normalization must work through the installed package's exports.
assert.equal(typeof geddes.read, 'function')
assert.equal(typeof geddes.readBytes, 'function')
const xy = Buffer.from('10 4\n10.5 81\n')
const expected = { x: [10, 10.5], y: [4, 81] }
assert.deepEqual(geddes.readBytes(xy, 'example.xy'), expected)
assert.deepEqual(geddes.readBytes(xy, 'example.xy', {}), expected)
assert.deepEqual(geddes.readBytes(xy, 'example.xy', { index: 0 }), expected)
assert.deepEqual(geddes.readBytes(Buffer.from('10.5 81\n10 4\n'), 'descending.xy'), expected)
const firstSource = path.resolve(corpus, manifest.fixtures[0].path)
assert.deepEqual(geddes.read(firstSource), geddes.read(firstSource, {}))
assert.deepEqual(geddes.read(firstSource), geddes.read(firstSource, { index: 0 }))

// Distinct ranges detect lost or ignored selection arguments.
const ranges = Buffer.from([
  '_FILEVERSION=3', "_DRIVE='COUPLED'", '_START=10', '_STEPSIZE=0.5',
  '_COUNTS', '4 81', '_START=20', '_COUNTS', '144 9 1', ''
].join('\n'))
assert.deepEqual(geddes.readBytes(ranges, 'ranges.uxd'), expected)
const secondRange = {
  x: [20, 20.5, 21], y: [144, 9, 1]
}
assert.deepEqual(geddes.readBytes(ranges, 'ranges.uxd', { index: 1 }), secondRange)
assert.throws(() => geddes.readBytes(ranges, 'ranges.uxd', { index: 2 }), /out of range/)
assert.throws(() => geddes.readBytes(xy, 'example.xy', { index: 1 }), /index must be 0/)
const selectionDir = fs.mkdtempSync(path.join(os.tmpdir(), 'geddes-node-selection-'))
try {
  const rangesPath = path.join(selectionDir, 'ranges.uxd')
  fs.writeFileSync(rangesPath, ranges)
  assert.deepEqual(geddes.read(rangesPath), expected)
  assert.deepEqual(geddes.read(rangesPath, { index: 0 }), expected)
  assert.deepEqual(geddes.read(rangesPath, { index: 1 }), secondRange)
  assert.throws(() => geddes.read(rangesPath, { index: 2 }), /out of range/)
  assert.throws(() => geddes.read(rangesPath, { index: 'one' }), Error)
} finally {
  fs.rmSync(selectionDir, { recursive: true, force: true })
}

const blocks = Buffer.from([
  'data_sample_A', 'loop_', '_pd_meas_2theta_scan', '_pd_meas_intensity_total',
  '10 4', '10.5 81', 'data_sample_B', 'loop_',
  '_pd_meas_2theta_scan', '_pd_meas_intensity_total', '20 144', '20.5 9', ''
].join('\n'))
assert.deepEqual(geddes.readBytes(blocks, 'blocks.cif'), expected)
assert.deepEqual(geddes.readBytes(blocks, 'blocks.cif', { block: '_B' }), {
  x: [20, 20.5], y: [144, 9]
})
assert.throws(() => geddes.readBytes(blocks, 'blocks.cif', { block: '_b' }), /no matching block/)

// Reader failures must reach JavaScript as exceptions, not empty patterns.
for (const [data, filename, message] of [
  [Buffer.alloc(0), 'empty.xy', /Parse error:/],
  [Buffer.from('10 1\n10 2\n'), 'duplicate.xy', /strictly increasing/],
  [Buffer.from('10 NaN\n11 2\n'), 'nonfinite.xy', /finite/],
  [Buffer.from([0, 255, 0]), 'unknown.bin', /Unknown format/],
  [Buffer.from('PK\x03\x04'), 'truncated.rasx', /Zip error:/],
  [Buffer.from('RAW2\0\0\0\0'), 'old.raw', /unsupported/]
]) {
  assert.throws(() => geddes.readBytes(data, filename), message, filename)
}
const missing = path.join(corpus, 'missing-node-api-file.xy')
assert.equal(fs.existsSync(missing), false, 'missing-file test requires an absent path')
assert.throws(() => geddes.read(missing), /IO error:/)
assert.throws(() => geddes.read(123), Error)
assert.throws(() => geddes.readBytes('10 1\n11 2\n', 'wrong-type.xy'), Error)
assert.throws(() => geddes.readBytes(xy, 'wrong-option.xy', { index: 'one' }), Error)

console.log(`Node: ${manifest.fixtures.length} fixtures match every x/y value from paths and bytes; API selection and error checks passed`)
