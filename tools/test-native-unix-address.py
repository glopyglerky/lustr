#!/usr/bin/env python3
"""Exercise the actual Unix-address C helpers under ASan/UBSan.

The native model uses the host's sockaddr_un. The Darwin model supplies Apple's
documented 104-byte layout but does not simulate syscalls or claim native macOS
execution. The compiler's actual helper bodies are read from sysnet.c.
"""

import argparse
import hashlib
import os
from pathlib import Path
import subprocess
import tempfile


def function(source, prefix):
    start = source.index(prefix)
    end = source.index("\n}", start) + 2
    return source[start:end]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, default=Path(__file__).resolve().parents[1] / "compiler/native/src/runtime/sysnet.c")
    parser.add_argument("--model", choices=["native", "darwin-layout", "all"], default="all")
    args = parser.parse_args()
    raw = args.source.read_bytes()
    source = raw.decode()
    helpers = "\n\n".join([
        function(source, "static socklen_t lust_sysnet_unix_addr("),
        function(source, "static void lust_sysnet_push_unix_addr("),
    ])
    common = r"""
#include <assert.h>
#include <errno.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/un.h>

#ifdef LUSTR_TEST_DARWIN
#ifndef __APPLE__
#define __APPLE__ 1
#endif
struct lustr_test_darwin_sockaddr_un {
    unsigned char sun_len;
    unsigned char sun_family;
    char sun_path[104];
};
#define sockaddr_un lustr_test_darwin_sockaddr_un
#endif

typedef struct { uint64_t kind; unsigned char bytes[256]; uint64_t len; } lust_Vec;
static void lust_sysnet_put64(lust_Vec *out, uint64_t kind) { out->kind = kind; }
static void lust_sysnet_push(lust_Vec *out, const void *p, uint64_t n) {
    assert(n <= sizeof out->bytes);
    memcpy(out->bytes, p, (size_t)n);
    out->len = n;
}
"""
    checks = r"""
int main(void) {
    struct sockaddr_un addr;
    const size_t capacity = sizeof addr.sun_path;
    const size_t offset = offsetof(struct sockaddr_un, sun_path);
    unsigned char input[256];
    memset(input, 'x', sizeof input);

    /* Invalid lengths must fail before any read or copy, for both kinds.
       The old Darwin implementation overflows for capacity + 3 here. */
    const uint64_t invalid[] = {capacity + 3, capacity, capacity + 1, UINT64_MAX};
    for (int abstract = 0; abstract <= 1; abstract++) {
        for (size_t i = 0; i < sizeof invalid / sizeof invalid[0]; i++) {
            memset(&addr, 0x5a, sizeof addr);
            struct sockaddr_un original = addr;
            errno = 0;
            assert(lust_sysnet_unix_addr(input, invalid[i], abstract, &addr) == 0);
            assert(errno == EINVAL);
            assert(memcmp(&addr, &original, sizeof addr) == 0);
        }
        socklen_t n = lust_sysnet_unix_addr(input, capacity - 1, abstract, &addr);
        assert(n == sizeof addr);
        assert(addr.sun_family == AF_UNIX);
        if (abstract) {
            assert(addr.sun_path[0] == 0);
            assert(memcmp(addr.sun_path + 1, input, capacity - 1) == 0);
        } else {
            assert(memcmp(addr.sun_path, input, capacity - 1) == 0);
            assert(addr.sun_path[capacity - 1] == 0);
        }
#ifdef __APPLE__
        assert(addr.sun_len == sizeof addr);
#endif
    }
    assert(lust_sysnet_unix_addr(NULL, 0, 0, &addr) == offset);
    assert(lust_sysnet_unix_addr(NULL, 0, 1, &addr) == offset + 1);

    lust_Vec out = {0};
    memset(&addr, 0, sizeof addr);
    lust_sysnet_push_unix_addr(&out, &addr, (socklen_t)offset);
    assert(out.kind == 0 && out.len == 0);
#ifdef __APPLE__
    /* Darwin may return 16 bytes for a padded unnamed socket address. */
    lust_sysnet_push_unix_addr(&out, &addr, 16);
    assert(out.kind == 0 && out.len == 0);
#else
    lust_sysnet_push_unix_addr(&out, &addr, (socklen_t)(offset + 1));
    assert(out.kind == 2 && out.len == 0);
    addr.sun_path[1] = 'a';
    lust_sysnet_push_unix_addr(&out, &addr, (socklen_t)(offset + 2));
    assert(out.kind == 2 && out.len == 1 && out.bytes[0] == 'a');
#endif
    memset(&addr, 'x', sizeof addr);
    out.len = 0;
    lust_sysnet_push_unix_addr(&out, &addr, (socklen_t)(sizeof addr + 10));
    assert(out.kind == 1 && out.len == capacity);
    for (size_t i = 0; i < capacity; i++) assert(out.bytes[i] == 'x');
    printf("Unix address helpers passed: path capacity=%zu\n", capacity);
    return 0;
}
"""
    with tempfile.TemporaryDirectory(prefix="lustr-unix-address-") as temp:
        directory = Path(temp)
        c_file = directory / "address.c"
        c_file.write_text(common + "\n" + helpers + "\n" + checks)
        for model in (["native", "darwin-layout"] if args.model == "all" else [args.model]):
            exe = directory / model
            command = [os.environ.get("CC", "cc"), "-std=c11", "-O1", "-g", "-Wall", "-Wextra", "-Werror", "-fsanitize=address,undefined", "-fno-omit-frame-pointer", "-fno-sanitize-recover=all"]
            if model == "darwin-layout":
                command.append("-DLUSTR_TEST_DARWIN=1")
            subprocess.run(command + [str(c_file), "-o", str(exe)], check=True)
            subprocess.run([str(exe)], check=True)
    print("sysnet.c SHA-256:", hashlib.sha256(raw).hexdigest())


if __name__ == "__main__":
    main()
