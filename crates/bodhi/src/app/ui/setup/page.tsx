'use client';

import { useEffect } from 'react';

import { zodResolver } from '@hookform/resolvers/zod';
import { motion } from 'framer-motion';
import { Loader2 } from 'lucide-react';
import { useRouter } from 'next/navigation';
import { useForm } from 'react-hook-form';

import { BenefitCard } from '@/app/ui/setup/BenefitCard';
import { SetupContainer, SetupCard } from '@/app/ui/setup/components';
import { itemVariants } from '@/app/ui/setup/types';
import { WelcomeCard } from '@/app/ui/setup/WelcomeCard';
import AppInitializer from '@/components/AppInitializer';
import { Button } from '@/components/ui/button';
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { useSetupApp } from '@/hooks/useInfo';
import { useLocalStorage } from '@/hooks/useLocalStorage';
import {
  FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED,
  ROUTE_SETUP_DOWNLOAD_MODELS,
  ROUTE_SETUP_RESOURCE_ADMIN,
} from '@/lib/constants';
import { setupFormSchema, SetupFormData } from '@/schemas/objs';

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
  const router = useRouter();
  const { showError } = useToastMessages();
  const [, setHasShownModelsPage] = useLocalStorage(FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED, false);

  useEffect(() => {
    setHasShownModelsPage(false);
  }, [setHasShownModelsPage]);

  const { mutate: setup, isLoading } = useSetupApp({
    onSuccess: (appInfo) => {
      if (appInfo.status === 'resource-admin') {
        router.push(ROUTE_SETUP_RESOURCE_ADMIN);
      } else {
        router.push(ROUTE_SETUP_DOWNLOAD_MODELS);
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

      <motion.div className="grid grid-cols-1 md:grid-cols-2 gap-4" variants={itemVariants} data-testid="benefits-grid">
        {benefits.map((benefit) => (
          <BenefitCard key={benefit.title} {...benefit} />
        ))}
      </motion.div>

      <SetupCard title="Setup Your Bodhi Server">
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

            <Button type="submit" className="w-full" size="lg" disabled={isLoading}>
              {isLoading ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Setting up...
                </>
              ) : (
                'Setup Bodhi Server →'
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
