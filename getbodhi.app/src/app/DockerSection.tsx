'use client';

import { Container } from '@/components/ui/container';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/tabs';
import { CommandSection } from '@/components/CommandSection';
import { Cpu, Zap, CheckCircle2 } from 'lucide-react';
import Link from 'next/link';
import { motion } from 'framer-motion';
import { getVariantMetadata } from '@/lib/docker-variants';
import { useState, useEffect } from 'react';

interface DockerVariant {
  image_tag: string;
  latest_tag: string;
  platforms: string[];
  pull_command: string;
  run_command?: string;
  gpu_type?: string;
  description?: string;
}

interface DockerData {
  version: string;
  tag: string;
  released_at: string;
  registry: string;
  variants: Record<string, DockerVariant>;
}

interface ReleasesData {
  docker: DockerData;
}

export function DockerSection() {
  const [dockerData, setDockerData] = useState<DockerData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetch('/releases.json')
      .then((res) => {
        if (!res.ok) throw new Error('Failed to load releases data');
        return res.json();
      })
      .then((data: ReleasesData) => {
        setDockerData(data.docker);
        setLoading(false);
      })
      .catch((err) => {
        console.error('Error loading Docker data:', err);
        setError('Failed to load Docker release information');
        setLoading(false);
      });
  }, []);

  if (loading) {
    return (
      <section id="docker-section" className="py-8 bg-gradient-to-b from-white to-slate-50">
        <Container>
          <div className="text-center text-muted-foreground">Loading Docker releases...</div>
        </Container>
      </section>
    );
  }

  if (error || !dockerData) {
    return (
      <section id="docker-section" className="py-8 bg-gradient-to-b from-white to-slate-50">
        <Container>
          <div className="text-center text-muted-foreground">{error || 'No Docker data available'}</div>
        </Container>
      </section>
    );
  }

  const variantKeys = Object.keys(dockerData.variants);
  const firstVariant = variantKeys[0] || 'cpu';

  return (
    <section id="docker-section" className="py-8 bg-gradient-to-b from-white to-slate-50">
      <Container>
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="max-w-4xl mx-auto"
        >
          <div className="text-center mb-12">
            <h2 className="text-3xl font-bold mb-4">Deploy with Docker</h2>
            <p className="text-muted-foreground max-w-2xl mx-auto">
              Pull official Docker images with hardware-specific optimizations. Available for CPU and GPU-accelerated
              inference.
            </p>
          </div>

          <Tabs defaultValue={firstVariant} className="w-full">
            <TabsList className="grid w-full grid-cols-3 mb-8">
              {variantKeys.map((variantKey) => {
                const variant = dockerData.variants[variantKey];
                const metadata = getVariantMetadata(variantKey);

                return (
                  <TabsTrigger key={variantKey} value={variantKey} className="flex items-center gap-2">
                    {metadata.gpuVendor ? <Zap className="h-4 w-4" /> : <Cpu className="h-4 w-4" />}
                    <span>{metadata.displayName}</span>
                    {metadata.recommended && (
                      <Badge variant="secondary" className="ml-1 bg-green-100 text-green-700 text-xs">
                        <CheckCircle2 className="h-3 w-3 mr-1" />
                        Recommended
                      </Badge>
                    )}
                    {variant.gpu_type && !metadata.recommended && (
                      <Badge variant="secondary" className="ml-1 text-xs">
                        {variant.gpu_type}
                      </Badge>
                    )}
                  </TabsTrigger>
                );
              })}
            </TabsList>

            {variantKeys.map((variantKey) => {
              const variant = dockerData.variants[variantKey];
              const metadata = getVariantMetadata(variantKey);

              return (
                <TabsContent key={variantKey} value={variantKey} className="space-y-6">
                  {/* Variant Info */}
                  <div className="bg-slate-50 dark:bg-slate-900 rounded-lg p-4 border border-slate-200 dark:border-slate-800">
                    <p className="text-sm text-slate-700 dark:text-slate-300 mb-2">{metadata.description}</p>
                    <p className="text-xs text-muted-foreground">
                      <span className="font-medium">Platforms:</span> {variant.platforms.join(', ')}
                    </p>
                  </div>

                  {/* Pull Command */}
                  <CommandSection title="Pull Image" command={variant.pull_command} language="bash" />

                  {/* Run Command */}
                  {variant.run_command && (
                    <CommandSection title="Run Container" command={variant.run_command} language="bash" />
                  )}
                </TabsContent>
              );
            })}
          </Tabs>

          <div className="mt-12 p-6 bg-blue-50 dark:bg-blue-950 rounded-lg border border-blue-200 dark:border-blue-800">
            <p className="text-sm text-blue-900 dark:text-blue-100">
              <strong>Important:</strong> Running Docker containers requires volume mounting and specific environment
              variables. See our{' '}
              <Link
                href="/docs/deployment/docker"
                className="text-blue-600 dark:text-blue-400 hover:underline font-semibold"
              >
                Docker deployment documentation
              </Link>{' '}
              for complete setup instructions.
            </p>
          </div>
        </motion.div>
      </Container>
    </section>
  );
}
