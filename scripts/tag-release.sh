#!/usr/bin/env bash
# Tags the current commit with the local date and time, as YYYYMMDD-HHMM, with no tag message.
# The tag is only created locally; the command to push it is printed at the end.
set -euo pipefail

# A tag names a commit, so uncommitted changes would not be part of it.
if [ -n "$(git status --porcelain --untracked-files=no)" ]; then
    echo "There are uncommitted changes, which the tag would not include. Commit them first." >&2
    exit 1
fi

tag="$(date +%Y%m%d-%H%M)"
git tag "$tag"

echo "Created tag $tag on $(git rev-parse --short HEAD)."
echo "To publish it: git push origin $tag"
