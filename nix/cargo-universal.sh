# The built-in catalog's `cargo-universal` check (src/catalog T11, paradigm
# V28): each bin target of a cargo workspace, built in release for
# aarch64-apple-darwin and x86_64-apple-darwin and joined by `lipo`, must
# hold exactly the arm64 and x86_64 slices, each with `minos` at the floor
# (14.0 today), so one binary runs on both Macs from that macOS on. Modelled on age-plugin-se's
# gate/universal.sh, which does the same for SwiftPM.
#
#   cargo-universal          build the workspace's bins and check them
#   cargo-universal BINARY   check BINARY only, building nothing
#
# CARGO_UNIVERSAL_MINOS is the floor, which nix sets from
# src/seed/cargo-config.toml; the build asks for it through
# MACOSX_DEPLOYMENT_TARGET. Only the slices' minos is judged, so a floor
# raised anywhere (a forced `.cargo/config.toml` `[env]`) fails. The universal binaries land in
# `<target dir>/universal-apple-darwin/release/`. nix sets
# CARGO_UNIVERSAL_SYSROOT on macOS: the host rustc's sysroot plus the other
# Mac's std, which nixpkgs' rustc lacks.

minos=${CARGO_UNIVERSAL_MINOS:?is unset; nix sets it from src/seed/cargo-config.toml}
archs="arm64 x86_64"

say() {
	printf 'cargo-universal: %s\n' "$1"
}

refuse() {
	say "$1" >&2
	exit 1
}

[ "$#" -le 1 ] || refuse 'usage: cargo-universal [BINARY]'

host=$(uname -s)
if [ "$host" != Darwin ]; then
	say "skipped: lipo joins Mac slices on macOS only, and this host is $host"
	exit 0
fi

# The first `minos` of the slice's LC_BUILD_VERSION load command.
slice_minos() {
	otool -arch "$1" -l "$2" | awk '$2 == "LC_BUILD_VERSION" { found = 1 } found && $1 == "minos" { print $2; exit }'
}

universal() {
	local binary=$1 found got arch
	[ -f "$binary" ] || refuse "no binary at $binary"
	found=$(lipo -archs "$binary" | tr ' ' '\n' | sed '/^$/d' | sort | tr '\n' ' ')
	[ "$found" = "$archs " ] || refuse "$binary holds [${found% }], expected [$archs]"
	for arch in $archs; do
		got=$(slice_minos "$arch" "$binary")
		[ "$got" = "$minos" ] || refuse "$binary $arch slice has minos ${got:-none}, expected $minos (paradigm V28)"
	done
	say "$binary holds $archs, each minos $minos"
}

if [ "$#" -eq 1 ]; then
	universal "$1"
	exit 0
fi

metadata=$(cargo metadata --no-deps --format-version 1)
bins=$(jq -r '.packages[].targets[] | select(.kind | index("bin")) | .name' <<<"$metadata" | sort -u)
if [ -z "$bins" ]; then
	say 'skipped: the workspace has no bin target, and a library ships no binary'
	exit 0
fi
target=$(jq -r .target_directory <<<"$metadata")

export MACOSX_DEPLOYMENT_TARGET=$minos
triples="aarch64-apple-darwin x86_64-apple-darwin"
for triple in $triples; do
	if [ -n "${CARGO_UNIVERSAL_SYSROOT:-}" ]; then
		var=CARGO_TARGET_$(printf '%s' "$triple" | tr 'a-z-' 'A-Z_')_RUSTFLAGS
		export "$var=--sysroot $CARGO_UNIVERSAL_SYSROOT"
	fi
	cargo build -q --release --workspace --bins --target "$triple"
done

out=$target/universal-apple-darwin/release
mkdir -p "$out"
for bin in $bins; do
	slices=()
	for triple in $triples; do slices+=("$target/$triple/release/$bin"); done
	lipo -create "${slices[@]}" -output "$out/$bin"
	universal "$out/$bin"
done
