#!/usr/bin/env bash
# Resolve host SSH keys into the container's ~/.ssh.
#
# devcontainer.json bind-mounts two candidate host directories (read-only):
#   /root/.ssh-host-win  <- %USERPROFILE%\.ssh   (Windows host)
#   /root/.ssh-host-mac  <- $HOME/.ssh           (macOS / Linux host)
#
# On any given host only one of HOME / USERPROFILE is normally set, so only one
# candidate has real keys; the other resolves to an empty placeholder mount.
#
# Preference: if BOTH candidates contain keys (e.g. Git Bash on Windows exports
# HOME as well as USERPROFILE), USERPROFILE wins.
# If NEITHER candidate contains anything, skip silently (mount nothing).
set -euo pipefail

has_files() { [ -d "$1" ] && [ -n "$(ls -A "$1" 2>/dev/null)" ]; }

src=""
if has_files /root/.ssh-host-win; then
    src=/root/.ssh-host-win          # %USERPROFILE%\.ssh  (preferred)
elif has_files /root/.ssh-host-mac; then
    src=/root/.ssh-host-mac          # $HOME/.ssh
fi

if [ -z "$src" ]; then
    echo "setup-ssh: no host SSH keys found (checked %USERPROFILE%\\.ssh and \$HOME/.ssh); skipping."
    exit 0
fi

echo "setup-ssh: using host SSH keys from $src"
mkdir -p ~/.ssh
cp -r "$src"/. ~/.ssh/
chmod 700 ~/.ssh
chmod 600 ~/.ssh/id_* 2>/dev/null || true
