import { zodResolver } from '@hookform/resolvers/zod';
import { useNavigate } from '@tanstack/react-router';
import { createFileRoute } from '@tanstack/react-router';
import { motion } from 'framer-motion';
import { ArrowRight, Loader2 } from 'lucide-react';
import { useForm } from 'react-hook-form';

import AppInitializer from '@/components/AppInitializer';
import { Button } from '@/components/ui/button';
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import { useSetupApp } from '@/hooks/info';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { ROUTE_SETUP_DOWNLOAD_MODELS, ROUTE_SETUP_RESOURCE_ADMIN } from '@/lib/constants';
import { setupFormSchema, SetupFormData } from '@/schemas/objs';

import { SetupContainer, SetupCard } from './-components';
import { BenefitCard } from './-components/BenefitCard';
import { WelcomeCard } from './-components/WelcomeCard';
import { itemVariants } from './-shared/types';

export const Route = createFileRoute('/setup/')({
  component: Setup,
});

const benefits = [
  {
    title: 'Complete Privacy',
    description:
      'Your data stays under your control. Choose local models for maximum privacy or connect to trusted APIs.',
    icon: '🔒',
  },
  {
    title: 'Cost Freedom',
    description: 'Run unlimited local AI without fees. Use your own API keys for cloud models. You control the costs.',
    icon: '💰',
  },
  {
    title: 'Hybrid Flexibility',
    description: 'Run local models on your hardware or connect to OpenAI, Anthropic, and other API providers.',
    icon: '🚀',
    isNew: true,
  },
  {
    title: 'Multi-User Ready',
    description: 'Built for teams and families. Role-based access with admin controls and user management.',
    icon: '👥',
    isNew: true,
  },
  {
    title: 'Browser AI Revolution',
    description:
      'Enable AI on any website through our browser extension. Publishers save costs, users get enhanced experiences.',
    icon: '🌐',
    isNew: true,
  },
  {
    title: 'Open Ecosystem',
    description: 'Powered by llama.cpp, compatible with HuggingFace models, OpenAI APIs, and more.',
    icon: '🌟',
  },
];

function SetupContent() {
  const navigate = useNavigate();
  const { showError } = useToastMessages();

  const { mutate: setup, isPending: isLoading } = useSetupApp({
    onSuccess: (appInfo) => {
      if (appInfo.status === 'resource_admin') {
        navigate({ to: ROUTE_SETUP_RESOURCE_ADMIN });
      } else {
        navigate({ to: ROUTE_SETUP_DOWNLOAD_MODELS });
      }
    },
    onError: (error) => {
      showError('Error', error);
    },
  });

  const form = useForm<SetupFormData>({
    resolver: zodResolver(setupFormSchema),
    mode: 'onSubmit',
    defaultValues: {
      name: '',
      description: '',
    },
  });

  const handleSetup = (data: SetupFormData) => {
    setup({
      name: data.name,
      description: data.description || undefined,
    });
  };

  return (
    <SetupContainer data-testid="setup-welcome-page">
      <WelcomeCard />

      <motion.div
        className="mb-6 grid grid-cols-1 gap-3.5 md:grid-cols-2"
        variants={itemVariants}
        data-testid="benefits-grid"
      >
        {benefits.map((benefit) => (
          <BenefitCard key={benefit.title} {...benefit} />
        ))}
      </motion.div>

      <SetupCard
        title="Set up your Bodhi server"
        description="Give this instance a name so you can recognize it later."
      >
        <Form {...form}>
          <form onSubmit={form.handleSubmit(handleSetup)} className="space-y-6" data-testid="setup-form">
            <FormField
              control={form.control}
              name="name"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Server Name *</FormLabel>
                  <FormControl>
                    <Input
                      placeholder="John Doe's Bodhi App Server"
                      {...field}
                      disabled={isLoading}
                      data-testid="server-name-input"
                    />
                  </FormControl>
                  <FormMessage />
                  <p className="text-sm text-muted-foreground">
                    Minimum 10 characters. This will identify your server instance.
                  </p>
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="description"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Description (Optional)</FormLabel>
                  <FormControl>
                    <Textarea
                      placeholder="A description of your Bodhi server instance..."
                      rows={3}
                      {...field}
                      disabled={isLoading}
                      data-testid="description-input"
                    />
                  </FormControl>
                  <FormMessage />
                  <p className="text-sm text-muted-foreground">
                    Optional description to help you identify this server.
                  </p>
                </FormItem>
              )}
            />

            <Button type="submit" className="w-full gap-2" size="lg" disabled={isLoading} data-testid="setup-submit">
              {isLoading ? (
                <>
                  <Loader2 className="h-4 w-4 animate-spin" />
                  Setting up...
                </>
              ) : (
                <>
                  Set up Bodhi Server
                  <ArrowRight className="h-4 w-4" />
                </>
              )}
            </Button>
          </form>
        </Form>
      </SetupCard>
    </SetupContainer>
  );
}

export default function Setup() {
  return (
    <AppInitializer allowedStatus="setup" authenticated={false}>
      <SetupContent />
    </AppInitializer>
  );
}
