import { expect, vi, test } from 'vitest';
import { redirect } from 'next/navigation';
import Home from '../page';

vi.mock('next/navigation', () => ({
  redirect: vi.fn(),
}));

test('Home redirects to /ui', () => {
  Home();
  expect(redirect).toHaveBeenCalledWith('/ui');
});
