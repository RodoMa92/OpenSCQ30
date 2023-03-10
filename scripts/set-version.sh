#!/usr/bin/env bash
set -e
set -o pipefail

set_version_in_cargo_toml() {
    # --null-data treats the entire file as one line. this mkes it easy to only replace the first occurrance of
    # "version =" to avoid catching any dependency versions
    sed --in-place --regexp-extended --null-data "s/(\nversion *= *\")([0-9\.]+)(\"\n)/\1$2\3/" "$1"
}

set_version_in_build_gradle() {
    # --null-data treats the entire file as one line. this mkes it easy to only replace the first occurrance of
    # "version =" to avoid catching any dependency versions
    sed --in-place --regexp-extended --null-data "s/(\n *versionName \")([0-9\.]+)(\"\n)/\1$2\3/" "$1"
}

if [ -z "$1" ]; then
    cat <<EOF
Usage: set-version.sh 1.0.0
EOF
    exit 1
fi

if [[ ! "$1" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo Version number format must be x.y.z
    exit 2
fi

set_version_in_cargo_toml android/Cargo.toml "$1"
set_version_in_cargo_toml cli/Cargo.toml "$1"
set_version_in_cargo_toml gui/Cargo.toml "$1"
set_version_in_cargo_toml lib/Cargo.toml "$1"
set_version_in_build_gradle android/app/build.gradle "$1"
