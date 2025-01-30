import { useToast } from '@/hooks/use-toast';

export function useToastMessages() {
  const { toast } = useToast();

  return {
    showSuccess: (title: string, description: string) => {
      toast({
        title,
        description,
        duration: 1000,
      });
    },
    showError: (title: string, description: string) => {
      toast({
        title,
        description,
        variant: 'destructive',
        duration: 5000,
      });
    },
  };
}
