#!/usr/bin/env bash

set -euo pipefail

: "${RELEASE_TAG:?RELEASE_TAG is required}"
: "${GITHUB_EVENT_NAME:?GITHUB_EVENT_NAME is required}"
: "${GITHUB_SHA:?GITHUB_SHA is required}"
: "${GITHUB_REF_NAME:?GITHUB_REF_NAME is required}"
: "${DEFAULT_BRANCH:?DEFAULT_BRANCH is required}"
: "${GITHUB_REPOSITORY:?GITHUB_REPOSITORY is required}"

if [[ ! "${RELEASE_TAG}" =~ ^office2pdf-katana-v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "::error::Release tag must match office2pdf-katana-vMAJOR.MINOR.PATCH: ${RELEASE_TAG}"
  exit 1
fi

version="${RELEASE_TAG#office2pdf-katana-v}"
gh_cli_bin="${GH_CLI_BIN:-gh}"
tag_exists=false
release_commit="${GITHUB_SHA}"

if git rev-parse --verify --quiet "refs/tags/${RELEASE_TAG}^{commit}" >/dev/null; then
  tag_exists=true
  release_commit="$(git rev-list -n 1 "refs/tags/${RELEASE_TAG}")"
  git switch --detach "${RELEASE_TAG}"
elif [[ "${GITHUB_EVENT_NAME}" != "workflow_dispatch" ]]; then
  echo "::error::Release event tag ${RELEASE_TAG} is not available in the checkout"
  exit 1
elif [[ "${GITHUB_REF_NAME}" != "${DEFAULT_BRANCH}" ]]; then
  echo "::error::New releases must be dispatched from ${DEFAULT_BRANCH}, not ${GITHUB_REF_NAME}"
  exit 1
fi

metadata="$(cargo metadata --locked --no-deps --format-version 1)"
lib_version="$(jq -r '.packages[] | select(.name == "office2pdf-katana") | .version' <<<"${metadata}")"
cli_version="$(jq -r '.packages[] | select(.name == "office2pdf-cli") | .version' <<<"${metadata}")"
cli_dependency="$(jq -r '.packages[] | select(.name == "office2pdf-cli") | .dependencies[] | select(.name == "office2pdf-katana") | .req' <<<"${metadata}")"

if [[ "${lib_version}" != "${version}" || "${cli_version}" != "${version}" ]]; then
  echo "::error::Cargo package versions (${lib_version}, ${cli_version}) do not match ${RELEASE_TAG}"
  exit 1
fi

case "${cli_dependency}" in
  "${version}"|"^${version}") ;;
  *)
    echo "::error::office2pdf-cli depends on office2pdf ${cli_dependency}, expected ${version}"
    exit 1
    ;;
esac

if "${gh_cli_bin}" release view "${RELEASE_TAG}" --repo "${GITHUB_REPOSITORY}" >/dev/null 2>&1; then
  echo "Reusing existing GitHub Release ${RELEASE_TAG} at ${release_commit}"
  exit 0
fi

if [[ "${tag_exists}" == true ]]; then
  previous_tag="$(git describe --tags --abbrev=0 "${RELEASE_TAG}^" 2>/dev/null || true)"
else
  previous_tag="$(git describe --tags --abbrev=0 "${release_commit}" 2>/dev/null || true)"
fi

generate_args=(
  --method POST
  "repos/${GITHUB_REPOSITORY}/releases/generate-notes"
  -f "tag_name=${RELEASE_TAG}"
  -f "target_commitish=${release_commit}"
)
if [[ -n "${previous_tag}" ]]; then
  generate_args+=(-f "previous_tag_name=${previous_tag}")
fi

generated_notes="$("${gh_cli_bin}" api "${generate_args[@]}")"
release_name="$(jq -r '.name // empty' <<<"${generated_notes}")"
release_body="$(jq -r '.body // empty' <<<"${generated_notes}")"
if [[ -z "${release_name}" ]]; then
  release_name="${RELEASE_TAG}"
fi

notes_file="$(mktemp)"
trap 'rm -f "${notes_file}"' EXIT
printf '%s\n\n## Contributors\n' "${release_body}" >"${notes_file}"

if [[ -n "${previous_tag}" ]]; then
  contributors="$("${gh_cli_bin}" api "repos/${GITHUB_REPOSITORY}/compare/${previous_tag}...${release_commit}" \
    --jq '[.commits[].author | select(.login != null and (.login | endswith("[bot]") | not)) | {login, url: .html_url}] | unique_by(.login) | .[] | "- [@\(.login)](\(.url))"')"
else
  contributors=""
fi

if [[ -n "${contributors}" ]]; then
  printf '%s\n' "${contributors}" >>"${notes_file}"
else
  owner="${GITHUB_REPOSITORY%%/*}"
  printf '%s\n' "- [@${owner}](https://github.com/${owner})" >>"${notes_file}"
fi

create_args=(
  "${RELEASE_TAG}"
  --repo "${GITHUB_REPOSITORY}"
  --title "${release_name}"
  --notes-file "${notes_file}"
)
if [[ "${tag_exists}" == true ]]; then
  create_args+=(--verify-tag)
else
  create_args+=(--target "${release_commit}")
fi

"${gh_cli_bin}" release create "${create_args[@]}"
echo "Created GitHub Release ${RELEASE_TAG} at ${release_commit}"
