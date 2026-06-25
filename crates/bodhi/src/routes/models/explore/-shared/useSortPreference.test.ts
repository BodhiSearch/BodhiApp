import { afterEach, beforeEach, describe, expect, it } from 'vitest';

import {
  persistSortPreference,
  readSortPreference,
  resolveSortPreference,
} from '@/routes/models/explore/-shared/useSortPreference';

const KEY = 'bodhi.explore.test.sort';
const SORTS = ['name', 'model_count', 'api_format'] as const;
const ORDERS = ['asc', 'desc'] as const;
type Sort = (typeof SORTS)[number];
type Order = (typeof ORDERS)[number];

const naturalOrder = (s: Sort): Order => (s === 'name' || s === 'api_format' ? 'asc' : 'desc');

function resolve(urlSort?: Sort, urlOrder?: Order) {
  return resolveSortPreference<Sort, Order>({
    urlSort,
    urlOrder,
    storageKey: KEY,
    validSorts: SORTS,
    validOrders: ORDERS,
    naturalOrder,
  });
}

describe('useSortPreference', () => {
  beforeEach(() => localStorage.clear());
  afterEach(() => localStorage.clear());

  it('URL sort wins over a stored preference', () => {
    persistSortPreference(KEY, 'name', 'asc');
    expect(resolve('model_count', 'desc')).toEqual({ sort: 'model_count', order: 'desc', fromStorage: false });
  });

  it('URL sort without order adopts the natural order', () => {
    expect(resolve('name')).toEqual({ sort: 'name', order: 'asc', fromStorage: false });
    expect(resolve('model_count')).toEqual({ sort: 'model_count', order: 'desc', fromStorage: false });
  });

  it('falls back to the stored preference when the URL has no sort (fromStorage=true)', () => {
    persistSortPreference(KEY, 'api_format', 'asc');
    expect(resolve()).toEqual({ sort: 'api_format', order: 'asc', fromStorage: true });
  });

  it('returns undefined sort when neither URL nor storage has a preference (API natural order)', () => {
    expect(resolve()).toEqual({ sort: undefined, order: undefined, fromStorage: false });
  });

  it('persist round-trips through read', () => {
    persistSortPreference(KEY, 'model_count', 'desc');
    expect(readSortPreference(KEY, SORTS, ORDERS)).toEqual({ sort: 'model_count', order: 'desc' });
  });

  it('ignores an invalid/stale stored sort key', () => {
    localStorage.setItem(KEY, JSON.stringify({ sort: 'rank', order: 'desc' })); // 'rank' no longer valid
    expect(readSortPreference(KEY, SORTS, ORDERS)).toBeNull();
    expect(resolve()).toEqual({ sort: undefined, order: undefined, fromStorage: false });
  });

  it('ignores malformed JSON in storage', () => {
    localStorage.setItem(KEY, 'not-json');
    expect(readSortPreference(KEY, SORTS, ORDERS)).toBeNull();
  });
});
