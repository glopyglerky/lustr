#!/usr/bin/env bash
# Regenerate all native standard-library Unicode data from the exact imported
# Rust SDK. A compiler build never runs this script or reads its host's tables.
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
rustc_bin=${RUSTC:-rustc}
rust_source_pin=5b7b37aefcb5b2fba3c2392bc9065b9e4602d494
mode=${1:-write}
if [[ $# -gt 1 || ( "$mode" != write && "$mode" != --check ) ]]; then
    echo "usage: gen-unicode.sh [--check]" >&2
    exit 2
fi
actual=$("$rustc_bin" -vV | sed -n 's/^commit-hash: //p')
if [[ "$actual" != "$rust_source_pin" ]]; then
    echo "gen-unicode: expected Rust source $rust_source_pin, got $actual" >&2
    exit 1
fi
rg -Fqx "source = \"$rust_source_pin\"" "$root/../../upstream/bluefe-core.pin"
LUSTR_UNICODE_RUST_SOURCE="$rust_source_pin" "$rustc_bin" --edition 2024 -O -o "$work/gen_unicode" "$root/tools/gen_unicode_semantics.rs"
"$work/gen_unicode" "$work/data"
if [[ "$mode" == --check ]]; then
    cmp "$work/data/unicode_data.rs" "$root/src/unicode_data.rs"
    for file in unicode_table.h escape_table.h case_tables.h; do
        cmp "$work/data/$file" "$root/src/runtime/$file"
    done
    echo "pinned compiler and runtime Unicode tables match exactly"
    exit
fi
mv "$work/data/unicode_data.rs" "$root/src/unicode_data.rs"
for file in unicode_table.h escape_table.h case_tables.h; do
    mv "$work/data/$file" "$root/src/runtime/$file"
done
echo "wrote pinned compiler and runtime Unicode tables"
