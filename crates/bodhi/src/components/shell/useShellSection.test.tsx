import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { useShellSection } from './useShellSection';

// useShellSection reads route staticData via useMatches(); mock it so we control the match chain.
let mockMatches: Array<{ staticData?: { section?: string; subPage?: string | null } }> = [];
vi.mock('@tanstack/react-router', () => ({
  useMatches: () => mockMatches,
}));

function Probe() {
  const { section, subPage } = useShellSection();
  return (
    <div>
      <span data-testid="section">{section || 'none'}</span>
      <span data-testid="subPage">{subPage ?? 'null'}</span>
    </div>
  );
}

function renderWith(matches: typeof mockMatches) {
  mockMatches = matches;
  render(<Probe />);
}

describe('useShellSection', () => {
  it('reads section + subPage from the deepest match that declares a section', () => {
    renderWith([
      { staticData: {} }, // root
      { staticData: { section: 'models', subPage: 'my-models' } },
    ]);
    expect(screen.getByTestId('section')).toHaveTextContent('models');
    expect(screen.getByTestId('subPage')).toHaveTextContent('my-models');
  });

  it('treats a section without subPage as no sub-page highlight', () => {
    renderWith([{ staticData: { section: 'chat' } }]);
    expect(screen.getByTestId('section')).toHaveTextContent('chat');
    expect(screen.getByTestId('subPage')).toHaveTextContent('null');
  });

  it('prefers the deepest declaring match (child refines parent)', () => {
    renderWith([
      { staticData: { section: 'models' } },
      { staticData: { section: 'models', subPage: 'explore-local' } },
    ]);
    expect(screen.getByTestId('subPage')).toHaveTextContent('explore-local');
  });

  it('returns no highlight when no match declares a section (bare/undeclared routes)', () => {
    renderWith([{ staticData: {} }, { staticData: undefined }]);
    expect(screen.getByTestId('section')).toHaveTextContent('none');
    expect(screen.getByTestId('subPage')).toHaveTextContent('null');
  });
});
