import {
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
  Outlet,
  RouterProvider,
} from '@tanstack/react-router';

// Real in-memory TanStack Router so useSearch/useNavigate and router.history.back/forward work.
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
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    path: path as any,
    validateSearch,
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    component: Screen as any,
  });
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const routeTree = rootRoute.addChildren([screenRoute as any]);
  const history = createMemoryHistory({ initialEntries: initialEntries ?? [path] });
  return createRouter({ routeTree, history, trailingSlash: 'always' });
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function RouteHarness({ router }: { router: any }) {
  return <RouterProvider router={router} />;
}
