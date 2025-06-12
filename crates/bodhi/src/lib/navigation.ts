// Next.js navigation utilities
import { useRouter as useNextRouter, usePathname as useNextPathname, useSearchParams as useNextSearchParams } from 'next/navigation';

// Next.js useRouter
export function useRouter() {
  const router = useNextRouter();

  return {
    push: (url: string) => router.push(url),
    replace: (url: string) => router.replace(url),
    back: () => router.back(),
    forward: () => router.forward(),
    refresh: () => router.refresh(),
  };
}

// Next.js usePathname
export function usePathname() {
  return useNextPathname();
}

// Next.js useSearchParams
export function useSearchParams() {
  const searchParams = useNextSearchParams();

  return {
    get: (key: string) => searchParams.get(key),
    getAll: (key: string) => searchParams.getAll(key),
    has: (key: string) => searchParams.has(key),
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
