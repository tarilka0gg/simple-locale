# simple-locale

[![CI](https://github.com/tarilka0gg/simple-locale/actions/workflows/ci.yml/badge.svg)](https://github.com/tarilka0gg/simple-locale/actions/workflows/ci.yml) [![License](https://img.shields.io/github/license/tarilka0gg/simple-locale)](LICENSE) [![Changelog](https://img.shields.io/badge/changelog-CHANGELOG.md-blue)](CHANGELOG.md)

One language setting for all Simple Linux programs. A tiny Rust crate and a JavaScript twin that answer one question, *which
language should this program speak?*, the same way everywhere: a shared config file, the system locale, English by default.
The full rule is in [SPEC.md](SPEC.md).

```toml
# ~/.config/simple-linux/locale.toml  (or /etc/simple-linux/locale.toml for the whole machine)
lang = "auto"     # "auto" asks the system locale; or "en", "uk", ...
fallback = "en"
```

`SIMPLE_LANG=uk some-program` beats the files for one run; a program may also have its own variable (`USTAN_LANG`).

## Rust

```toml
[dependencies]
simple-locale = { git = "https://github.com/tarilka0gg/simple-locale" }
```

```rust
let lang = simple_locale::resolve(&["en", "uk"], Some("USTAN_LANG")); // "en" or "uk"
```

No dependencies. `resolve_with` takes the environment and the file texts as arguments, which is how the rules are tested.

## A page served by a Rust program

```rust
let head = simple_locale::html_head(&["en", "uk"], Some("RING2ZERO_LANG")); // <meta name="simple-lang"> + the script
```

and in the page: `const lang = SimpleLocale.detect(['en', 'uk']); const t = SimpleLocale.makeT(catalogs, lang);`.
`js/simple-locale.js` also works on its own (it is a plain script and a CommonJS module).

## Tests

```
cargo test
node --test js/test.mjs
```

## Changelog

What changed in each version is in [CHANGELOG.md](CHANGELOG.md).

## License

MIT, see [LICENSE](LICENSE).
