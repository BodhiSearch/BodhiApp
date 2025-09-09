import { useEffect, useState } from 'react';

/**
 * Custom hook for generating responsive data-testid attributes based on current viewport
 *
 * Breakpoint strategy:
 * - Mobile (< 768px): 'm-' prefix
 * - Tablet (768px - 1023px): 'tab-' prefix
 * - Desktop (>= 1024px): no prefix
 *
 * @example
 * const getTestId = useResponsiveTestId();
 * <button data-testid={getTestId('send-button')} />
 * // Mobile: data-testid="m-send-button"
 * // Tablet: data-testid="tab-send-button"
 * // Desktop: data-testid="send-button"
 */
export function useResponsiveTestId() {
  const [viewport, setViewport] = useState<{ width: number; height: number } | null>(null);

  useEffect(() => {
    if (typeof window === 'undefined') return;

    const updateViewport = () => {
      setViewport({
        width: window.innerWidth,
        height: window.innerHeight,
      });
    };

    // Initial viewport detection
    updateViewport();

    // Listen for viewport changes
    window.addEventListener('resize', updateViewport);

    return () => {
      window.removeEventListener('resize', updateViewport);
    };
  }, []);

  const getTestId = (baseId: string): string => {
    // SSR safety - return base ID during server-side rendering
    if (!viewport || typeof window === 'undefined') {
      return baseId;
    }

    const { width } = viewport;

    // Mobile breakpoint (< 768px) - TailwindCSS md: breakpoint
    if (width < 768) {
      return `m-${baseId}`;
    }

    // Tablet breakpoint (768px - 1023px) - TailwindCSS md: to lg: range
    if (width < 1024) {
      return `tab-${baseId}`;
    }

    // Desktop breakpoint (>= 1024px) - TailwindCSS lg: and above
    return baseId;
  };

  return getTestId;
}

/**
 * Utility function for determining current viewport type
 * Useful for conditional logic in components
 */
export function useViewportType(): 'mobile' | 'tablet' | 'desktop' | null {
  const [viewportType, setViewportType] = useState<'mobile' | 'tablet' | 'desktop' | null>(null);

  useEffect(() => {
    if (typeof window === 'undefined') return;

    const updateViewportType = () => {
      const width = window.innerWidth;

      if (width < 768) {
        setViewportType('mobile');
      } else if (width < 1024) {
        setViewportType('tablet');
      } else {
        setViewportType('desktop');
      }
    };

    updateViewportType();
    window.addEventListener('resize', updateViewportType);

    return () => {
      window.removeEventListener('resize', updateViewportType);
    };
  }, []);

  return viewportType;
}

/**
 * Static utility function for generating responsive test IDs
 * Useful in contexts where hooks cannot be used
 *
 * @param baseId - Base test ID without prefix
 * @param viewportWidth - Current viewport width
 */
export function getResponsiveTestId(baseId: string, viewportWidth: number): string {
  if (viewportWidth < 768) {
    return `m-${baseId}`;
  }

  if (viewportWidth < 1024) {
    return `tab-${baseId}`;
  }

  return baseId;
}

/**
 * Viewport breakpoint constants
 * Aligned with TailwindCSS default breakpoints
 */
export const VIEWPORT_BREAKPOINTS = {
  mobile: {
    min: 0,
    max: 767,
  },
  tablet: {
    min: 768,
    max: 1023,
  },
  desktop: {
    min: 1024,
    max: Infinity,
  },
} as const;

export type ViewportType = keyof typeof VIEWPORT_BREAKPOINTS;
