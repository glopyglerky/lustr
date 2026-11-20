# Native generic application candidate

This experimental image extension keeps a concrete application tied to the
symbolic declaration that produced it. It records the stable template identity,
ordered type and typed const arguments, and concrete field roots. The original
native substitution routine must reproduce those fields. The original native
materializer then creates consumer instances; native layout, Copy, drop,
effective C ABI and region checks precede imported semantic publication.
Templates have no invented machine layout or ABI.

Every image's closed and symbolic definitions stage before any application's
materialization. Registered template rows remain occupied when an existing
sparse instance table gains imported declarations. This preserves the template
boundary and original instance identities.

The first unchanged producer using `Pair<i64>` in signatures and `Pair<u8>`
in a constant was refused by the earlier native image path; the separate Rust
oracle built its library. The current candidate builds and has executed four
native/oracle consumer pairs with matching output and status, including returns,
arguments, independent identities, constants, arrays, enums, and bool/char const
arguments. Six invalid consumers reject through the original native move/type
checks and the separate oracle. The bounded application gate also passes thirteen
checksum-valid malformed cases, later-image ABI refusal before semantic
publication, and installation into an existing sparse generic-instance table.
A source review approved the bounded implementation. Broader regression checks
remain in progress; these observations do not qualify the whole compiler.

The first candidate gate failed on an overly broad declaration preflight.
A later helper build failed on incorrect test API names. The next gate passed
the runtime/rejection pairs, then failed because its malformed field mutation
selected a forward node before reaching the expected substitution check.
Those failures remain retained and are not counted as successful full gates.

The source integration remains unadmitted. Full Rust, real core and no_std
runtime, generic bodies and predicates, native Cargo composition and equivalent
complete-work speed remain required. Existing host libc/libm/loader dependencies
and SDK compiler bootstrap/oracle use remain explicit.
