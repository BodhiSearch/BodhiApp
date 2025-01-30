import { Alert, AlertDescription } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { useLocalStorage } from '@/hooks/useLocalStorage';
import { X } from 'lucide-react';

interface UserOnboardingProps {
  storageKey: string;
  children: React.ReactNode;
}

export function UserOnboarding({ storageKey, children }: UserOnboardingProps) {
  const [hasDismissed, setHasDismissed] = useLocalStorage(storageKey, false);

  if (hasDismissed) {
    return null;
  }

  return (
    <Alert className="mb-4">
      <AlertDescription className="flex items-center justify-between gap-4">
        <span>{children}</span>
        <Button
          variant="ghost"
          size="sm"
          className="h-8 w-8 p-0 shrink-0"
          onClick={() => setHasDismissed(true)}
          title="Dismiss"
        >
          <X className="h-4 w-4" />
          <span className="sr-only">Dismiss</span>
        </Button>
      </AlertDescription>
    </Alert>
  );
}
