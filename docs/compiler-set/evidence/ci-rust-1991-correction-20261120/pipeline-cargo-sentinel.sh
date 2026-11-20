#!/bin/sh
if [ "$1" = "+1.99.1" ] && [ "$2" = "test" ]; then
  printf "%s\n" "controlled Cargo failure for log retention" >&2
  exit 101
fi
exec <RUSTUP_PROXY_BIN>/cargo "$@"
