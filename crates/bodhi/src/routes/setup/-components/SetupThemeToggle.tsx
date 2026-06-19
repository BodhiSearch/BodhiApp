import { useEffect, useState } from 'react';

import { Moon, Sun } from 'lucide-react';

import { useTheme } from '@/components/ThemeProvider';

/** Fixed top-right light/dark toggle for the bare setup wizard.
 *  Theme is owned by ThemeProvider (it toggles the `.dark` class on <html>);
 *  we read that class to pick the icon, and flip to the opposite explicit theme. */
export function SetupThemeToggle() {
  const { setTheme } = useTheme();
  const [isDark, setIsDark] = useState(false);

  useEffect(() => {
    const root = window.document.documentElement;
    const sync = () => setIsDark(root.classList.contains('dark'));
    sync();
    const observer = new MutationObserver(sync);
    observer.observe(root, { attributes: true, attributeFilter: ['class'] });
    return () => observer.disconnect();
  }, []);

  return (
    <button
      type="button"
      data-testid="setup-theme-toggle"
      onClick={() => setTheme(isDark ? 'light' : 'dark')}
      aria-label={isDark ? 'Switch to light theme' : 'Switch to dark theme'}
      title={isDark ? 'Switch to light theme' : 'Switch to dark theme'}
      className="fixed right-4 top-4 z-50 flex h-10 w-10 items-center justify-center rounded-full border border-border bg-card text-foreground shadow-sm transition-colors hover:border-primary/50 hover:text-[hsl(var(--primary-hover))]"
    >
      {isDark ? <Sun className="h-[18px] w-[18px]" /> : <Moon className="h-[18px] w-[18px]" />}
    </button>
  );
}
