'use client';

import { Button } from '@/components/ui/button';
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { useCreateToken } from '@/hooks/useApiTokens';
import { useUser } from '@/hooks/useUsers';
import { ApiTokenResponse } from '@bodhiapp/ts-client';
import { zodResolver } from '@hookform/resolvers/zod';
import { Loader2 } from 'lucide-react';
import { useMemo } from 'react';
import { useForm } from 'react-hook-form';
import { z } from 'zod';

export const createTokenSchema = z.object({
  name: z.string().optional(),
  scope: z.enum(['scope_token_user', 'scope_token_power_user']),
});

export type TokenFormData = z.infer<typeof createTokenSchema>;

interface TokenFormProps {
  onTokenCreated: (token: ApiTokenResponse) => void;
}

export function TokenForm({ onTokenCreated }: TokenFormProps) {
  const { showSuccess, showError } = useToastMessages();
  const { data: userInfo } = useUser();

  // Determine available scope options based on user role
  const scopeOptions = useMemo(() => {
    const userRole = userInfo?.auth_status === 'logged_in' ? userInfo.role : undefined;
    if (userRole === 'resource_user') {
      return [{ value: 'scope_token_user', label: 'User' }];
    }
    return [
      { value: 'scope_token_user', label: 'User' },
      { value: 'scope_token_power_user', label: 'PowerUser' },
    ];
  }, [userInfo]);

  const form = useForm<TokenFormData>({
    resolver: zodResolver(createTokenSchema),
    mode: 'onSubmit',
    defaultValues: {
      name: '',
      scope: 'scope_token_user',
    },
  });

  const { mutate: createToken, isLoading } = useCreateToken({
    onSuccess: (response) => {
      onTokenCreated(response);
      form.reset({
        name: '',
        scope: 'scope_token_user',
      });
      showSuccess('Success', 'API token successfully generated');
    },
    onError: (message) => {
      showError('Error', message);
    },
  });

  const onSubmit = (data: TokenFormData) => {
    createToken(data);
  };

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4" data-testid="token-form">
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
                  data-testid="token-name-input"
                  {...field}
                />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          control={form.control}
          name="scope"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Token Scope</FormLabel>
              <Select onValueChange={field.onChange} defaultValue={field.value} disabled={isLoading}>
                <FormControl>
                  <SelectTrigger data-testid="token-scope-select">
                    <SelectValue placeholder="Select scope" />
                  </SelectTrigger>
                </FormControl>
                <SelectContent>
                  {scopeOptions.map((option) => (
                    <SelectItem key={option.value} value={option.value} data-testid={`scope-option-${option.value}`}>
                      {option.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <FormMessage />
            </FormItem>
          )}
        />
        <Button type="submit" disabled={isLoading} data-testid="generate-token-button">
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
