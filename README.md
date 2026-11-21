# Wortfilter

A plugin for [Hauch](https://github.com/axiomata-labs): your own replacements for words its recogniser gets wrong —
"Lirfel" → "Lirvel" — applied to everything Hauch recognises before it shows it. Whole words only, regardless of
case, the longest first.

Install it on Hauch's page "Plugins" with `axiomata-labs/hauch-wortfilter`, or:

```sh
hauch plugins install axiomata-labs/hauch-wortfilter
```

Its page lists the replacements and adds new ones; since 0.2.0 it has an entry of its own in Hauch's navigation
(the place `sidebar`, which the user may take back). It asks for one right, `storage`: a folder of its own for the
list.

Built from [axiomata-plugin-template](https://github.com/axiomata-labs/axiomata-plugin-template) in Rust:
`scripts/release.sh 0.2.1` builds `dist/`, a tag `v0.2.1` publishes a release.

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
