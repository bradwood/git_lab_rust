#!/bin/bash
set -ex

baseurl="https://gitlab.com/api/v4/projects/${CI_PROJECT_ID}"
fullname="${TARBALL}"
linkname="${TARBALL}"
linkurl="${baseurl}/jobs/${CI_JOB_ID}/artifacts/${fullname}"
linklist="[{\"name\": \"${linkname}\", \"url\": \"${linkurl}\"}]"

descr="$(curl -H \"PRIVATE-TOKEN:\ ${PRIVATE_TOKEN}\" ${baseurl}/repository/tags/${CI_COMMIT_TAG}|jq -r '.message')"

DATA="
{
  \"name\": \"${progname} version ${CI_COMMIT_TAG}\",
  \"description\": \"${descr}\",
  \"tag_name\": \"${CI_COMMIT_TAG}\",
  \"assets\": {
    \"links\": "${linklist}"
  }
}
"
curl -H 'Content-Type: application/json' -X POST -H "PRIVATE-TOKEN: ${PRIVATE_TOKEN}" "${baseurl}/releases" -d "${DATA}"

