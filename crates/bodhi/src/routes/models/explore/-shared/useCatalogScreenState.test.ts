import { act, renderHook } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { useCatalogScreenState } from './useCatalogScreenState';

/**
 * The hook is generic over the URL search shape; this test uses a tiny shape with a couple of
 * facet fields (`type`, `lab`) and a few sort keys, then asserts the navigate() updater output by
 * applying it to a `prev` object — exactly how TanStack Router runs the functional updater.
 */
interface TestSearch {
  q?: string;
  sort?: 'relevance' | 'updated' | 'name';
  order?: 'asc' | 'desc';
  page?: number;
  select?: string;
  type?: string[];
  lab?: boolean;
}
interface TestFacets {
  type: string[];
  lab?: boolean;
}

const NATURAL: Record<NonNullable<TestSearch['sort']>, 'asc' | 'desc'> = {
  relevance: 'desc',
  updated: 'desc',
  name: 'asc',
};

function setup(search: TestSearch) {
  const navigate = vi.fn();
  const { result } = renderHook(() =>
    useCatalogScreenState<TestSearch, TestFacets, NonNullable<TestSearch['sort']>>({
      search,
      navigate,
      searchToFacets: (s) => ({ type: s.type ?? [], lab: s.lab }),
      facetsToSearch: (f) => ({ ...(f.type.length ? { type: f.type } : {}), ...(f.lab ? { lab: true } : {}) }),
      hasActiveFacets: (f) => f.type.length > 0 || !!f.lab,
      sortConfig: {
        storageKey: 'test.sort',
        persistedSorts: ['updated', 'name'],
        validOrders: ['asc', 'desc'],
        naturalOrder: (s) => NATURAL[s],
        searchRelevanceSort: 'relevance',
      },
    })
  );
  // Apply the last navigate() functional updater to `prev` and return the resulting search.
  const applyLast = (prev: TestSearch): TestSearch => {
    const arg = navigate.mock.calls.at(-1)![0];
    return arg.search(prev);
  };
  return { result, navigate, applyLast };
}

beforeEach(() => {
  localStorage.clear();
});

describe('useCatalogScreenState', () => {
  it('derives facets / committedSearch / page / selectedKey from the URL search', () => {
    const { result } = setup({ q: 'gpt', page: 2, select: 'a/b', type: ['chat'] });
    expect(result.current.committedSearch).toBe('gpt');
    expect(result.current.page).toBe(2);
    expect(result.current.selectedKey).toBe('a/b');
    expect(result.current.facets).toEqual({ type: ['chat'], lab: undefined });
  });

  it('commitSearch sets q + sort=relevance and drops page/order', () => {
    const { result, applyLast } = setup({ page: 3, order: 'asc' });
    act(() => result.current.commitSearch('llama'));
    const out = applyLast({ page: 3, order: 'asc' });
    expect(out).toEqual({ q: 'llama', sort: 'relevance' });
  });

  it('commitSearch with empty text drops q and sort', () => {
    const { result, applyLast } = setup({ q: 'x', sort: 'relevance' });
    act(() => result.current.commitSearch('   '));
    const out = applyLast({ q: 'x', sort: 'relevance', page: 2 });
    expect(out.q).toBeUndefined();
    expect(out.sort).toBeUndefined();
  });

  it('without a relevance sort, commitSearch only toggles q (sort/order untouched)', () => {
    const navigate = vi.fn();
    const { result } = renderHook(() =>
      useCatalogScreenState<TestSearch, TestFacets, NonNullable<TestSearch['sort']>>({
        search: { sort: 'name', order: 'desc' },
        navigate,
        searchToFacets: (s) => ({ type: s.type ?? [], lab: s.lab }),
        facetsToSearch: (f) => (f.type.length ? { type: f.type } : {}),
        hasActiveFacets: (f) => f.type.length > 0,
        sortConfig: {
          storageKey: 'test.sort2',
          persistedSorts: ['updated', 'name'],
          validOrders: ['asc', 'desc'],
          naturalOrder: (s) => NATURAL[s],
          // no searchRelevanceSort
        },
      })
    );
    act(() => result.current.commitSearch('hello'));
    const out = navigate.mock.calls.at(-1)![0].search({ sort: 'name', order: 'desc', page: 3 });
    expect(out).toEqual({ q: 'hello', sort: 'name', order: 'desc' }); // page dropped, sort/order kept
  });

  it('onSort adopts the natural order for a new column and omits order when natural', () => {
    const { result, applyLast } = setup({});
    act(() => result.current.onSort('name'));
    const out = applyLast({ page: 2 });
    expect(out.sort).toBe('name');
    expect(out.order).toBeUndefined(); // 'name' natural is 'asc' → omitted
    expect(out.page).toBeUndefined();
  });

  it('onSort toggles direction when clicking the active column', () => {
    const { result, applyLast } = setup({ sort: 'name' }); // name resolves to asc
    act(() => result.current.onSort('name'));
    const out = applyLast({ sort: 'name' });
    expect(out.sort).toBe('name');
    expect(out.order).toBe('desc'); // toggled away from natural asc → explicit
  });

  it('onFacetsChange replaces the facet slice while keeping q/sort/order', () => {
    const { result, applyLast } = setup({ q: 'x', sort: 'name', order: 'desc', type: ['old'] });
    act(() => result.current.onFacetsChange({ type: ['new'], lab: true }));
    const out = applyLast({ q: 'x', sort: 'name', order: 'desc', type: ['old'] });
    expect(out).toEqual({ q: 'x', sort: 'name', order: 'desc', type: ['new'], lab: true });
  });

  it('resetMode precedence: filters → query → none', () => {
    expect(setup({ type: ['chat'] }).result.current.resetMode).toBe('filters');
    expect(setup({ q: 'x' }).result.current.resetMode).toBe('query');
    expect(setup({}).result.current.resetMode).toBe('none');
  });

  it('onReset clears facets first, then the query', () => {
    const withFacets = setup({ q: 'x', type: ['chat'] });
    act(() => withFacets.result.current.onReset());
    expect(withFacets.applyLast({ q: 'x', type: ['chat'] })).toEqual({ q: 'x' }); // facets dropped, q kept

    const queryOnly = setup({ q: 'x' });
    act(() => queryOnly.result.current.onReset());
    const out = queryOnly.applyLast({ q: 'x' });
    expect(out.q).toBeUndefined();
  });

  it('onPage strips page on page 1 and sets it otherwise', () => {
    const { result, applyLast } = setup({});
    act(() => result.current.onPage(1));
    expect(applyLast({ page: 5 }).page).toBeUndefined();
    act(() => result.current.onPage(4));
    expect(applyLast({}).page).toBe(4);
  });

  it('select dedupes and writes select via replace', () => {
    const { result, navigate, applyLast } = setup({ select: 'a' });
    act(() => result.current.select('a')); // same → dedup, no navigate
    expect(navigate).not.toHaveBeenCalled();
    act(() => result.current.select('b'));
    expect(navigate.mock.calls.at(-1)![0].replace).toBe(true);
    expect(applyLast({}).select).toBe('b');
    act(() => result.current.select(null));
    expect(applyLast({ select: 'b' }).select).toBeUndefined();
  });
});
