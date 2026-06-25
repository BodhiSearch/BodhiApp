// Shared sort-preference resolver for the Explore catalog pages (API Models + API Providers).
//
// Behavior (identical on both pages, different storage keys):
//   - The URL `?sort=`/`?order=` is the strongest signal — a shared/deep link wins so it round-trips.
//   - Otherwise a previously-persisted preference (localStorage) is applied to the REQUEST silently;
//     it is deliberately NOT written back to the URL, so the URL stays clean until the user clicks.
//   - Otherwise nothing → effective sort is `undefined` → the caller omits sort/order and the API
//     returns its natural order.
//
// This module is pure + framework-free (no router/React coupling): the screen owns useSearch()/
// navigate() and just feeds the current URL sort/order in. That keeps the read-once discipline
// (no effect that writes the URL → no render loop) and makes the logic trivially unit-testable.

export type SortPref<S extends string, O extends string> = { sort: S; order: O };

function isBrowser(): boolean {
  return typeof window !== 'undefined' && typeof window.localStorage !== 'undefined';
}

/** Read a persisted `{sort, order}` pref, dropping anything not in the page's valid sort/order sets. */
export function readSortPreference<S extends string, O extends string>(
  storageKey: string,
  validSorts: readonly S[],
  validOrders: readonly O[]
): SortPref<S, O> | null {
  if (!isBrowser()) return null;
  let raw: string | null = null;
  try {
    raw = window.localStorage.getItem(storageKey);
  } catch {
    return null;
  }
  if (!raw) return null;
  try {
    const parsed = JSON.parse(raw) as { sort?: unknown; order?: unknown };
    const sort = parsed?.sort;
    const order = parsed?.order;
    if (typeof sort !== 'string' || !(validSorts as readonly string[]).includes(sort)) return null;
    if (typeof order !== 'string' || !(validOrders as readonly string[]).includes(order)) return null;
    return { sort: sort as S, order: order as O };
  } catch {
    return null;
  }
}

/** Persist the user's explicit sort pick. Best-effort — storage failures (private mode, quota) are ignored. */
export function persistSortPreference<S extends string, O extends string>(storageKey: string, sort: S, order: O): void {
  if (!isBrowser()) return;
  try {
    window.localStorage.setItem(storageKey, JSON.stringify({ sort, order }));
  } catch {
    // ignore
  }
}

/**
 * Resolve the effective sort/order for a render given the current URL params and the page config.
 *
 * Returns `{ sort: undefined, order: undefined, fromStorage: false }` when nothing is set (→ API
 * natural order). `fromStorage` is true when the pref came from localStorage rather than the URL —
 * the caller uses it to decide whether to ALSO reflect the sort in the URL (it should not; storage
 * prefs stay out of the URL).
 */
export function resolveSortPreference<S extends string, O extends string>(opts: {
  urlSort: S | undefined;
  urlOrder: O | undefined;
  storageKey: string;
  validSorts: readonly S[];
  validOrders: readonly O[];
  /** Natural order for a given sort key, used when the URL/stored pref omits `order`. */
  naturalOrder: (sort: S) => O;
}): { sort: S | undefined; order: O | undefined; fromStorage: boolean } {
  const { urlSort, urlOrder, storageKey, validSorts, validOrders, naturalOrder } = opts;

  // URL wins (shareable links).
  if (urlSort && (validSorts as readonly string[]).includes(urlSort)) {
    return { sort: urlSort, order: urlOrder ?? naturalOrder(urlSort), fromStorage: false };
  }

  // Else fall back to the persisted preference (applied to the request only).
  const stored = readSortPreference(storageKey, validSorts, validOrders);
  if (stored) {
    return { sort: stored.sort, order: stored.order, fromStorage: true };
  }

  // Else no sort → natural order from the API.
  return { sort: undefined, order: undefined, fromStorage: false };
}
