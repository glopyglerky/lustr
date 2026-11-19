# Native admission and incomplete scalar integration

The [exclusive coverage contract](exclusive-coverage-contract.md) governs this
boundary. The retained scalar object bridge admits work from real native
checked state before C generation. Its representation is narrower than Lust's
supported set. A gap in that bridge remains unfinished integration.

The general mixed compiler launcher has been retired. This document describes
the remaining bounded native component, not a complete mixed compiler or a
Rust replacement package.

## Correct the previous classification

The prior implementation returned successful `Complementary` replies when a
checked native body needed 128-bit helpers, unit values, a wider ABI or another
request mode absent from the scalar bridge. That incorrectly treated the
bridge's represented set as Lust's supported set. Ordinary native compilation
already handles examples from those classes.

The current outcome is `IntegrationIncomplete`, with status 4. It does not
declare the source invalid, classify the work as absent from Lust, or authorize
another compiler to take ownership. Unqualified targets are also bridge gaps
at this interface: a target check alone is not a complete native capability
proof. No existing producer establishes a genuine complementary capability.

## Preserve one checked invocation

The request runs the existing Lust frontend, body checking, borrow checking and
lowering. [`mixed_unit::prepare`](../../crates/lustr-native-core/src/mixed_unit.rs)
examines that invocation's actual program, selected definition, stable slot,
signature and lowered IR before generation.

An admitted unit receives a private, non-cloneable `PreparedMixed` that borrows
the same checked program. Emission consumes it once. A check returns a checked
outcome without generating C. The boundary proof remains tied to definition,
body, signature and native dependency reads. Incomplete integration retains
passing native state without publishing an object. Repeating the request cannot
turn a cached integration gap into success.

## Current outcomes

| Wire outcome | Status | Meaning |
| --- | --- | --- |
| `mixed-v1:checked` | 0 | Native checking and the represented scalar boundary passed for this check. |
| `mixed-v1:emitted` | 0 | The admitted definition produced its validated native object. |
| `mixed-v1:integration-incomplete` | 4 | The bridge cannot compose this request. Compilation remains incomplete and ownership is unchanged. |
| `mixed-v1:failed` | Nonzero | The frontend rejected the request or a compiler, emitter, transport or artifact operation failed. The status retains its original meaning. |

The protocol reader rejects inconsistent statuses and the obsolete
`mixed-v1:complementary` tag. The compiler build identity separates incompatible
clients, retained sessions and artifacts. Ordinary replies retain their existing
four-field format. Diagnostic text is never a routing decision.

The actual complement must eventually use evidence about operations absent
from Lust in their requested context, justified imported implementation and
explicit semantic ownership. It cannot be inferred from these bridge refusals.

## Verification and earlier evidence

Focused controls compile and execute division, 128-bit arithmetic and unit-value
programs with ordinary Lust, then require incomplete-integration status for the
same bodies at the scalar bridge. They also cover repeated check/build requests,
request-mode gaps, owned preparation, rejected input, emission failures and
strict wire validation. Results require execution at the reported source; test
presence alone is not qualification.

The [earlier admission report](https://github.com/glopyglerky/lustr/blob/ec99578d51c93674af3a1bcf9c182f4ad4b17d89/docs/compiler-set/native-admission.md)
and its [original verification](evidence/native-admission-20261118/verification.json)
remain historical evidence. Their successful complementary outcomes describe
the retired classification. They do not authorize current compiler ownership
or establish full Lust integration, general Rust coverage, metadata completion
or near-stock performance.
