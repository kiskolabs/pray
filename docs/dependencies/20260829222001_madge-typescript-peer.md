# madge TypeScript peerOptional vs Dependabot TypeScript 7

## Dependency

madge 8.0.0 (devDependency of npmjs/pray-cli). peerOptional typescript ^5.4.4. Dependabot PR 12 set typescript to ^7.0.2. Classification: dev or test only.

## Symptom

npm ci failed with ERESOLVE. The npm CI job never reached tests or npm audit.

## Evidence

CI run 33270173050, npm job. madge issue: https://github.com/pahen/madge/issues/462. After pinning typescript to ~5.9.3 and @types/node to ^22.15.0, npm ci succeeded. npm test: 83 pass, 0 fail. npm run lint passed. npm audit reported 0 vulnerabilities after overrides for js-yaml >=4.3.2, nanoid >=3.3.18, and postcss >=8.5.23.

@types/node ^26.2.0 compiled Buffer/chunk types that did not match Node 22 in .github/workflows/ci.yml (ssh-agent.ts). Pinning types to 22 removed that without a source change.

## Suggested fix

Pin TypeScript on 5.9 until madge publishes a peer that allows 6 or 7. Pin @types/node on 22 until CI node-version rises. Ignore Dependabot semver-major for both names. Do not introduce a second cycle checker.

## Next

Watch pahen/madge#462. Raise TypeScript when the peer range allows it. Raise @types/node when the workflow node-version rises.

## Source

https://github.com/pahen/madge/issues/462
Dependabot PR 12
.github/dependabot.yml ignore rules
npmjs/pray-cli/package.json
usr/docs/issues/20260829222000_ci-job-failures.md
