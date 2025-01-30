import { Loader2 } from 'lucide-react';

interface LoadingProps {
  message?: string;
}

export function Loading({ message = 'Loading...' }: LoadingProps) {
  return (
    <div className="flex flex-1 items-center justify-center">
      <div className="flex flex-col items-center gap-4">
        <Loader2 className="h-12 w-12 animate-spin text-muted-foreground" />
        <p className="text-muted-foreground">{message}</p>
      </div>
    </div>
  );
}
