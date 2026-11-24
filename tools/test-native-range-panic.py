#!/usr/bin/env python3
"""Check the real runtime range helper's flag-unwind return boundary.

The host harness supplies a panic that returns, as native flag-mode unwinding
does. It exercises the checked-in helper, including overlapping invalid bounds,
and requires one panic, no output writes after failure, and exact valid bounds.
It does not replace generated-program sanitizer or exception-table checks.
"""

import argparse
import hashlib
import json
import pathlib
import subprocess
import tempfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=pathlib.Path)
    parser.add_argument("--cc", default="cc")
    parser.add_argument("--output", type=pathlib.Path)
    args = parser.parse_args()
    source = args.source or pathlib.Path(__file__).resolve().parents[1] / "compiler/native/src/runtime/core.c"
    raw = source.read_bytes()
    text = raw.decode()
    start = text.index("static void lust_range_bounds_check(")
    stop = text.index("\n/* `Vec::drain`:", start)
    helper = text[start:stop]
    harness = r'''
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#define LUST_BODY(...) __VA_ARGS__
typedef uint64_t lust_loc;
typedef struct { char bytes[160]; unsigned len; } lust_String;
static unsigned panics;
static lust_String lust_string_new(void) { return (lust_String){0}; }
static void lust_string_push_cstr(lust_String *s, const char *p) {
    unsigned n = (unsigned)strlen(p);
    memcpy(s->bytes + s->len, p, n + 1);
    s->len += n;
}
static void lust_fmt_u64(lust_String *s, uint64_t v) {
    s->len += (unsigned)snprintf(s->bytes + s->len, sizeof s->bytes - s->len, "%llu", (unsigned long long)v);
}
static void lust_panic(lust_String *s, lust_loc loc) {
    (void)s; (void)loc;
    ++panics;
}
'''
    harness += helper
    harness += r'''
int main(void) {
    unsigned seen = 0, invalid = 0, failures = 0;
    for (uint64_t len = 0; len <= 4; ++len)
    for (uint64_t st = 0; st < 3; ++st)
    for (uint64_t sv = 0; sv <= 6; ++sv)
    for (uint64_t et = 0; et < 3; ++et)
    for (uint64_t ev = 0; ev <= 6; ++ev) {
        /* Convert the public Included/Excluded/Unbounded definitions into
           half-open bounds independently of the runtime's check ordering. */
        uint64_t start = st == 2 ? 0 : sv + (st == 1);
        uint64_t end = et == 2 ? len : ev + (et == 0);
        unsigned bad = start > end || end > len;
        uint64_t out_s = UINT64_MAX, out_e = UINT64_MAX;
        panics = 0;
        lust_range_bounds_check(len, st, sv, et, ev, &out_s, &out_e, 0);
        ++seen;
        invalid += bad;
        if (bad ? panics != 1 || out_s != UINT64_MAX || out_e != UINT64_MAX
                : panics != 0 || out_s != start || out_e != end) {
            if (failures < 8) fprintf(stderr, "range len=%llu start=(%llu,%llu) end=(%llu,%llu): panic calls=%u expected=%u; outputs=(%llu,%llu)\n",
                (unsigned long long)len, (unsigned long long)st, (unsigned long long)sv,
                (unsigned long long)et, (unsigned long long)ev, panics, bad,
                (unsigned long long)out_s, (unsigned long long)out_e);
            ++failures;
        }
    }
    printf("{\"cases\":%u,\"invalid\":%u,\"failures\":%u}\n", seen, invalid, failures);
    return failures != 0;
}
'''
    with tempfile.TemporaryDirectory(prefix="lustr-range-panic-") as temporary:
        directory = pathlib.Path(temporary)
        c_file = directory / "range.c"
        c_file.write_text(harness)
        binary = directory / "range"
        subprocess.run([args.cc, "-std=c11", "-O1", "-Wall", "-Wextra", "-Werror", str(c_file), "-o", str(binary)], check=True)
        run = subprocess.run([str(binary)], capture_output=True, text=True)
    result = json.loads(run.stdout)
    result.update({
        "schema": "lustr-range-panic-v1",
        "runtime_source_sha256": hashlib.sha256(raw).hexdigest(),
        "helper_sha256": hashlib.sha256(helper.encode()).hexdigest(),
        "harness_sha256": hashlib.sha256(harness.encode()).hexdigest(),
        "exit_code": run.returncode,
        "stderr": run.stderr,
        "limits": "Host execution of the actual range helper with a returning panic; generated-program and platform qualification are separate.",
    })
    rendered = json.dumps(result, indent=2) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered)
    print(rendered, end="")
    raise SystemExit(run.returncode)


if __name__ == "__main__":
    main()
