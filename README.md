# pklith

Materialize [hk](https://hk.jdx.dev) git hook guardrails from what a
repository's files require.

pklith looks at every tracked file, works out which linters and checks its
type needs (several per type, plus rules such as "every implementation has a
test"), and turns that into hk steps and the nix packages that run them. It
can lay those checks into a repository one commit per check, and it reports
and enforces that no file goes unchecked.

The command is `pkli`. It is not part of [Pkl](https://pkl-lang.org), the
configuration language hk uses; the name only borrows it.

Status: specification only. The design lives in [`SPEC.md`](SPEC.md) and the
`SPEC.md` files under `src/`.

## License

MIT
