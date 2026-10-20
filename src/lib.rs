//! One language setting for every Simple Linux program (see `SPEC.md`).
//!
//! The language of a program is the first of these that names a language the program supports:
//!
//! 1. the program's own override variable (`USTAN_LANG`, ...),
//! 2. `SIMPLE_LANG`,
//! 3. `lang` in the user's `locale.toml` (`~/.config/simple-linux/`),
//! 4. `lang` in the system's `/etc/simple-linux/locale.toml`,
//! 5. the system locale: `LC_ALL`, `LC_MESSAGES`, `LANGUAGE`, `LANG`,
//! 6. `fallback` of the config files, else `en`.
//!
//! `lang = "auto"` (or no `lang`) in a config file means "ask the next source". `C` and `POSIX` are not languages.
//! A language the program does not support is skipped, never an error.

use std::path::PathBuf;

/// The language-independent default.
pub const DEFAULT: &str = "en";

/// The variable that overrides the language of every program.
pub const ENV_LANG: &str = "SIMPLE_LANG";

/// The JavaScript twin of this crate (`js/simple-locale.js`), for pages a Rust server hands out.
pub const JS: &str = include_str!("../js/simple-locale.js");

/// What a `locale.toml` says.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Config {
    /// `lang`: a language code; `None` for `auto` or when the key is missing.
    pub lang: Option<String>,
    /// `fallback`: the language when nothing else decides.
    pub fallback: Option<String>,
}

/// Reads the two keys of a `locale.toml` (`lang = "uk"`, `fallback = "en"`; `#` starts a comment). Nothing else is understood or needed.
pub fn parse_config(text: &str) -> Config {
    let mut cfg = Config::default();
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim().trim_matches('"').trim_matches('\'').trim();
        let code = if value.is_empty() || value.eq_ignore_ascii_case("auto") {
            None
        } else {
            normalize(value)
        };
        match key.trim() {
            "lang" => cfg.lang = code,
            "fallback" => cfg.fallback = code,
            _ => {}
        }
    }
    cfg
}

/// `uk_UA.UTF-8@euro` → `uk`, `en-GB` → `en`, `C` / `POSIX` / empty → `None`.
pub fn normalize(tag: &str) -> Option<String> {
    let tag = tag.trim();
    let base = tag.split(['.', '@']).next().unwrap_or("");
    let lang = base
        .split(['_', '-'])
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    if lang.is_empty()
        || lang == "c"
        || lang == "posix"
        || !lang.chars().all(|c| c.is_ascii_alphabetic())
    {
        return None;
    }
    Some(lang)
}

/// Where the user's and the system's `locale.toml` are.
pub fn config_paths() -> (Option<PathBuf>, PathBuf) {
    let user = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .filter(|v| !v.is_empty())
                .map(|h| PathBuf::from(h).join(".config"))
        })
        .map(|d| d.join("simple-linux/locale.toml"));
    (user, PathBuf::from("/etc/simple-linux/locale.toml"))
}

/// The language of the program: `supported` are the codes it has texts for (the first one is its own default).
/// `override_var` is the program's own variable (`Some("USTAN_LANG")`).
pub fn resolve(supported: &[&str], override_var: Option<&str>) -> String {
    let (user, system) = config_paths();
    let read = |p: Option<PathBuf>| p.and_then(|p| std::fs::read_to_string(p).ok());
    resolve_with(
        supported,
        override_var,
        |k| std::env::var(k).ok(),
        read(user).as_deref(),
        read(Some(system)).as_deref(),
    )
}

