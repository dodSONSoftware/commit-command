#!/bin/sh
for argument do
  if [ "$argument" = rev-list ] && [ -f /private/tmp/gitfleet-m3-native-35cm8or4/slow-read.enabled ]; then
    printf '%s\n' "$$" > /private/tmp/gitfleet-m3-native-35cm8or4/read-started.pid
    /bin/sleep 60
    break
  fi
done
exec /opt/homebrew/Cellar/git/2.56.0_1/bin/git "$@"
