
import { AppBreadcrumb } from '@/components/navigation/AppBreadcrumb';
import { AppNavigation } from '@/components/navigation/AppNavigation';
import { usePathname } from '@/lib/navigation';
import { siGithub } from 'simple-icons';

// Using the same URL from setup/complete/page.tsx
const GITHUB_REPO_URL = 'https://github.com/BodhiSearch/BodhiApp';

function SimpleIcon({
  icon,
  className,
}: {
  icon: { path: string };
  className?: string;
}) {
  return (
    <svg
      role="img"
      viewBox="0 0 24 24"
      className={className}
      fill="currentColor"
    >
      <path d={icon.path} />
    </svg>
  );
}

export function AppHeader() {
  const pathname = usePathname();
  const shouldRenderHeader = !pathname?.startsWith('/ui/setup/');

  const handleGitHubClick = () => {
    window.open(GITHUB_REPO_URL, '_blank');
  };

  if (!shouldRenderHeader) {
    return null;
  }

  return (
    <header
      className="sticky top-0 z-50 h-16 border-b bg-header-elevated/90 backdrop-blur-sm"
      data-testid="app-header"
    >
      <div className="flex h-full items-center justify-between px-4">
        <div className="flex h-full items-center">
          <AppNavigation />
          <AppBreadcrumb />
        </div>
        <button
          onClick={handleGitHubClick}
          className="flex items-center gap-2 rounded-lg p-2 text-sm text-muted-foreground transition-colors hover:bg-zinc-100 dark:hover:bg-zinc-800"
          aria-label="Star on GitHub - Support the project, track updates, and contribute to development"
        >
          <SimpleIcon icon={siGithub} className="h-5 w-5" />
        </button>
      </div>
    </header>
  );
}
