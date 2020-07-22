#!/bin/bash
set -e

ranges=$(paste -d- <(echo;git tag --sort=v:refname) <(git tag --sort=v:refname) | sed '1d;$d; s/-/../'|tac)

cat <<HERE
# Change log

This is the log of all commits by each release. Earlier commit history is a
little untidy, but it should be cleaner for newer releases.

HERE

for range in $ranges
do
  tag=$(echo $range| sed 's/.*\.\.//')
  c_date=$(git log -1 --format=%cd --date=short $tag)

  echo \#\# Version ${tag} released on ${c_date}
  git log --pretty=' - %s ' $range | egrep -v "docs:|ci:|rel:|chore:|Merge\ branch|wip:"
  echo
done
