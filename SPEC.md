# One language setting for Simple Linux programs

Every program of the Simple Linux projects (the installer, ustan, Ring-2Zero, Portage Store, tui-shim, ...) chooses its interface
language the same way, so that one file or one variable changes all of them. English is the default; a program that has no
translation simply stays English.

## The rule

The language of a program is the **first** of these that names a language the program supports:

| # | Source | Example |
|---|---|---|
| 1 | the program's own variable (the program documents it) | `USTAN_LANG=uk` |
| 2 | `SIMPLE_LANG` | `SIMPLE_LANG=uk` |
| 3 | `lang` in the user's `~/.config/simple-linux/locale.toml` (`$XDG_CONFIG_HOME` is honoured) | `lang = "uk"` |
| 4 | `lang` in the machine's `/etc/simple-linux/locale.toml` | `lang = "uk"` |
| 5 | the system locale: `LC_ALL`, `LC_MESSAGES`, `LANGUAGE` (a `:` list; the first supported item), `LANG` | `LANG=uk_UA.UTF-8` |
| 6 | `fallback` of the config files (user, then system) | `fallback = "en"` |
| 7 | `en` (or the program's first supported language if it has no English) | |

* A tag is reduced to its language: `uk_UA.UTF-8`, `uk-UA` and `uk` are all `uk`.
* `C`, `POSIX` and empty values are not languages; they are skipped.
* `lang = "auto"` (or no `lang`) means "ask the next source".
* A language the program does not support is **skipped**, not an error: `LANG=de_DE.UTF-8` gives a program with `en` and `uk` English.
* Reading a missing or broken file is not an error either; it is the same as an empty one.

## The file

`locale.toml` has two keys; nothing else is read:

```toml
lang = "auto"      # "auto", or a code such as "en", "uk"
fallback = "en"    # used when nothing else decides
```

`#` starts a comment. The file is the same in the user's and in the system's place; the user's wins.

## Web pages

A page served by a program (Ring-2Zero's client) decides in the browser, in this order:

1. `?lang=uk` in the URL,
2. the language the server was configured with, handed over as `<meta name="simple-lang" content="uk">` (`auto` when nothing is configured),
3. the browser's languages (`navigator.languages`), the first supported one,
4. `en`.

The server's side is the same crate: `simple_locale::configured()` reads rules 1-4 above (without the locale, because the
server's locale says nothing about the person at the browser) and `simple_locale::html_head()` writes the `<meta>` tag and the script.

## What this is not

Message catalogs stay in each program: the crate and the script only decide *which* language. A program picks its texts with
its own mechanism (`ustan` has `t!(uk, en)`, Ring-2Zero's page has a small dictionary).

## Programs

| Program | Languages | Own variable | Uses |
|---|---|---|---|
| ustan | en, uk | `USTAN_LANG` | the Rust crate |
| Ring-2Zero (client page) | en, uk | `RING2ZERO_LANG` | the crate (server) and the script (page) |
| gentoo-installer, Portage Store, tui-shim | en | | nothing yet; they join by calling `resolve(&["en", ...], ...)` when they get a translation |
