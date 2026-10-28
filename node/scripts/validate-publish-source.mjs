import assert from 'node:assert/strict'
import { appendFileSync, readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

// Keep this set aligned with the complete matrix in check-node.yml.
export const expectedSourceChecks = [
  'check / version', 'check / pack',
  'check / build (windows-latest, x86_64-pc-windows-msvc)',
  'check / build (windows-11-arm, aarch64-pc-windows-msvc)',
  'check / build (macos-latest, aarch64-apple-darwin)',
  'check / build (ubuntu-latest, x86_64-unknown-linux-gnu, --use-napi-cross)',
  'check / build (ubuntu-latest, aarch64-unknown-linux-gnu, --use-napi-cross)',
  ...['ubuntu-latest', 'ubuntu-24.04-arm', 'macos-latest', 'windows-latest']
    .flatMap(host => ['16', '24'].map(version => `check / install-test (${host}, ${version})`)),
  'check / install-test (windows-11-arm, 20)', 'check / install-test (windows-11-arm, 24)',
]

export function validatePublishSource({ run, jobs, artifact, repository, version, tagSha }) {
  assert.equal(run.path, '.github/workflows/publish-npm.yml', 'Source must be the npm publishing workflow')
  assert.equal(run.repository.full_name, repository, 'Source repository differs')
  assert.equal(run.head_repository.full_name, repository, 'Source cannot be a fork')
  assert.equal(run.event, 'push', 'Source must be a tag-triggered release run')
  assert.equal(run.head_branch, `v${version}`, 'Source tag differs from the current package version')
  assert.equal(run.head_sha, tagSha, 'Source commit differs from the release tag')
  assert.equal(run.status, 'completed', 'Source run must be completed')
  const checks = jobs.filter(job => job.name !== 'publish')
  assert.deepEqual(checks.map(job => job.name).sort(), [...expectedSourceChecks].sort(),
    'Source must contain the complete expected package-check matrix')
  assert(checks.every(job => job.conclusion === 'success'),
    'Every source package check must have succeeded')
  assert.equal(artifact.name, 'node-packages', 'Unexpected source artifact')
  assert.equal(artifact.expired, false, 'Source artifact has expired')
  assert.equal(artifact.workflow_run.id, run.id, 'Artifact belongs to another run')
  assert.equal(artifact.workflow_run.head_sha, tagSha, 'Artifact commit differs from the release tag')
  assert.match(artifact.digest, /^sha256:[a-f0-9]{64}$/, 'Source artifact must have a SHA-256 digest')
}

async function main(runId) {
  assert.match(runId ?? '', /^[1-9][0-9]*$/, 'Source run ID must be a positive integer')
  const repository = process.env.GITHUB_REPOSITORY
  assert(repository && process.env.GH_TOKEN, 'GitHub repository and token are required')
  const { version } = JSON.parse(readFileSync('node/package.json', 'utf8'))
  const api = async endpoint => {
    const response = await fetch(`https://api.github.com/repos/${repository}/${endpoint}`, {
      headers: { authorization: `Bearer ${process.env.GH_TOKEN}`, accept: 'application/vnd.github+json',
        'X-GitHub-Api-Version': '2022-11-28' },
      signal: AbortSignal.timeout(30_000),
    })
    assert.equal(response.status, 200, `Cannot verify source release: GitHub HTTP ${response.status}`)
    return response.json()
  }
  const list = async (endpoint, key) => {
    const items = []
    for (let page = 1; ; page++) {
      const data = await api(`${endpoint}?per_page=100&page=${page}`)
      items.push(...data[key])
      if (items.length >= data.total_count) return items
      assert(data[key].length > 0, 'Incomplete GitHub pagination')
    }
  }
  const [run, jobs, artifacts, tag] = await Promise.all([
    api(`actions/runs/${runId}`),
    list(`actions/runs/${runId}/jobs`, 'jobs'),
    list(`actions/runs/${runId}/artifacts`, 'artifacts'),
    api(`commits/${encodeURIComponent(`v${version}`)}`),
  ])
  assert.equal(run.id, Number(runId), 'GitHub returned another source run')
  const matches = artifacts.filter(artifact => artifact.name === 'node-packages')
  assert.equal(matches.length, 1, 'Expected exactly one checked node-packages artifact')
  validatePublishSource({ run, jobs, artifact: matches[0], repository, version, tagSha: tag.sha })
  assert(process.env.GITHUB_OUTPUT, 'GitHub Actions output file is required')
  appendFileSync(process.env.GITHUB_OUTPUT, `artifact_id=${matches[0].id}\n`)
  console.log(`Verified checked artifact ${matches[0].id} from ${repository} run ${runId} at ${tag.sha}`)
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  await main(process.argv[2])
}