/// [`resolve`] with everything it reads handed in, so that the rules can be tested.
pub fn resolve_with(
    supported: &[&str],
    override_var: Option<&str>,
    env: impl Fn(&str) -> Option<String>,
    user_config: Option<&str>,
    system_config: Option<&str>,
) -> String {
    let pick = |code: Option<String>| code.filter(|c| supported.contains(&c.as_str()));
    let user = user_config.map(parse_config).unwrap_or_default();
    let system = system_config.map(parse_config).unwrap_or_default();

    let from_env = |name: &str| {
        env(name)
            .filter(|v| !v.is_empty())
            .and_then(|v| normalize(&v))
    };
    if let Some(var) = override_var {
        if let Some(l) = pick(from_env(var)) {
            return l;
        }
    }
    if let Some(l) = pick(from_env(ENV_LANG)) {
        return l;
    }
    for cfg in [&user, &system] {
        if let Some(l) = pick(cfg.lang.clone()) {
            return l;
        }
    }
    for var in ["LC_ALL", "LC_MESSAGES"] {
        if let Some(l) = pick(from_env(var)) {
            return l;
        }
    }
    // LANGUAGE is a list: `uk:en`
    if let Some(list) = env("LANGUAGE") {
        for item in list.split(':') {
            if let Some(l) = pick(normalize(item)) {
                return l;
            }
        }
    }
    if let Some(l) = pick(from_env("LANG")) {
        return l;
    }
    for cfg in [&user, &system] {
        if let Some(l) = pick(cfg.fallback.clone()) {
            return l;
        }
    }
    if supported.contains(&DEFAULT) {
        DEFAULT.to_string()
    } else {
        supported.first().copied().unwrap_or(DEFAULT).to_string()
    }
}

/// The language the configuration asks for, without looking at the locale: what a server tells its web page
/// (`None`: the page decides from the browser's language).
pub fn configured(supported: &[&str], override_var: Option<&str>) -> Option<String> {
    let (user, system) = config_paths();
    let read = |p: Option<PathBuf>| p.and_then(|p| std::fs::read_to_string(p).ok());
    configured_with(
        supported,
        override_var,
        |k| std::env::var(k).ok(),
        read(user).as_deref(),
        read(Some(system)).as_deref(),
    )
}

/// [`configured`] with everything it reads handed in.
pub fn configured_with(
    supported: &[&str],
    override_var: Option<&str>,
    env: impl Fn(&str) -> Option<String>,
    user_config: Option<&str>,
    system_config: Option<&str>,
) -> Option<String> {
    let pick = |code: Option<String>| code.filter(|c| supported.contains(&c.as_str()));
    let from_env = |name: &str| {
        env(name)
            .filter(|v| !v.is_empty())
            .and_then(|v| normalize(&v))
    };
    override_var
        .and_then(|v| pick(from_env(v)))
        .or_else(|| pick(from_env(ENV_LANG)))
        .or_else(|| pick(user_config.map(parse_config).and_then(|c| c.lang)))
        .or_else(|| pick(system_config.map(parse_config).and_then(|c| c.lang)))
}

/// The two tags a Rust server puts into the `<head>` of a page: the configured language (`auto` when the browser should
/// decide) and the script of `js/simple-locale.js`, which defines `SimpleLocale`.
pub fn html_head(supported: &[&str], override_var: Option<&str>) -> String {
    let configured = configured(supported, override_var).unwrap_or_else(|| "auto".to_string());
    format!("<meta name=\"simple-lang\" content=\"{configured}\">\n<script>\n{JS}\n</script>")
}

