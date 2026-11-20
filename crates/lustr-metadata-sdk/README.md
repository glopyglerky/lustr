# Metadata SDK host adapter

This host adapter loads the original-reader construction receipt for the
selective retained native artifact service and its separate research qualifier. It uses `std` filesystem/path APIs,
JSON and SHA256 to verify the exact ordered public `core` and private
`compiler_builtins` context, both metadata/code artifact pairs, and the original
metadata header/version/hash. It returns immutable facts to the separate
`core`/`alloc` metadata serializer.

The caller must bind the receipt to the reviewed output of
`tools/build-metadata-sdk-receipt.py`. `open_bound` verifies the expected digest
on the same single read that supplies its parsed facts. It does not verify a
path and then reread a potentially different receipt. Arbitrary supplied JSON is not an SDK
certificate, and matching artifact digests do not establish a full Rust compiler.
The constructor uses the pinned original reader and a fixed reference at
construction time. This loader invokes no compiler and reads no user source.
The context should be opened once and retained; per-request metadata encoding
does not repeat SDK hashing or filesystem work.

The exclusive-native product feature declares this selective host dependency;
its public integration is authored and awaiting exact-source qualification. Its
host dependencies do not enter `lustr-metadata-encoder`'s `no_std` library graph. The
complete native compiler and its retained transport still use `std` and Unix
facilities; this bounded separation does not claim their migration.
