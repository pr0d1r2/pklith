use super::{Facts, render, stale_prose};
use crate::facts::Nixpkgs;

fn nixpkgs() -> Nixpkgs {
    let (series, rev, day) = ("26.05", "f5c082a", "2026-09-25");
    Nixpkgs {
        series: series.into(),
        rev: rev.into(),
        day: day.into(),
    }
}

fn facts() -> Facts {
    Facts {
        slug: "o/r".into(),
        name: "pklith".into(),
        license: "MIT".into(),
        edition: "2024".into(),
        msrv: "1.95".into(),
        dependencies: 1,
        floor: "100".into(),
        steps: (38, 45),
        spec: (119, 21, 20),
        nixpkgs: nixpkgs(),
        platforms: vec![("arm", "macos")],
    }
}

/// Alt text and URL come from the same value, escaped the shields.io way.
#[test]
fn the_block_renders_every_group() {
    let block = render(&facts());
    for want in [
        "[![CI](https://github.com/o/r/actions/workflows/ci.yml/badge.svg)](https://github.com/o/r/actions/workflows/ci.yml)\n",
        "[![crates.io](https://img.shields.io/crates/v/pklith.svg)](https://crates.io/crates/pklith)\n[![docs.rs](https://docs.rs/pklith/badge.svg)](https://docs.rs/pklith)\n",
        "[![gate steps 38 commit / 45 push](https://img.shields.io/badge/gate_steps-38_commit_%2F_45_push-6E4AFF)](hk.pkl)\n",
        "[![coverage floor 100%](https://img.shields.io/badge/coverage_floor-%E2%89%A5100%25-brightgreen)](.coverage)\n",
        "[![nixpkgs 26.05 (2026-09-25 - f5c082a)](https://img.shields.io/badge/nixpkgs-26.05_(2026--09--25_--_f5c082a)-5277C3?logo=nixos&logoColor=white)](flake.lock)\n",
        "[![arm macos](https://img.shields.io/badge/macos-5277C3?logo=arm&logoColor=white)](.github/workflows/ci.yml)\n",
        "[![built with SDD](https://img.shields.io/badge/built_with-spec--driven_development-D97757)](SPEC.md)\n<!-- END badges -->\n",
    ] {
        assert!(block.contains(want), "missing {want}");
    }
}

/// An empty or zero fact refuses the render, naming it (dev V1).
#[test]
fn an_empty_fact_is_refused() {
    assert!(facts().checked().is_ok());
    let empty = Facts {
        dependencies: 0,
        ..facts()
    };
    assert_eq!(
        empty.checked().err().as_deref(),
        Some("Cargo.toml [dependencies] read empty or zero; fix the source")
    );
}

/// The disclaimer's numbers must be the facts', across wrapped lines.
#[test]
fn stale_prose_is_named() {
    let good = "**38 steps on commit,\n45 on push** and **119 `§V`\ninvariants** and **21 `§B` bugs**, a floor of 100% here";
    assert!(stale_prose(good, &facts()).is_empty());
    let stale = stale_prose(&good.replace("119", "36"), &facts());
    assert_eq!(stale, ["**119 `§V` invariants**"]);
}
