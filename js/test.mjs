import test from 'node:test';
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
const L = createRequire(import.meta.url)('./simple-locale.js');
const SUP = ['en', 'uk'];

test('tags reduce to a language code', () => {
  assert.equal(L.normalize('uk-UA'), 'uk');
  assert.equal(L.normalize('uk_UA.UTF-8'), 'uk');
  assert.equal(L.normalize('C'), null);
  assert.equal(L.normalize('POSIX'), null);
  assert.equal(L.normalize(''), null);
});

test('English when nothing says otherwise', () => {
  assert.equal(L.detect(SUP, { search: '', configured: null, languages: [] }), 'en');
  assert.equal(L.detect(SUP, { search: '', configured: 'auto', languages: ['de-DE'] }), 'en');
});

test('the browser language picks Ukrainian, the first supported one of the list', () => {
  assert.equal(L.detect(SUP, { search: '', configured: 'auto', languages: ['uk-UA', 'en'] }), 'uk');
  assert.equal(L.detect(SUP, { search: '', configured: null, languages: ['de', 'uk'] }), 'uk');
});

test('order: ?lang= beats the server, the server beats the browser', () => {
  assert.equal(L.detect(SUP, { search: '?x=1&lang=en', configured: 'uk', languages: ['uk'] }), 'en');
  assert.equal(L.detect(SUP, { search: '', configured: 'en', languages: ['uk'] }), 'en');
  assert.equal(L.detect(SUP, { search: '?lang=xx', configured: 'uk', languages: ['en'] }), 'uk', 'an unsupported ?lang= is ignored');
});

test('a page that only has English always gets it', () => {
  assert.equal(L.detect(['en'], { search: '?lang=uk', configured: 'uk', languages: ['uk'] }), 'en');
});

test('makeT falls back to English, then to the key', () => {
  const t = L.makeT({ en: { a: 'A', b: 'B' }, uk: { a: 'А' } }, 'uk');
  assert.equal(t('a'), 'А');
  assert.equal(t('b'), 'B');
  assert.equal(t('zzz'), 'zzz');
});
