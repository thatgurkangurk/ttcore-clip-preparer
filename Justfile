set default-list := true

version := `cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version'`
tag := "v" + version

release dry="false":
    #!/usr/bin/env bash
    set -euo pipefail

    if [ "{{ dry }}" = "true" ] || [ "{{ dry }}" = "--dry" ] || [ "{{ dry }}" = "1" ]; then
        echo "[DRY RUN]"
    fi

    tag_exists=$(git ls-remote --exit-code --tags origin "refs/tags/{{ tag }}" >/dev/null 2>&1 && echo "true" || echo "false")
    if [ "$tag_exists" = "true" ]; then
        echo "error: release version {{ tag }} already exists on remote origin." >&2
        exit 1
    fi

    if [ "{{ dry }}" = "true" ] || [ "{{ dry }}" = "--dry" ] || [ "{{ dry }}" = "1" ]; then
        echo "[DRY RUN] would release version {{ tag }} via 'gh workflow run release.yaml'"
    else
        echo "releasing version {{ tag }}..."
        gh workflow run release.yaml
    fi
