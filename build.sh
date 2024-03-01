#!/bin/sh
repo_dir="$(dirname "$0")"
script_name="$(basename "$0")"
staging_dir="${repo_dir}/staging"
target_dir="${repo_dir}/target/release"

# Echo to stderr then exit with error code 1
panic() {
  >&2 echo "${script_name}: $*"
  exit 1
}

# This won't go too well when run on Windows. Remember to set the PREFIX.
prefix=${PREFIX:-/usr/local}

# Make sure that we are indeed in the repository
cd "$repo_dir" || panic "Failed to change directory to $?. Exiting"

# Get the package name through `cargo pkgid` if different else just use the basename
# Should print out the former when there's a different name, or else, the latter:
# path+file://${repo_dir}#${pkgname}@0.1.1
# path+file://$(dirname "$repo_dir")/${pkgname}#0.1.1
pkgid=$(cargo pkgid) || panic "Couldn't get the package ID. Rust not installed?"
# Try to trim the version after '@' if the output is like the former
pkgname=${pkgid%@*}
if [ "$pkgid" != "$pkgname" ]; then
  # Uses a different package name because there is an '@' that was trimmed off, try to get that
  # Trim everything before the '#'
  pkgname=${pkgname#*#}
else
  # Same package name as the directory because unchanged, get the basename instead
  # Trim the parent directory
  pkgname=${pkgname##*/}
  # Trim the version after the '#'
  pkgname=${pkgname%#*}
fi

# Create the staging directory
rm -rf "$staging_dir"
mkdir -p "$staging_dir"

# Build then move the executable
cargo build --release || panic "Cannot be built!"
mkdir -p "${staging_dir}${prefix}/bin"
mv "${target_dir}/${pkgname}" "${staging_dir}/${prefix}/bin/${pkgname}" || panic "The binary cannot be moved!"

# All done
exit 0
