# Wortfilter

A plugin for [Hauch](https://github.com/axiomata-labs): your own replacements for words its recogniser gets wrong —
"Lirfel" → "Lirvel" — applied to everything Hauch recognises before it shows it. Whole words only, regardless of
case, the longest first.

Install it on Hauch's page "Plugins" with `axiomata-labs/hauch-wortfilter`, or:

```sh
hauch plugins install axiomata-labs/hauch-wortfilter
```

Its page there lists the replacements and adds new ones. It asks for one right, `storage`: a folder of its own for the
list.

Built from [axiomata-plugin-template](https://github.com/axiomata-labs/axiomata-plugin-template) in Rust:
`scripts/release.sh 0.1.0` builds `dist/`, a tag `v0.1.0` publishes a release.

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
