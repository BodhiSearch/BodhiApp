import { useEffect, useState } from 'react';

import { Slider } from '@/components/ui/slider';
import './catalog.css';

/**
 * Debounced slider controls shared across the catalog/list screens (Explore · API Models sidebar,
 * My Models size facet, …). Each emits `onCommit` only on release (`onValueCommit`), so a drag fires
 * one query, not N. The value label is hidden at the default position and revealed on hover/drag via
 * the `.cat-range-val.visible` class.
 */

/** A debounced single-thumb range slider: emits onCommit only when the user releases. */
export function RangeControl({
  value,
  display,
  max,
  step,
  format,
  testId,
  onCommit,
}: {
  value: number;
  display?: number;
  max: number;
  step: number;
  format: (v: number) => string;
  testId: string;
  onCommit: (v: number) => void;
}) {
  const [local, setLocal] = useState(display ?? value);
  const [dragging, setDragging] = useState(false);
  const [hovering, setHovering] = useState(false);
  useEffect(() => {
    setLocal(display ?? value);
  }, [display, value]);

  const isDefault = local <= 0;
  const showVal = !isDefault || dragging || hovering;

  return (
    <div
      className="cat-range-stack"
      data-testid={testId}
      onMouseEnter={() => setHovering(true)}
      onMouseLeave={() => setHovering(false)}
    >
      <div className="cat-range-head cat-range-head--value-only">
        <span
          className={`cat-range-val${showVal ? ' visible' : ''}`}
          data-testid={`${testId}-val`}
          aria-hidden={!showVal}
        >
          {format(local)}
        </span>
      </div>
      <Slider
        value={[local]}
        min={0}
        max={max}
        step={step}
        onValueChange={(vals) => {
          setDragging(true);
          setLocal(vals[0]);
        }}
        onValueCommit={(vals) => {
          setDragging(false);
          onCommit(vals[0]);
        }}
        data-testid={`${testId}-slider`}
      />
    </div>
  );
}

/** A debounced two-thumb range slider: emits onCommit(lo, hi) on release. */
export function DualRangeControl({
  axis,
  min,
  max,
  ceiling,
  step,
  format,
  maxLabel,
  disabled,
  testId,
  onCommit,
}: {
  axis: string;
  min: number;
  max: number;
  ceiling: number;
  step: number;
  format: (v: number) => string;
  maxLabel: string;
  disabled?: boolean;
  testId: string;
  onCommit: (lo: number, hi: number) => void;
}) {
  const [local, setLocal] = useState<[number, number]>([min, max]);
  const [dragging, setDragging] = useState(false);
  const [hovering, setHovering] = useState(false);
  useEffect(() => {
    setLocal([min, max]);
  }, [min, max]);

  const isDefault = local[0] <= 0 && local[1] >= ceiling;
  const showVal = !isDefault || dragging || hovering;
  const valText = `${format(local[0])} – ${local[1] >= ceiling ? maxLabel : format(local[1])}`;

  return (
    <div
      className="cat-range-stack"
      data-testid={testId}
      onMouseEnter={() => setHovering(true)}
      onMouseLeave={() => setHovering(false)}
    >
      <div className="cat-range-head">
        <span className="cat-range-axis">{axis}</span>
        <span
          className={`cat-range-val${showVal ? ' visible' : ''}`}
          data-testid={`${testId}-val`}
          aria-hidden={!showVal}
        >
          {valText}
        </span>
      </div>
      <Slider
        value={local}
        min={0}
        max={ceiling}
        step={step}
        disabled={disabled}
        onValueChange={(vals) => {
          setDragging(true);
          setLocal([vals[0], vals[1]]);
        }}
        onValueCommit={(vals) => {
          setDragging(false);
          onCommit(vals[0], vals[1]);
        }}
        data-testid={`${testId}-slider`}
      />
    </div>
  );
}
