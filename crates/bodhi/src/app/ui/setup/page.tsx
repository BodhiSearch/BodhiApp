'use client';

import { BenefitCard } from '@/app/ui/setup/BenefitCard';
import { SetupProgress } from '@/app/ui/setup/SetupProgress';
import { containerVariants, itemVariants } from '@/app/ui/setup/types';
import { WelcomeCard } from '@/app/ui/setup/WelcomeCard';
import AppInitializer from '@/components/AppInitializer';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { useLocalStorage } from '@/hooks/useLocalStorage';
import { useAppInfo, useSetupApp } from '@/hooks/useQuery';
import {
  FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED,
  ROUTE_SETUP_DOWNLOAD_MODELS,
  ROUTE_SETUP_RESOURCE_ADMIN,
} from '@/lib/constants';
import { setupFormSchema, SetupFormData } from '@/schemas/objs';
import { zodResolver } from '@hookform/resolvers/zod';
import { motion } from 'framer-motion';
import { Loader2 } from 'lucide-react';
import { useRouter } from 'next/navigation';
import { useEffect } from 'react';
import { useForm } from 'react-hook-form';

const benefits = [
  {
    title: 'Complete Privacy',
    description: 'Your chat data stays on your device. No data sharing/transfer out of your device.',
    icon: '🔒',
  },
  {
    title: 'Always Free',
    description: 'Run unlimited AI inferences locally without usage fees or API costs.',
    icon: '💰',
  },
  {
    title: 'Full Control',
    description: 'Choose and run any compatible LLM model. Customize settings.',
    icon: '🚀',
  },
  {
    title: 'Local Performance',
    description: "Direct access to your hardware's capabilities without latency.",
    icon: '⚡',
  },
  {
    title: 'AI for Everyone',
    description:
      'Experience AI without technical complexity. Simple, intuitive UI unlocks the power of LLM for everyone.',
    icon: '🙏',
  },
  {
    title: 'Solid Foundation',
    description: 'Leverages open-source eco-system: HuggingFace, llama.cpp, etc.',
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
    <main className="min-h-screen bg-background p-4 md:p-8">
      <motion.div
        className="mx-auto max-w-4xl space-y-8"
        variants={containerVariants}
        initial="hidden"
        animate="visible"
      >
        <SetupProgress currentStep={1} totalSteps={4} />
        <WelcomeCard />

        <motion.div className="grid grid-cols-1 md:grid-cols-2 gap-4" variants={itemVariants}>
          {benefits.map((benefit) => (
            <BenefitCard key={benefit.title} {...benefit} />
          ))}
        </motion.div>

        <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.3 }}>
          <Card>
            <CardHeader>
              <CardTitle className="text-center">Setup Your Bodhi Server</CardTitle>
            </CardHeader>
            <CardContent>
              <Form {...form}>
                <form onSubmit={form.handleSubmit(handleSetup)} className="space-y-6" data-testid="setup-form">
                  <FormField
                    control={form.control}
                    name="name"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>Server Name *</FormLabel>
                        <FormControl>
                          <Input placeholder="My Bodhi Server" {...field} disabled={isLoading} />
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
            </CardContent>
          </Card>
        </motion.div>
      </motion.div>
    </main>
  );
}

export default function Setup() {
  return (
    <AppInitializer allowedStatus="setup" authenticated={false}>
      <SetupContent />
    </AppInitializer>
  );
}
