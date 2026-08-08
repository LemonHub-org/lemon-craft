# Contributing

Thank you for contributing to LemonCraft!

## Where to contribute

- Code: open a pull request against [the repository](https://github.com/LemonHub-org/craft)
- Issues: GitHub issues

## Guidelines

- Keep changes focused and small; discuss larger designs (gameplay direction,
  architecture) in an issue first.
- Follow the existing code style: `cargo fmt` and `cargo clippy -- -D warnings`
  must pass (see the verification section below).
- LemonCraft has diverged from upstream Veloren. Upstream policies do not apply;
  treat this repository's conventions as the source of truth.
- Do not commit content you do not have the right to distribute under
  GPL-3.0-or-later.

## Verification

Run the CI checks before opening a PR (all use `--locked`):

1. `cargo clippy --all-targets --locked --features="bin_cmd_doc_gen,bin_compression,bin_csv,bin_graphviz,bin_bot,bin_asset_migrate,bin,stat,cli" -- -D warnings`
2. `cargo clippy -p lemoncraft-voxygen --locked --no-default-features --features="default-publish" -- -D warnings`
3. `cargo clippy --locked --bin lemoncraft-server-cli --no-default-features -F simd -- -D warnings`
4. `cargo fmt --all -- --check`
