#!/usr/bin/env python3
"""Require the sanitizer compiler to detect real address, UB, and leak faults.

SAN_CC names one executable, as in the native differential and crate harnesses.
The default is gcc. Apple Clang can compile ASan while its runtime rejects
detect_leaks=1; selecting a compiler is qualified by execution, not its name.
"""

import os
from pathlib import Path
import subprocess
import tempfile


SOURCE = r"""
#include <limits.h>
#include <stdlib.h>
static void *volatile leaked;
int main(int argc, char **argv) {
    if (argc != 2) return 2;
    if (argv[1][0] == 'l') {
        leaked = malloc(73);
        if (!leaked) return 3;
        *(volatile char *)leaked = 1;
        leaked = 0;
        return 0;
    }
    if (argv[1][0] == 'u') {
        volatile int value = INT_MAX;
        return value + argc;
    }
    volatile char *p = malloc(16);
    if (!p) return 3;
    p[argv[1][0] == 'a' ? 16 : 0] = 1;
    free((void *)p);
    return 0;
}
"""


def main():
    compiler = os.environ.get("SAN_CC", "gcc")
    subprocess.run([compiler, "--version"], check=True, timeout=10)
    with tempfile.TemporaryDirectory(prefix="lustr-sanitizers-") as directory:
        source = Path(directory) / "probe.c"
        binary = Path(directory) / "probe"
        source.write_text(SOURCE)
        subprocess.run(
            [compiler, "-O1", "-g", "-Werror=incompatible-pointer-types",
             "-fno-sanitize-recover=undefined", "-fsanitize=address,undefined",
             str(source), "-o", str(binary)],
            check=True, timeout=30,
        )
        env = dict(os.environ, LC_ALL="C", ASAN_OPTIONS="detect_leaks=1:allocator_may_return_null=1")
        # Do not let inherited suppressions make a broken compiler look usable.
        env.pop("LSAN_OPTIONS", None)
        env.pop("UBSAN_OPTIONS", None)
        for mode, diagnostic in [
            ("clean", None),
            ("address", "AddressSanitizer: heap-buffer-overflow"),
            ("undefined", "runtime error: signed integer overflow"),
            ("leak", "LeakSanitizer: detected memory leaks"),
        ]:
            result = subprocess.run(
                [str(binary), mode], env=env, capture_output=True, text=True, timeout=15,
            )
            if diagnostic is None:
                passed = result.returncode == 0 and "Sanitizer" not in result.stderr
            else:
                passed = result.returncode != 0 and diagnostic in result.stderr
            if not passed:
                raise AssertionError(
                    f"SAN_CC={compiler!r} failed the {mode} sanitizer capability: "
                    f"exit={result.returncode}\n{result.stderr}"
                )
            print(f"ok native sanitizer capability: {mode}", flush=True)
    print("native sanitizer capabilities: 4 passed")


if __name__ == "__main__":
    main()
