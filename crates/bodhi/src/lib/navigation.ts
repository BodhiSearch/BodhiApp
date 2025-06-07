// React Router equivalents for Next.js navigation
import {
  useNavigate,
  useLocation,
  useSearchParams as useRouterSearchParams,
} from 'react-router-dom';

// Equivalent to Next.js useRouter
export function useRouter() {
  const navigate = useNavigate();
  const location = useLocation();

  return {
    push: (url: string) => navigate(url),
    replace: (url: string) => navigate(url, { replace: true }),
    back: () => navigate(-1),
    forward: () => navigate(1),
    refresh: () => window.location.reload(),
    pathname: location.pathname,
    query: Object.fromEntries(new URLSearchParams(location.search)),
    asPath: location.pathname + location.search,
  };
}

// Equivalent to Next.js usePathname
export function usePathname() {
  const location = useLocation();
  return location.pathname;
}

// Equivalent to Next.js useSearchParams
export function useSearchParams() {
  const [searchParams, setSearchParams] = useRouterSearchParams();

  return {
    get: (key: string) => searchParams.get(key),
    getAll: (key: string) => searchParams.getAll(key),
    has: (key: string) => searchParams.has(key),
    set: (key: string, value: string) => {
      const newParams = new URLSearchParams(searchParams);
      newParams.set(key, value);
      setSearchParams(newParams);
    },
    delete: (key: string) => {
      const newParams = new URLSearchParams(searchParams);
      newParams.delete(key);
      setSearchParams(newParams);
    },
    toString: () => searchParams.toString(),
  };
}

// Equivalent to Next.js redirect
export function redirect(url: string) {
  window.location.href = url;
}

// Equivalent to Next.js notFound
export function notFound() {
  throw new Error('Not Found');
}
