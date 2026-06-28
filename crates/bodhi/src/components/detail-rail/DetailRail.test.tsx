import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { DetailRail, DetailRailBody, DetailRailRow, DetailRailRows, DetailRailSection } from './DetailRail';

describe('DetailRail primitives', () => {
  it('DetailRail renders the dp-panel wrapper with className + testid passthrough', () => {
    render(
      <DetailRail className="models-screen-rail" testId="cat-detail-x">
        <span>body</span>
      </DetailRail>
    );
    const panel = screen.getByTestId('cat-detail-x');
    expect(panel).toHaveClass('dp-panel');
    expect(panel).toHaveClass('models-screen-rail');
  });

  it('DetailRailBody / DetailRailSection render the dp-body / dp-section + label', () => {
    render(
      <DetailRailBody>
        <DetailRailSection label="Specs">
          <span data-testid="content">x</span>
        </DetailRailSection>
      </DetailRailBody>
    );
    expect(document.querySelector('.dp-body')).toBeInTheDocument();
    expect(document.querySelector('.dp-section')).toBeInTheDocument();
    expect(screen.getByText('Specs')).toHaveClass('dp-sec-lbl');
    expect(screen.getByTestId('content')).toBeInTheDocument();
  });

  it('DetailRailSection accepts a ReactNode label', () => {
    render(
      <DetailRailSection label={<span data-testid="rich">Served by (2)</span>}>
        <span>x</span>
      </DetailRailSection>
    );
    expect(screen.getByTestId('rich')).toBeInTheDocument();
  });

  it('DetailRailRows wraps children in dp-rows with a testid', () => {
    render(
      <DetailRailRows testId="meta">
        <span>x</span>
      </DetailRailRows>
    );
    const rows = screen.getByTestId('meta');
    expect(rows).toHaveClass('dp-rows');
  });

  it('DetailRailRow renders key + value, value mono by default', () => {
    render(<DetailRailRow k="Context" v="200K" />);
    expect(screen.getByText('Context')).toHaveClass('dp-row-k');
    const v = screen.getByText('200K');
    expect(v).toHaveClass('dp-row-v');
    expect(v).toHaveClass('mono');
  });

  it('DetailRailRow omits the mono class when mono=false', () => {
    render(<DetailRailRow k="Created" v="Jan 2, 2024" mono={false} />);
    expect(screen.getByText('Jan 2, 2024')).not.toHaveClass('mono');
  });

  it('DetailRailRow renders nothing when the value is empty/nullish', () => {
    const { container } = render(<DetailRailRow k="Knowledge" v={undefined} />);
    expect(container.querySelector('.dp-row')).toBeNull();
  });

  it('DetailRailRow renders an empty-string value as absent', () => {
    const { container } = render(<DetailRailRow k="X" v="" />);
    expect(container.querySelector('.dp-row')).toBeNull();
  });
});
