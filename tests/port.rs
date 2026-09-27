//! Root T54: set-and-setting's lefthook fragments, as
//! `assemble-lefthook.sh` merged them, against pklith's fragments. The
//! assembler's bats cases that describe a step set (one fragment's
//! commands, merged once, split by hook) become one golden here: every
//! legacy command lands in the same pklith fragment, renamed as docs/PORT.md
//! says, or PORT.md names it as dropped. Nothing leaves silently. The cases
//! about lefthook itself (`extends`, the migration overlay, `remotes`, YAML
//! markers) have no hk counterpart and are not ported.

use pklith::catalog::fragment::Fragment;

/// Each fragment's commands, pre-commit and pre-push merged, from
/// `setting/integrations/lefthook/*.yml` at set-and-setting 1377327.
const LEGACY: &str = "\
base changelog-touched gitleaks git-conflict-markers git-no-local-paths nix-flake-check linter-coverage commit-msg-lint
actions actionlint
ascii unicode-lint
awk gawk-lint
bats bats-parse bats-unit tdd-order-bats
brakeman brakeman
bundle-audit bundle-audit
just justfile-alphabetical justfile-no-embedded-shell
markdown markdownlint markdownlint-agentic
nix nixfmt statix deadnix nix-no-embedded-shell
reek reek
rspec rspec
rubocop rubocop
ruby
set rekall-check rekall-gnu-sed set-skill-extension set-skill-size set-ref-resolution set-bundle-content
shell shellcheck shfmt no-shell-functions
tcl tcl-syntax
toml taplo
xml xmllint
yaml yamllint
";

const PORT: &str = include_str!("../docs/PORT.md");

/// The rows of the Markdown table under `heading` (up to the next blank
/// line after it starts), cells trimmed, header and rule skipped.
fn table(after: &str) -> Vec<Vec<String>> {
    let start = PORT.find(after).map_or(PORT.len(), |i| i + after.len());
    PORT[start..]
        .lines()
        .skip_while(|l| !l.starts_with('|'))
        .take_while(|l| l.starts_with('|'))
        .skip(2)
        .map(|l| {
            l.trim_matches('|')
                .split('|')
                .map(|c| c.trim().to_owned())
                .collect()
        })
        .collect()
}

/// The first backticked id in a cell: the row's target; later ones are prose.
fn ticked(cell: &str) -> Option<String> {
    cell.split('`').nth(1).map(str::to_owned)
}

/// PORT.md's dropped checks, `set-skill-*` kept as a prefix.
fn dropped() -> Vec<String> {
    let rows = table("These were dropped:");
    let names = rows.iter().filter_map(|r| r.first());
    names
        .flat_map(|c| c.split(", ").map(str::to_owned).collect::<Vec<_>>())
        .collect()
}

fn is_dropped(command: &str, dropped: &[String]) -> bool {
    dropped.iter().any(|d| match d.strip_suffix('*') {
        Some(prefix) => command.starts_with(prefix),
        None => d == command,
    })
}

/// Where PORT.md's check table sends a legacy check: its first backticked
/// id, or the same name when the row says so.
fn mapped(command: &str) -> Option<String> {
    table("## Checks").into_iter().find_map(|row| {
        let (from, to) = (row.first()?, row.get(1)?);
        let names: Vec<&str> = from.split(", ").collect();
        names
            .contains(&command)
            .then(|| ticked(to).unwrap_or_else(|| command.to_owned()))
    })
}

fn ported(fragment: &str) -> Option<bool> {
    let rows = table("## Fragments");
    let row = rows.iter().find(|r| {
        r.first()
            .is_some_and(|f| f.split(", ").any(|n| n == fragment))
    })?;
    row.get(1).map(|c| c == "yes")
}

/// One legacy command of a ported fragment: dropped by name, or mapped
/// into `fragment`'s checks (`pklith check` stands for the verb itself).
fn landed(command: &str, fragment: &Fragment, dropped: &[String]) -> Result<(), String> {
    if is_dropped(command, dropped) {
        return Ok(());
    }
    let to = mapped(command).ok_or(format!("{command}: neither mapped nor dropped in PORT.md"))?;
    if to == "pklith check" || fragment.checks.contains(&to) {
        return Ok(());
    }
    Err(format!(
        "{command} maps to {to}, not in fragment {}",
        fragment.id
    ))
}

/// What is wrong with one legacy fragment line.
fn judged(line: &str, fragments: &[Fragment], dropped: &[String]) -> Vec<String> {
    let mut words = line.split(' ');
    let name = words.next().unwrap_or_default();
    let ours = fragments.iter().find(|f| f.id == name);
    match (ported(name), ours) {
        (Some(true), Some(f)) => words.filter_map(|c| landed(c, f, dropped).err()).collect(),
        (Some(false), None) => Vec::new(),
        (said, has) => vec![format!(
            "{name}: PORT.md says ported {said:?}, catalog has it: {}",
            has.is_some()
        )],
    }
}

#[test]
fn every_legacy_command_lands_or_is_named_as_dropped() -> Result<(), Box<dyn std::error::Error>> {
    let fragments = pklith::catalog::builtin_fragments()?;
    let dropped = dropped();
    let wrong: Vec<String> = LEGACY
        .lines()
        .flat_map(|l| judged(l, &fragments, &dropped))
        .collect();
    assert!(wrong.is_empty(), "{wrong:#?}");
    Ok(())
}

/// The assembler emitted a fragment named twice once; pklith's fragments
/// name each check once, so a step is never generated twice.
#[test]
fn no_fragment_names_a_check_twice() -> Result<(), Box<dyn std::error::Error>> {
    for f in pklith::catalog::builtin_fragments()? {
        let mut seen = f.checks.clone();
        seen.sort();
        seen.dedup();
        assert_eq!(seen.len(), f.checks.len(), "{}", f.id);
    }
    Ok(())
}
