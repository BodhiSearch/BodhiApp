'use client';

import { Button } from '@/components/ui/button';
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { useToast } from '@/hooks/use-toast';
import { TokenResponse, useCreateToken } from '@/hooks/useApiTokens';
import { zodResolver } from '@hookform/resolvers/zod';
import { Loader2 } from 'lucide-react';
import { useForm } from 'react-hook-form';
import { z } from 'zod';

export const createTokenSchema = z.object({
  name: z.string().optional(),
});

export type TokenFormData = z.infer<typeof createTokenSchema>;

interface TokenFormProps {
  onTokenCreated: (token: TokenResponse) => void;
}

export function TokenForm({ onTokenCreated }: TokenFormProps) {
  const { toast } = useToast();
  const form = useForm<TokenFormData>({
    resolver: zodResolver(createTokenSchema),
    mode: 'onSubmit',
    defaultValues: {
      name: '',
    },
  });

  const { mutate: createToken, isLoading } = useCreateToken({
    onSuccess: (response) => {
      onTokenCreated(response);
      form.reset();
      toast({
        title: 'Success',
        description: 'API token successfully generated',
        variant: 'default',
        duration: 5000,
      });
    },
    onError: (message) => {
      toast({
        title: 'Error',
        description: message,
        variant: 'destructive',
        duration: 5000,
      });
    },
  });

  const onSubmit = (data: TokenFormData) => {
    createToken(data);
  };

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
        <FormField
          control={form.control}
          name="name"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Token Name (Optional)</FormLabel>
              <FormControl>
                <Input
                  placeholder="Enter a name for your token"
                  disabled={isLoading}
                  {...field}
                />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <Button type="submit" disabled={isLoading}>
          {isLoading ? (
            <>
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              Generating...
            </>
          ) : (
            'Generate Token'
          )}
        </Button>
      </form>
    </Form>
  );
}
