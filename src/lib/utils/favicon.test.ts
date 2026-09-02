import { describe, expect, it } from 'vitest';
import { getFavicon } from './favicon';

describe('getFavicon', () => {
  it('returns a DuckDuckGo icon URL for a valid URL', () => {
    expect(getFavicon('https://example.com/feed.xml')).toBe('https://icons.duckduckgo.com/ip3/example.com.ico');
  });

  it('returns an empty string for an invalid URL', () => {
    expect(getFavicon('not a url')).toBe('');
  });
});
