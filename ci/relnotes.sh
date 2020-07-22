#!/bin/bash
set -e

tag=$1
range=$(paste -d- <(echo;git tag --sort=v:refname) <(git tag --sort=v:refname) | sed '1d;$d; s/-/../'| grep ${tag}$)
c_date=$(git log -1 --format=%cd --date=short $tag)

echo \#\# Version ${tag} released on ${c_date}
git log --pretty=' - %s ' $range | egrep -v "docs:|ci:|rel:|chore:|Merge\ branch|wip:"
