import assert from 'node:assert/strict'
import test from 'node:test'
import { expectedSourceChecks, validatePublishSource } from './validate-publish-source.mjs'

function source() {
  return {
    repository: 'jcwang587/geddes', version: '1.0.1', tagSha: 'release-commit',
    run: { id: 123, path: '.github/workflows/publish-npm.yml',
      repository: { full_name: 'jcwang587/geddes' }, head_repository: { full_name: 'jcwang587/geddes' },
      event: 'push', head_branch: 'v1.0.1', head_sha: 'release-commit', status: 'completed' },
    jobs: [...expectedSourceChecks.map(name => ({ name, conclusion: 'success' })),
      { name: 'publish', conclusion: 'failure' }],
    artifact: { name: 'node-packages', expired: false, digest: `sha256:${'a'.repeat(64)}`,
      workflow_run: { id: 123, head_sha: 'release-commit' } },
  }
}

test('accepts a failed publication after all original package checks succeeded', () => {
  assert.doesNotThrow(() => validatePublishSource(source()))
})

for (const [name, mutate] of [
  ['another workflow', data => { data.run.path = '.github/workflows/other.yml' }],
  ['a fork', data => { data.run.head_repository.full_name = 'other/geddes' }],
  ['another release tag', data => { data.run.head_branch = 'v1.0.0' }],
  ['a moved release tag', data => { data.tagSha = 'different-commit' }],
  ['an incomplete run', data => { data.run.status = 'in_progress' }],
  ['failed package checks', data => { data.jobs[1].conclusion = 'failure' }],
  ['skipped package checks', data => { data.jobs[1].conclusion = 'skipped' }],
  ['missing package checks', data => { data.jobs = [{ name: 'publish', conclusion: 'failure' }] }],
  ['a partial package-check matrix', data => { data.jobs.splice(3, 1) }],
  ['an expired artifact', data => { data.artifact.expired = true }],
  ['an artifact from another run', data => { data.artifact.workflow_run.id = 456 }],
  ['an artifact from another commit', data => { data.artifact.workflow_run.head_sha = 'different-commit' }],
]) {
  test(`rejects ${name}`, () => {
    const data = source()
    mutate(data)
    assert.throws(() => validatePublishSource(data))
  })
}
