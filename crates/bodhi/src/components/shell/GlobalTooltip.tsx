import { useEffect, useLayoutEffect, useRef, useState } from 'react';

/* ── Global tooltip (fixed-position; escapes sidebar overflow) ──
   One listener delegates over `.shell-tip` elements only — i.e. the
   COLLAPSED icon-rail buttons (and the avatar). Expanded sidebar items
   already show their label inline, and popover/popup options carry no
   tooltip, so neither fires a hint. Flips left near the viewport edge. */
interface TipState {
  text: string | null;
  top: number;
  aRight: number;
  aLeft: number;
}

export function GlobalTooltip() {
  const [tip, setTip] = useState<TipState | null>(null);
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    let cur: Element | null = null;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const onOver = (e: MouseEvent) => {
      const target = e.target as Element | null;
      const el = target?.closest?.('.shell-tip[data-tip]') ?? null;
      if (el === cur) return;
      cur = el;
      clearTimeout(timer);
      if (!el) {
        setTip(null);
        return;
      }
      timer = setTimeout(() => {
        if (cur !== el) return;
        const r = el.getBoundingClientRect();
        setTip({ text: el.getAttribute('data-tip'), top: r.top + r.height / 2, aRight: r.right, aLeft: r.left });
      }, 150);
    };
    const onOut = (e: MouseEvent) => {
      const target = e.target as Element | null;
      const el = target?.closest?.('.shell-tip[data-tip]') ?? null;
      if (!el || el !== cur) return;
      const related = e.relatedTarget as Node | null;
      if (related && el.contains(related)) return;
      cur = null;
      clearTimeout(timer);
      setTip(null);
    };
    document.addEventListener('mouseover', onOver, true);
    document.addEventListener('mouseout', onOut, true);
    return () => {
      document.removeEventListener('mouseover', onOver, true);
      document.removeEventListener('mouseout', onOut, true);
      clearTimeout(timer);
    };
  }, []);
  useLayoutEffect(() => {
    if (!tip || !ref.current) return;
    const w = ref.current.offsetWidth;
    let left = tip.aRight + 10;
    if (left + w > window.innerWidth - 8) left = Math.max(8, tip.aLeft - 10 - w);
    ref.current.style.left = left + 'px';
  }, [tip]);
  if (!tip || !tip.text) return null;
  return (
    <div ref={ref} className="shell-tooltip" style={{ top: tip.top, left: tip.aRight + 10 }}>
      {tip.text}
    </div>
  );
}
