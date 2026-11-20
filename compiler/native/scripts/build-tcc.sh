#!/usr/bin/env bash
# Builds the pinned tinycc (mob) into third_party/tcc. The lust build links its libtcc.a.
set -euo pipefail
TCC_COMMIT=43c7708b85681a2fd4451c8a541af4494a8919b2
root=$(cd "$(dirname "$0")/.." && pwd)
src=$root/third_party/tinycc
out=$root/third_party/tcc
if [ ! -d "$src/.git" ]; then
  git clone -q https://github.com/TinyCC/tinycc.git "$src"
fi
git -C "$src" fetch -q --depth 1 origin "$TCC_COMMIT" 2>/dev/null || true
git -C "$src" checkout -q "$TCC_COMMIT"
git -C "$src" diff --exit-code -- . ':!tccmacho.c'
# The pinned Mach-O signer interpolates paths into a shell command. Apply only
# the separately declared, exact-source signing repair; do not change the pin.
python3 "$root/../../tools/apply-tinycc-signing.py" "$src" --receipt "$src/.lustr-macho-signing.json"
cd "$src"
./configure --prefix="$out" >/dev/null
make -j"$(getconf _NPROCESSORS_ONLN)" >/dev/null
make install >/dev/null
mkdir -p "$out/share/licenses/tinycc"
cp COPYING "$out/share/licenses/tinycc/COPYING"
cp "$src/.lustr-macho-signing.json" "$out/share/licenses/tinycc/lustr-macho-signing.json"
printf '%s\n' 'https://github.com/TinyCC/tinycc' "revision $TCC_COMMIT" \
  'Declared Lustr Mach-O signing integration: lustr-macho-signing.json' > "$out/share/licenses/tinycc/SOURCE.txt"
echo "tcc $TCC_COMMIT installed in $out"
