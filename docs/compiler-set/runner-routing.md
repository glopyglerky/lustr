# Hosted-first runner routing

The workflows use standard GitHub-hosted runners by default. An absent or empty
`CI_RUNNER_ROUTES` repository variable selects those same hosted labels. A
trusted owner can publish a complete JSON map from a standard label to the
labels of a qualified replacement runner, for example:

```json
{"macos-26":["self-hosted","macOS","ARM64","qualified-macos-26"]}
```

Every element in a label array is required. This is one runner selector, not a
list of alternatives. The fallback must actually run the selected OS and
architecture, provide the workflow's tools, and enforce the admitted resource
and isolation limits. A Linux machine carrying a macOS label is not eligible.
Fork pull requests retain the standard hosted label and cannot select the
owner's private resources through this variable.

## What controls failover

The repository workflow does not infer runner health from a failing build.
An authenticated external owner must observe GitHub's acquisition failure,
verify that no runner or step executed, and retain the exact repository, run,
attempt and job identity. A normal compile failure, test failure, user
cancellation, or a workflow's own error message is not acquisition evidence.

When hosted capacity is unavailable, that owner can publish a matching admitted
pool route and request a bounded targeted retry. It retains uncertain API
delivery rather than sending duplicate retries. Fresh jobs follow the current
route. A queued job does not move merely because the variable changed.
Hosted-first policy restores the hosted selector when the outage observation
expires or fresh recovery evidence warrants it.

The retry API supplies no runner override. Live qualification must establish
that the selected workflow re-evaluates this repository variable on a rerun.
The owner must retain the original source and actor and explicitly qualify the
workflow and all dependent effects before automatic replay. Deployment or
publication workflows need their own idempotency qualification.

## Original source evidence

Each participating workflow has a `Runner source provenance` job. Its inline
script emits a `runner-source-v1` check annotation and uploads the same JSON as
`runner-source-<run>-<attempt>`. It captures the original event, source/ref,
workflow revision, actors, and pull-request head/base identities.

An owner must authenticate the GitHub Actions check, producer job, exact run and
attempt, and qualified workflow bytes. For a pull request, it must also verify
the immutable merge commit and its original parents. Current pull-request
metadata can change after a run and is not a substitute for this receipt. Old
runs without sufficient original evidence can remain held while a verified
platform outage still changes the route for future work.

## Routing canary

`runner-routing-canary.yml` records the actual environment, OS, architecture and
source identity. Its manual inputs select a platform and expected environment.
The optional first-attempt failure supports a controlled rerun exercise; it is
an intentional test failure and must never be classified as a hosted outage.

Qualify one hosted run, then one approved targeted rerun onto the matching
disposable pool, then hosted recovery. Check retained artifacts and authenticated
job assignment, not just workflow acceptance. Keep the canary receipt bound to
the exact admitted workflow digest. A successful canary alone does not qualify
the compiler gate or performance claims.

The main CI workflow additionally exercises portable process ownership, actual
local IPC, native ABI values, SDK identity and base-library lifetime on both
Linux and Apple silicon. See [host portability](host-portability.md) for the
runtime boundaries and the results that still require native execution.
