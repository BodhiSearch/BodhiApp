// Sort-preference resolver shared by the Explore catalog pages. Precedence: URL (deep-link) wins,
// else a persisted localStorage pref applies silently (never written back to the URL), else natural
// API order. Kept pure/framework-free so the screen's read-once URL discipline holds.

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
 * Resolves the effective sort/order for a render. `fromStorage: true` means the pref came from
 * localStorage, not the URL — the caller must not write it back to the URL.
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

  const stored = readSortPreference(storageKey, validSorts, validOrders);
  if (stored) {
    return { sort: stored.sort, order: stored.order, fromStorage: true };
  }

  return { sort: undefined, order: undefined, fromStorage: false };
}
