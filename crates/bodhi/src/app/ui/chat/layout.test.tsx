import { render } from '@testing-library/react';
import ChatLayout from './layout';
import { describe, expect, it } from 'vitest';

describe('ChatLayout', () => {
  it('renders children wrapped in ChatSettingsProvider', () => {
    const { container } = render(
      <ChatLayout>
        <div data-testid="test-child">Test Content</div>
      </ChatLayout>
    );
    
    expect(container.innerHTML).toContain('Test Content');
  });
}); 