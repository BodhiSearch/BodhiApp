import { useListKeyNav } from '@/components/shell';
import { fireEvent, render } from '@testing-library/react';
import { useState } from 'react';
import { describe, expect, it, vi } from 'vitest';

/**
 * Test harness: a minimal master-detail list whose rows mirror the real screens —
 * `.l-scroll` container, `.l-listrow` rows that carry `.active` when selected, each with a
 * stretched `.l-rowlink` anchor that runs the row's select handler (exactly like LinkRow).
 */
function ListHarness({ count = 3, onSelect = (_: number) => {} }: { count?: number; onSelect?: (i: number) => void }) {
  const [sel, setSel] = useState<number | null>(null);
  useListKeyNav();
  const pick = (i: number) => {
    setSel(i);
    onSelect(i);
  };
  return (
    <div className="l-scroll">
      <div className="l-listview">
        {Array.from({ length: count }).map((_, i) => (
          <div
            key={i}
            className={`l-listrow${sel === i ? ' active' : ''}`}
            data-testid={`row-${i}`}
            onClick={() => pick(i)}
          >
            {/* eslint-disable-next-line jsx-a11y/anchor-is-valid */}
            <a
              className="l-rowlink"
              href="#"
              data-testid={`link-${i}`}
              onClick={(e) => {
                e.preventDefault();
                e.stopPropagation();
                pick(i);
              }}
            />
            <span>Row {i}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

const press = (key: string) => fireEvent.keyDown(document, { key });

describe('useListKeyNav', () => {
  it('ArrowDown from no selection selects the first row', () => {
    const onSelect = vi.fn();
    render(<ListHarness onSelect={onSelect} />);
    press('ArrowDown');
    expect(onSelect).toHaveBeenLastCalledWith(0);
  });

  it('ArrowUp from no selection selects the last row', () => {
    const onSelect = vi.fn();
    render(<ListHarness count={3} onSelect={onSelect} />);
    press('ArrowUp');
    expect(onSelect).toHaveBeenLastCalledWith(2);
  });

  it('ArrowDown moves selection down one row and is eager (opens detail)', () => {
    const onSelect = vi.fn();
    render(<ListHarness onSelect={onSelect} />);
    press('ArrowDown'); // -> 0
    press('ArrowDown'); // -> 1
    expect(onSelect).toHaveBeenLastCalledWith(1);
  });

  it('stops at the bottom edge (no wrap)', () => {
    const onSelect = vi.fn();
    render(<ListHarness count={3} onSelect={onSelect} />);
    press('End'); // -> 2 (last)
    onSelect.mockClear();
    press('ArrowDown'); // already last → no change, no extra select
    expect(onSelect).not.toHaveBeenCalled();
  });

  it('stops at the top edge (no wrap)', () => {
    const onSelect = vi.fn();
    render(<ListHarness count={3} onSelect={onSelect} />);
    press('Home'); // -> 0 (first)
    onSelect.mockClear();
    press('ArrowUp'); // already first → no change
    expect(onSelect).not.toHaveBeenCalled();
  });

  it('Home/End jump to first/last', () => {
    const onSelect = vi.fn();
    render(<ListHarness count={4} onSelect={onSelect} />);
    press('End');
    expect(onSelect).toHaveBeenLastCalledWith(3);
    press('Home');
    expect(onSelect).toHaveBeenLastCalledWith(0);
  });

  it('ignores arrow keys while focus is in a text input', () => {
    const onSelect = vi.fn();
    render(
      <>
        <input data-testid="search" />
        <ListHarness onSelect={onSelect} />
      </>
    );
    (document.querySelector('[data-testid="search"]') as HTMLInputElement).focus();
    press('ArrowDown');
    expect(onSelect).not.toHaveBeenCalled();
  });

  it('ignores modified arrow keys (Cmd/Ctrl/Alt)', () => {
    const onSelect = vi.fn();
    render(<ListHarness onSelect={onSelect} />);
    fireEvent.keyDown(document, { key: 'ArrowDown', metaKey: true });
    expect(onSelect).not.toHaveBeenCalled();
  });
});
