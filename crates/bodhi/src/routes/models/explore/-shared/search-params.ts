import { z } from 'zod';

/**
 * A repeatable enum URL param. Repeatable params arrive as an array (2+ values) or a bare string
 * (one value, e.g. a cross-route link). Coerce a lone value into a one-element array, drop members
 * outside the allowed set, and omit the key entirely when nothing survives. Shared by every catalog
 * route schema (Explore · API Models / Providers / Local, My Models).
 */
export function arrayParam<T extends readonly [string, ...string[]]>(values: T) {
  const allowed = new Set<string>(values);
  return z.preprocess(
    (v) => {
      if (v == null) return undefined;
      const arr = (Array.isArray(v) ? v : [v]).filter((x): x is string => typeof x === 'string' && allowed.has(x));
      return arr.length ? arr : undefined;
    },
    z.array(z.enum(values)).optional()
  );
}
