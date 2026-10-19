// simple-locale for web pages: the twin of the Rust crate (see SPEC.md).
// Plain script (defines `SimpleLocale`) that also works as a CommonJS module.
//
//   const lang = SimpleLocale.detect(['en', 'uk']);          // in a page
//   const t = SimpleLocale.makeT({ en: {...}, uk: {...} }, lang);
//
// Order: `?lang=` in the URL, then the language the server was configured with (`<meta name="simple-lang">`, unless
// "auto"), then the browser's languages, then English.
(function (root, factory) {
  if (typeof module === 'object' && module.exports) module.exports = factory();
  else root.SimpleLocale = factory();
})(typeof self !== 'undefined' ? self : this, function () {
  var DEFAULT = 'en';

  // "uk-UA", "uk_UA.UTF-8", "en_GB@euro" -> "uk" / "en"; "C", "POSIX", "" -> null
  function normalize(tag) {
    if (!tag) return null;
    var base = String(tag).trim().split(/[.@]/)[0];
    var lang = base.split(/[_-]/)[0].toLowerCase();
    if (!lang || lang === 'c' || lang === 'posix' || !/^[a-z]+$/.test(lang)) return null;
    return lang;
  }

  // options (all optional, for tests and non-browser use): search, configured, languages
  function detect(supported, options) {
    var o = options || {};
    var sup = supported && supported.length ? supported : [DEFAULT];
    var ok = function (code) { return code && sup.indexOf(code) !== -1 ? code : null; };

    var search = o.search !== undefined ? o.search : (typeof location !== 'undefined' ? location.search : '');
    var q = /[?&]lang=([^&]*)/.exec(search || '');
    var fromUrl = q ? ok(normalize(decodeURIComponent(q[1]))) : null;
    if (fromUrl) return fromUrl;

    var configured = o.configured;
    if (configured === undefined && typeof document !== 'undefined') {
      var meta = document.querySelector('meta[name="simple-lang"]');
      configured = meta ? meta.getAttribute('content') : null;
    }
    if (configured && configured !== 'auto') {
      var fromServer = ok(normalize(configured));
      if (fromServer) return fromServer;
    }

    var languages = o.languages;
    if (languages === undefined && typeof navigator !== 'undefined') {
      languages = navigator.languages && navigator.languages.length ? navigator.languages : [navigator.language];
    }
    for (var i = 0; i < (languages || []).length; i++) {
      var fromBrowser = ok(normalize(languages[i]));
      if (fromBrowser) return fromBrowser;
    }
    return ok(DEFAULT) || sup[0];
  }

  // catalogs: { en: { key: 'text' }, uk: { ... } }; a missing key falls back to English, then to the key itself
  function makeT(catalogs, lang) {
    return function (key) {
      var own = catalogs[lang] || {};
      if (own[key] !== undefined) return own[key];
      var en = catalogs[DEFAULT] || {};
      return en[key] !== undefined ? en[key] : key;
    };
  }

  return { DEFAULT: DEFAULT, normalize: normalize, detect: detect, makeT: makeT };
});
