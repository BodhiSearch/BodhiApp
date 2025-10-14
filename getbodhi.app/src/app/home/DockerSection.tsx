'use client';

import { Container } from '@/components/ui/container';
import { Badge } from '@/components/ui/badge';
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { CopyableCodeBlock } from '@/app/home/CopyableCodeBlock';
import { Cpu, Zap, CheckCircle2 } from 'lucide-react';
import Link from 'next/link';
import { motion } from 'framer-motion';
import { getVariantMetadata } from '@/lib/docker-variants';
import { generateDockerRunCommand } from '@/lib/docker-commands';
import { useState, useEffect, useMemo } from 'react';
import { cn } from '@/lib/utils';
import { fadeInUp, variantTransition } from '@/app/home/animations';
import type { DockerData, ReleasesData } from '@/types/docker';

// Helper to get complete Tailwind classes for variant colors
const getVariantColorClasses = (color: string): { light: string; dark: string } => {
  const colorMap: Record<string, { light: string; dark: string }> = {
    blue: { light: 'bg-blue-100', dark: 'dark:bg-blue-900/20' },
    green: { light: 'bg-green-100', dark: 'dark:bg-green-900/20' },
    red: { light: 'bg-red-100', dark: 'dark:bg-red-900/20' },
    purple: { light: 'bg-purple-100', dark: 'dark:bg-purple-900/20' },
    indigo: { light: 'bg-indigo-100', dark: 'dark:bg-indigo-900/20' },
    orange: { light: 'bg-orange-100', dark: 'dark:bg-orange-900/20' },
    teal: { light: 'bg-teal-100', dark: 'dark:bg-teal-900/20' },
    gray: { light: 'bg-gray-100', dark: 'dark:bg-gray-900/20' },
  };
  return colorMap[color] || { light: 'bg-gray-100', dark: 'dark:bg-gray-900/20' };
};

