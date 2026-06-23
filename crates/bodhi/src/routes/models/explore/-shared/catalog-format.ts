/**
 * Shared formatting + display helpers for the two Explore · API catalog pages (Models + Providers).
 * Pure functions, no React — safe to unit-test directly.
 */
import type { Capability } from '@bodhiapp/reference-api-types';

/** Context window → compact "200K" / "1M"; "—" when null. */
export function fmtContext(tokens: number | null | undefined): string {
  if (tokens == null) return '—';
  if (tokens >= 1_000_000) return `${(tokens / 1_000_000).toFixed(tokens % 1_000_000 === 0 ? 0 : 1)}M`;
  if (tokens >= 1_000) return `${Math.round(tokens / 1_000)}K`;
  return String(tokens);
}

/** $/Mtok price; "—" when null. (Use {@link isFree} to decide a "Free" badge across in+out.) */
export function fmtPrice(perM: number | null | undefined): string {
  if (perM == null) return '—';
  if (perM === 0) return '$0';
  return perM < 1 ? `$${perM.toFixed(2)}` : `$${perM % 1 === 0 ? perM : perM.toFixed(2)}`;
}

/** A model/provider is "Free" when both input and output are 0. */
export function isFree(inPerM: number | null | undefined, outPerM: number | null | undefined): boolean {
  return inPerM === 0 && outPerM === 0;
}

/** First 1–2 letters for a logo monogram (logos currently 404 upstream). */
export function monogram(name: string): string {
  const words = name
    .trim()
    .split(/[\s-]+/)
    .filter(Boolean);
  if (words.length >= 2) return (words[0][0] + words[1][0]).toUpperCase();
  return name.trim().slice(0, 2).toUpperCase();
}

/** Stable per-name tint index (0–5) for monogram tiles — deterministic, palette-token driven. */
export function tintIndex(key: string): number {
  let h = 0;
  for (let i = 0; i < key.length; i++) h = (h * 31 + key.charCodeAt(i)) >>> 0;
  return h % 6;
}

export const CAP_LABELS: Record<Capability, string> = {
  reasoning: 'Reasoning',
  tool_call: 'Tool use',
  structured_output: 'Structured',
  attachment: 'Attachment',
  vision: 'Vision',
};

/** Capability → tone (drives the `cap-chip cap-<tone>` class; styled off the lotus palette tokens). */
export const CAP_TONE: Record<Capability, 'indigo' | 'muted'> = {
  reasoning: 'indigo',
  tool_call: 'indigo',
  vision: 'indigo',
  structured_output: 'muted',
  attachment: 'muted',
};
