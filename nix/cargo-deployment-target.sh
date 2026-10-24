# The built-in catalog's `cargo-deployment-target` check (src/catalog T13,
# paradigm T18 and V28): a cargo workspace at the repository root pins
# paradigm V28's macOS floor in its own `.cargo/config.toml`,
#
#   [env]
#   MACOSX_DEPLOYMENT_TARGET = { value = "<floor>", force = true }
#
# (or the same value as an `[env.MACOSX_DEPLOYMENT_TARGET]` table), so
# cargo builds for that floor everywhere: the dev shell, CI, nix's
# buildRustPackage and plain cargo. `force` is required: nixpkgs' darwin
# stdenv exports its own MACOSX_DEPLOYMENT_TARGET in the dev shell and in
# buildRustPackage, and cargo's [env] does not override a variable that is
# already set without it. taplo parses the file and jq reads its JSON, so
# no TOML text is matched. CARGO_DEPLOYMENT_TARGET is the floor; nix sets
# it from src/seed/cargo-config.toml. Without a Cargo.toml at the
# repository root it passes with a notice.

floor=${CARGO_DEPLOYMENT_TARGET:?is unset; nix sets it from src/seed/cargo-config.toml}
config=.cargo/config.toml
want="[env] MACOSX_DEPLOYMENT_TARGET = { value = \"$floor\", force = true }"

say() {
	printf 'cargo-deployment-target: %s\n' "$1"
}

refuse() {
	say "$1" >&2
	exit 1
}

[ "$#" -eq 0 ] || refuse 'usage: cargo-deployment-target (run at the repository root)'

if [ ! -f Cargo.toml ]; then
	say 'skipped: no Cargo.toml at the repository root, so no cargo workspace to pin'
	exit 0
fi
[ -f "$config" ] || refuse "$config is missing; it must hold $want"
json=$(taplo get -f "$config" -o json) || refuse "$config does not parse as TOML; it must hold $want"

entry=$(jq -c '.env.MACOSX_DEPLOYMENT_TARGET // null' <<<"$json")
[ "$entry" != null ] || refuse "$config sets no [env] MACOSX_DEPLOYMENT_TARGET; it must hold $want"
value=$(jq -r 'if type == "string" then . elif type == "object" and (.value | type) == "string" then .value else "" end' <<<"$entry")
[ "$value" = "$floor" ] || refuse "$config sets MACOSX_DEPLOYMENT_TARGET to ${value:-no string}, expected $floor (paradigm V28); it must hold $want"
force=$(jq -r 'type == "object" and .force == true' <<<"$entry")
[ "$force" = true ] || refuse "$config sets MACOSX_DEPLOYMENT_TARGET without force = true, so a value the environment already holds (nix's darwin stdenv exports one) wins; it must hold $want"
