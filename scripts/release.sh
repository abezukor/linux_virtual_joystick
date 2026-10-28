#!/usr/bin/env bash
# Prepare and push a release; GitHub Actions builds and publishes the AppImage.
set -euo pipefail
export LC_ALL=C

release_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd -- "$release_root"

die() {
    printf 'Release stopped: %s\n' "$*" >&2
    exit 1
}

valid_version() {
    local number='(0|[1-9][0-9]*)' identifier='[0-9A-Za-z-]+'
    local prerelease item
    local -a identifiers
    [[ $1 =~ ^$number\.$number\.$number(-$identifier(\.$identifier)*)?(\+$identifier(\.$identifier)*)?$ ]] || return 1
    prerelease=${1%%+*}
    if [[ $prerelease == *-* ]]; then
        IFS=. read -r -a identifiers <<< "${prerelease#*-}"
        for item in "${identifiers[@]}"; do
            if [[ $item =~ ^[0-9]+$ && ${#item} -gt 1 && $item == 0* ]]; then
                return 1
            fi
        done
    fi
}

# Compare decimal strings without integer overflow or octal interpretation.
number_greater() {
    [[ ${#1} -gt ${#2} || ( ${#1} -eq ${#2} && $1 > $2 ) ]]
}

version_greater() {
    local left=${1%%+*} right=${2%%+*} index left_id right_id
    local -a left_core right_core left_pre right_pre
    IFS=. read -r -a left_core <<< "${left%%-*}"
    IFS=. read -r -a right_core <<< "${right%%-*}"
    for index in 0 1 2; do
        if [[ ${left_core[index]} != "${right_core[index]}" ]]; then
            number_greater "${left_core[index]}" "${right_core[index]}"
            return
        fi
    done
    # A stable release sorts after its prereleases; build metadata is ignored.
    if [[ $left != *-* ]]; then
        [[ $right == *-* ]]
        return
    elif [[ $right != *-* ]]; then
        return 1
    fi
    IFS=. read -r -a left_pre <<< "${left#*-}"
    IFS=. read -r -a right_pre <<< "${right#*-}"
    for ((index = 0; index < ${#left_pre[@]} && index < ${#right_pre[@]}; index++)); do
        left_id=${left_pre[index]}
        right_id=${right_pre[index]}
        [[ $left_id != "$right_id" ]] || continue
        if [[ $left_id =~ ^[0-9]+$ && $right_id =~ ^[0-9]+$ ]]; then
            number_greater "$left_id" "$right_id"
        elif [[ $left_id =~ ^[0-9]+$ ]]; then
            return 1
        elif [[ $right_id =~ ^[0-9]+$ ]]; then
            return 0
        else
            [[ $left_id > $right_id ]]
        fi
        return
    done
    [[ ${#left_pre[@]} -gt ${#right_pre[@]} ]]
}

# Read or replace only this app's version, preserving dependency versions and
# the rest of the manifest/lockfile formatting. An empty third argument reads.
package_version() {
    awk -v mode="$2" -v requested="${3:-}" '
        function quoted(line) {
            sub(/^[^"]*"/, "", line)
            sub(/".*$/, "", line)
            return line
        }
        /^\[/ {
            in_package = mode == "manifest" ? /^\[package\][[:space:]]*(#.*)?$/ : /^\[\[package\]\][[:space:]]*$/
            name = ""
        }
        in_package && /^name[[:space:]]*=/ { name = quoted($0) }
        in_package && /^version[[:space:]]*=[[:space:]]*"/ &&
            (mode == "manifest" || name == "linux_virtual_joystick") {
            found++
            if (requested == "") { print quoted($0); exit }
            sub(/"[^"]*"/, "\"" requested "\"")
        }
        requested != "" { print }
        END { if (found != 1) exit 1 }
    ' "$1"
}

[[ -z $(git status --porcelain) ]] || die 'Commit or stash your changes before making a release.'
branch=$(git symbolic-ref --quiet --short HEAD) || die 'Check out a branch before making a release (HEAD is detached).'
remote=$(git config --get "branch.$branch.remote" || true)
remote=${remote:-origin}
[[ $remote != . ]] || die 'This branch tracks a local branch; configure a GitHub remote first.'
target=$(git config --get "branch.$branch.merge" || true)
target=${target:-refs/heads/$branch}
[[ $target == refs/heads/* ]] || die 'The upstream must be a remote branch.'

current=$(package_version Cargo.toml manifest) || die 'Cannot find the app version in Cargo.toml.'
locked=$(package_version Cargo.lock lock) || die 'Cannot find the app version in Cargo.lock.'
[[ $current == "$locked" ]] || die 'Cargo.toml and Cargo.lock have different app versions; fix them first.'
valid_version "$current" || die 'The current app version is not a semantic version.'
printf 'Current app version: %s\nFetching release tags from %s...\n' "$current" "$remote"
git fetch --tags "$remote"
if [[ -n $(git ls-remote --heads "$remote" "$target") ]]; then
    git fetch "$remote" "$target"
    git merge-base --is-ancestor FETCH_HEAD HEAD || die 'Your branch is behind or diverges from its upstream. Pull/merge it first.'
fi

latest_tag=''
latest_version=''
tags=$(git tag --list)
while IFS= read -r tag; do
    candidate=${tag#v}
    if valid_version "$candidate"; then
        if [[ -z $latest_version ]] || version_greater "$candidate" "$latest_version"; then
            latest_tag=$tag
            latest_version=$candidate
        fi
    fi
done <<< "$tags"
printf 'Latest release tag: %s\n' "${latest_tag:-(none)}"
printf 'New release version (without a v prefix): '
read -r new_version || die 'Release cancelled.'
valid_version "$new_version" || die 'Use a semantic version such as 2.0.1 or 2.1.0-rc.1, without leading zeros.'
version_greater "$new_version" "$current" || die 'The new version must be newer than the app version and all release tags.'
if [[ -n $latest_version ]] && ! version_greater "$new_version" "$latest_version"; then
    die 'The new version must be newer than the app version and all release tags.'
fi
if git show-ref --verify --quiet "refs/tags/$new_version"; then
    die "Tag $new_version already exists."
fi

original_head=$(git rev-parse HEAD)
release_tmpdir=$(mktemp -d)
cp Cargo.toml Cargo.lock "$release_tmpdir/"
push_args=(push --atomic "$remote" "HEAD:$target" "refs/tags/$new_version:refs/tags/$new_version")

cleanup() {
    local status=$?
    trap - EXIT
    if ((status != 0)); then
        if [[ $(git rev-parse HEAD) == "$original_head" ]]; then
            cp "$release_tmpdir/Cargo.toml" "$release_tmpdir/Cargo.lock" .
            git restore --staged -- Cargo.toml Cargo.lock
        else
            printf 'The release commit is preserved locally; no rollback was attempted.\n' >&2
            printf 'Inspect it and the tag before retrying: ' >&2
            printf '%q ' git "${push_args[@]}" >&2
            printf '\n' >&2
        fi
    fi
    rm -rf -- "$release_tmpdir"
    exit "$status"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

package_version Cargo.toml manifest "$new_version" > "$release_tmpdir/manifest.new"
package_version Cargo.lock lock "$new_version" > "$release_tmpdir/lock.new"
cat "$release_tmpdir/manifest.new" > Cargo.toml
cat "$release_tmpdir/lock.new" > Cargo.lock
git add -- Cargo.toml Cargo.lock
git commit -m "Release $new_version" -- Cargo.toml Cargo.lock
git tag -a "$new_version" -m "Release $new_version"
printf 'Pushing release %s to %s...\n' "$new_version" "$remote"
git "${push_args[@]}"
printf 'Release %s pushed. GitHub Actions will build and publish the AppImage.\n' "$new_version"
