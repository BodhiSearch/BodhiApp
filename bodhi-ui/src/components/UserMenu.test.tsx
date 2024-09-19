'use client';

import { createWrapper } from '@/tests/wrapper';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import {
  afterAll,
  beforeAll,
  beforeEach,
  describe,
  expect,
  it,
  vi,
} from 'vitest';
import UserMenu from './UserMenu';

// Mock the Next.js router
const routerPushMock = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: routerPushMock,
  }),
}));

// Mock the useToast hook
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({
    toast: vi.fn(),
  }),
}));

const server = setupServer();

beforeAll(() => server.listen());
afterAll(() => server.close());
beforeEach(() => server.resetHandlers());

describe('UserMenu', () => {
  it('renders user email', () => {
    const wrapper = createWrapper();
    render(<UserMenu />, { wrapper });
    expect(screen.getByText('user@example.com')).toBeInTheDocument();
  });

  it('handles successful logout', async () => {
    const user = userEvent.setup();
    const wrapper = createWrapper();
    server.use(
      rest.post('*/api/ui/logout', (_, res, ctx) => {
        return res(ctx.set('location', '/ui/test/home'), ctx.status(302));
      })
    );

    render(<UserMenu />, { wrapper });

    const userEmail = screen.getByText('user@example.com');
    await user.click(userEmail);

    const logoutButton = await screen.findByText('Logout');
    await user.click(logoutButton);
    expect(routerPushMock).toHaveBeenCalledWith('/ui/test/home');
  });
});