/// `t!("English {x}", "Українською {x}")`-style helpers are left to each program: this crate only decides the language.
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let m: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |k| m.get(k).cloned()
    }
    const SUP: &[&str] = &["en", "uk"];

    #[test]
    fn tags_are_reduced_to_a_language_code() {
        assert_eq!(normalize("uk_UA.UTF-8").as_deref(), Some("uk"));
        assert_eq!(normalize("en-GB").as_deref(), Some("en"));
        assert_eq!(normalize("de_DE@euro").as_deref(), Some("de"));
        assert_eq!(normalize("C"), None);
        assert_eq!(normalize("POSIX"), None);
        assert_eq!(normalize(""), None);
        assert_eq!(normalize("C.UTF-8"), None);
    }

    #[test]
    fn a_config_file_has_lang_and_fallback_and_auto_means_none() {
        let c = parse_config("# my setup\nlang = \"uk\"  # Ukrainian\nfallback='en'\nother = 1\n");
        assert_eq!(
            (c.lang.as_deref(), c.fallback.as_deref()),
            (Some("uk"), Some("en"))
        );
        assert_eq!(parse_config("lang = \"auto\"\n").lang, None);
        assert_eq!(parse_config("").lang, None);
        assert_eq!(
            parse_config("lang = uk_UA.UTF-8").lang.as_deref(),
            Some("uk")
        );
    }

    #[test]
    fn english_when_nothing_says_otherwise() {
        assert_eq!(resolve_with(SUP, None, env(&[]), None, None), "en");
        assert_eq!(
            resolve_with(SUP, None, env(&[("LANG", "C.UTF-8")]), None, None),
            "en"
        );
        assert_eq!(
            resolve_with(SUP, None, env(&[("LANG", "de_DE.UTF-8")]), None, None),
            "en",
            "unsupported: skipped"
        );
    }

    #[test]
    fn the_system_locale_picks_ukrainian() {
        assert_eq!(
            resolve_with(SUP, None, env(&[("LANG", "uk_UA.UTF-8")]), None, None),
            "uk"
        );
        assert_eq!(
            resolve_with(SUP, None, env(&[("LANGUAGE", "de:uk:en")]), None, None),
            "uk"
        );
        // LC_ALL beats LANG
        assert_eq!(
            resolve_with(
                SUP,
                None,
                env(&[("LC_ALL", "en_US.UTF-8"), ("LANG", "uk_UA.UTF-8")]),
                None,
                None
            ),
            "en"
        );
    }

    #[test]
    fn the_order_is_override_simple_lang_user_system_locale_fallback() {
        let e = env(&[
            ("USTAN_LANG", "en"),
            ("SIMPLE_LANG", "uk"),
            ("LANG", "uk_UA.UTF-8"),
        ]);
        assert_eq!(
            resolve_with(SUP, Some("USTAN_LANG"), &e, Some("lang = \"uk\""), None),
            "en",
            "the program's own variable first"
        );
        assert_eq!(
            resolve_with(SUP, None, &e, Some("lang = \"en\""), None),
            "uk",
            "SIMPLE_LANG before the files"
        );
        let none = env(&[("LANG", "uk_UA.UTF-8")]);
        assert_eq!(
            resolve_with(
                SUP,
                None,
                &none,
                Some("lang = \"en\""),
                Some("lang = \"uk\"")
            ),
            "en",
            "user before system"
        );
        assert_eq!(
            resolve_with(SUP, None, &none, None, Some("lang = \"en\"")),
            "en",
            "a file before the locale"
        );
        assert_eq!(
            resolve_with(SUP, None, &none, Some("lang = \"auto\""), None),
            "uk",
            "auto asks the locale"
        );
        assert_eq!(
            resolve_with(SUP, None, env(&[]), Some("fallback = \"uk\""), None),
            "uk",
            "the fallback last"
        );
    }

    #[test]
    fn a_program_that_only_has_english_always_gets_it() {
        assert_eq!(
            resolve_with(
                &["en"],
                None,
                env(&[("LANG", "uk_UA.UTF-8")]),
                Some("lang = \"uk\""),
                None
            ),
            "en"
        );
    }

    #[test]
    fn a_server_tells_its_page_only_what_the_configuration_says() {
        assert_eq!(
            configured_with(SUP, None, env(&[("LANG", "uk_UA.UTF-8")]), None, None),
            None,
            "the locale is not configuration"
        );
        assert_eq!(
            configured_with(SUP, None, env(&[]), Some("lang = \"uk\""), None).as_deref(),
            Some("uk")
        );
        assert_eq!(
            configured_with(
                SUP,
                Some("RING2ZERO_LANG"),
                env(&[("RING2ZERO_LANG", "en")]),
                Some("lang = \"uk\""),
                None
            )
            .as_deref(),
            Some("en")
        );
    }

    #[test]
    fn the_page_head_carries_the_script() {
        let h = html_head(SUP, None);
        assert!(h.contains("<meta name=\"simple-lang\" content=\""));
        assert!(h.contains("SimpleLocale"), "{h}");
    }
}
