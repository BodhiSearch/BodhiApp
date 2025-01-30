import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { AlertCircle } from 'lucide-react';

interface ErrorPageProps {
  title?: string;
  message: string;
}

export function ErrorPage({ title = 'Error', message }: ErrorPageProps) {
  return (
    <div className="flex flex-1 items-center justify-center p-6">
      <Alert variant="destructive" className="max-w-md">
        <AlertCircle className="h-4 w-4" />
        <AlertTitle>{title}</AlertTitle>
        <AlertDescription>{message}</AlertDescription>
      </Alert>
    </div>
  );
}
