import {
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
  Outlet,
  RouterProvider,
} from '@tanstack/react-router';

/**
 * Mount a single screen behind a real (in-memory) TanStack Router so components that read
 * `useSearch`/`useNavigate` (or `getRouteApi(path)`) work and browser-history navigation
 * (`router.history.back/forward`) drives them — no mocking of the router needed.
 *
 * Returns the `router` so tests can drive history and assert the URL:
 *   const { router } = renderWithRoute({ path: '/models/explore/api/', validateSearch, Screen });
 *   router.history.back();
 *   expect(router.state.location.search).toMatchObject({ ... });
 */
export function makeRouteRouter({
  path,
  validateSearch,
  Screen,
  initialEntries,
}: {
  path: string;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  validateSearch?: (s: Record<string, unknown>) => any;
  Screen: React.ComponentType;
  initialEntries?: string[];
}) {
  const rootRoute = createRootRoute({ component: () => <Outlet /> });
  const screenRoute = createRoute({
    getParentRoute: () => rootRoute,
    // Test-only infra: the route path/component are dynamic, so the precise router generics don't apply.
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    path: path as any,
    validateSearch,
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    component: Screen as any,
  });
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const routeTree = rootRoute.addChildren([screenRoute as any]);
  const history = createMemoryHistory({ initialEntries: initialEntries ?? [path] });
  // Mirror production router config (main.tsx) so <Link> hrefs match what ships.
  return createRouter({ routeTree, history, trailingSlash: 'always' });
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function RouteHarness({ router }: { router: any }) {
  return <RouterProvider router={router} />;
}