export function DockerSection() {
  const [dockerData, setDockerData] = useState<DockerData | null>(null);
  const [selectedVariant, setSelectedVariant] = useState<string>('cpu');
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

  // Get all variants from dockerData (already sorted in releases.json)
  // Must be called before any early returns to follow Rules of Hooks
  const variantKeys = useMemo(
    () => (dockerData?.variants ? Object.keys(dockerData.variants) : []),
    [dockerData?.variants]
  );

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

  // Ensure selectedVariant is valid, fallback to first available
  const validSelectedVariant = dockerData.variants[selectedVariant] ? selectedVariant : variantKeys[0] || 'cpu';
  const currentVariant = dockerData.variants[validSelectedVariant];
  const currentMetadata = getVariantMetadata(validSelectedVariant);

  if (!currentVariant) {
    return (
      <section id="docker-section" className="py-8 bg-gradient-to-b from-white to-slate-50">
        <Container>
          <div className="text-center text-muted-foreground">No Docker variants available</div>
        </Container>
      </section>
    );
  }

  return (
    <section id="docker-section" className="py-8 bg-gradient-to-b from-white to-slate-50">
      <Container>
        <motion.div {...fadeInUp} className="max-w-4xl mx-auto">
          <div className="text-center mb-12">
            <h2 className="text-3xl font-bold mb-4">Deploy with Docker</h2>
          </div>

          {/* Variant Selector */}
          <div className="mb-8">
            <Select value={validSelectedVariant} onValueChange={setSelectedVariant}>
              <SelectTrigger className="w-full h-auto min-h-[4rem] text-base">
                <SelectValue>
                  <div className="flex items-center gap-3 py-2">
                    {/* Icon */}
                    <div
                      className={cn(
                        'p-2.5 rounded-lg',
                        getVariantColorClasses(currentMetadata.color).light,
                        getVariantColorClasses(currentMetadata.color).dark
                      )}
                    >
                      {currentMetadata.gpuVendor ? (
                        <Zap className="h-5 w-5 text-slate-700" />
                      ) : (
                        <Cpu className="h-5 w-5 text-slate-700" />
                      )}
                    </div>

                    {/* Variant Name */}
                    <div className="flex flex-col items-start flex-1">
                      <span className="font-semibold text-slate-900">{currentMetadata.displayName}</span>
                      <span className="text-sm text-muted-foreground">
                        {currentVariant.description || currentMetadata.description}
                      </span>
                    </div>

                    {/* Badges */}
                    <div className="flex flex-wrap gap-2">
                      {currentMetadata.recommended && (
                        <Badge variant="secondary" className="bg-green-100 text-green-700 border-green-200">
                          <CheckCircle2 className="h-3 w-3 mr-1" />
                          Recommended
                        </Badge>
                      )}
                      {currentVariant.gpu_type && (
                        <Badge variant="outline" className="border-slate-300">
                          {currentVariant.gpu_type}
                        </Badge>
                      )}
                    </div>
                  </div>
                </SelectValue>
              </SelectTrigger>

              <SelectContent>
                <SelectGroup>
                  <SelectLabel>Available Variants</SelectLabel>
                  {variantKeys.map((variantKey) => {
                    const variant = dockerData.variants[variantKey];
                    const metadata = getVariantMetadata(variantKey);

                    return (
                      <SelectItem key={variantKey} value={variantKey} className="h-auto min-h-[4rem] cursor-pointer">
                        <div className="flex items-center gap-3 py-2 w-full">
                          {/* Icon */}
                          <div className={cn('p-2 rounded-lg', getVariantColorClasses(metadata.color).light)}>
                            {metadata.gpuVendor ? (
                              <Zap className="h-4 w-4 text-slate-700" />
                            ) : (
                              <Cpu className="h-4 w-4 text-slate-700" />
                            )}
                          </div>

                          {/* Info */}
                          <div className="flex flex-col items-start flex-1 min-w-0">
                            <span className="font-medium text-slate-900">{metadata.displayName}</span>
                            <span className="text-xs text-muted-foreground truncate w-full">
                              {variant.description || metadata.description}
                            </span>
                          </div>

                          {/* Badges */}
                          <div className="flex flex-wrap gap-1 ml-2">
                            {metadata.recommended && (
                              <Badge
                                variant="secondary"
                                className="text-xs bg-green-100 text-green-700 border-green-200"
                              >
                                Recommended
                              </Badge>
                            )}
                            {variant.gpu_type && (
                              <Badge variant="outline" className="text-xs border-slate-300">
                                {variant.gpu_type}
                              </Badge>
                            )}
                          </div>
                        </div>
                      </SelectItem>
                    );
                  })}
                </SelectGroup>
              </SelectContent>
            </Select>
          </div>

          {/* Variant Details - Animated */}
          <motion.div key={validSelectedVariant} {...variantTransition} className="space-y-6">
            {/* Info Cards Grid */}
            <div className="grid md:grid-cols-2 gap-4">
              <div className="p-4 bg-slate-50 dark:bg-slate-900 rounded-lg border border-slate-200 dark:border-slate-800">
                <div className="text-sm font-medium text-muted-foreground mb-1">Platforms</div>
                <div className="text-base font-semibold text-slate-900 dark:text-slate-100">
                  {currentVariant.platforms.join(', ')}
                </div>
              </div>

              <div className="p-4 bg-slate-50 dark:bg-slate-900 rounded-lg border border-slate-200 dark:border-slate-800">
                <div className="text-sm font-medium text-muted-foreground mb-1">Image Tag</div>
                <div className="text-base font-mono font-semibold text-slate-900 dark:text-slate-100">
                  {currentVariant.latest_tag}
                </div>
              </div>

              {currentVariant.gpu_type && (
                <>
                  <div className="p-4 bg-slate-50 dark:bg-slate-900 rounded-lg border border-slate-200 dark:border-slate-800">
                    <div className="text-sm font-medium text-muted-foreground mb-1">GPU Vendor</div>
                    <div className="text-base font-semibold text-slate-900 dark:text-slate-100">
                      {currentVariant.gpu_type}
                    </div>
                  </div>

                  <div className="p-4 bg-slate-50 dark:bg-slate-900 rounded-lg border border-slate-200 dark:border-slate-800">
                    <div className="text-sm font-medium text-muted-foreground mb-1">Acceleration</div>
                    <div className="text-base font-semibold text-slate-900 dark:text-slate-100">
                      Hardware GPU Acceleration
                    </div>
                  </div>
                </>
              )}
            </div>

            {/* Description Card */}
            <div className="p-4 bg-blue-50 dark:bg-blue-950 rounded-lg border border-blue-200 dark:border-blue-800">
              <p className="text-sm text-blue-900 dark:text-blue-100">{currentMetadata.description}</p>
            </div>

            {/* Pull Command */}
            <div className="space-y-2">
              <h3 className="text-sm font-semibold text-muted-foreground">Pull Image</h3>
              <CopyableCodeBlock command={currentVariant.pull_command} language="bash" />
            </div>

            {/* Run Command */}
            <div className="space-y-2">
              <h3 className="text-sm font-semibold text-muted-foreground">Run Container</h3>
              <CopyableCodeBlock
                command={generateDockerRunCommand({
                  registry: dockerData.registry,
                  tag: currentVariant.latest_tag,
                  dockerFlags: currentVariant.docker_flags,
                })}
                language="bash"
              />
            </div>
          </motion.div>

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
