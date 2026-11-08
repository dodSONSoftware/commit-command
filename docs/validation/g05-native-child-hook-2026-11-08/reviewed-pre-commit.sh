#!/bin/sh
set -eu
fixture='/private/tmp/gitfleet-m3-native-g05-child-29iydypi'
printf 'run\n' >> "$fixture/hook-runs"
if [ -f "$fixture/reject-child" ]; then
  printf 'Fixture pre-commit rejection\n' >&2
  exit 23
fi
: > "$fixture/child-hook-running"
while [ ! -f "$fixture/release-child" ]; do /bin/sleep 0.1; done
