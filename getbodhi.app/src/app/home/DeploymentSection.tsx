'use client';

import { memo } from 'react';
import { Monitor, Container as ContainerIcon, Cloud, Zap, HardDrive, Server } from 'lucide-react';
import { Container } from '@/components/ui/container';
import { FeatureCard } from '@/app/home/FeatureCard';
import { SectionHeader } from '@/app/home/SectionHeader';
import { SECTION_GRADIENTS, STYLES } from '@/lib/constants';
import { cn } from '@/lib/utils';

const deploymentOptions = [
  {
    icon: <Monitor className={cn(STYLES.iconSize, STYLES.iconColor)} />,
    title: 'Multi-Platform Desktop',
    description: 'Native desktop apps for Windows, macOS (Intel/ARM), and Linux with Tauri.',
    href: '/docs/install/',
  },
  {
    icon: <ContainerIcon className={cn(STYLES.iconSize, STYLES.iconColor)} />,
    title: 'Docker Variants',
    description: 'CPU (AMD64/ARM64), CUDA, ROCm, and Vulkan optimized images for every hardware.',
    href: '/docs/deployment/docker/',
  },
  {
    icon: <Cloud className={cn(STYLES.iconSize, STYLES.iconColor)} />,
    title: 'Cloud Ready',
    description: 'RunPod auto-configuration and support for any Docker-compatible cloud platform.',
    href: '/docs/deployment/docker/',
  },
  {
    icon: <Zap className={cn(STYLES.iconSize, STYLES.iconColor)} />,
    title: 'GPU Acceleration',
    description: '8-12x speedup with CUDA/ROCm GPU support for NVIDIA and AMD graphics cards.',
    href: '/docs/deployment/docker/',
  },
  {
    icon: <HardDrive className={cn(STYLES.iconSize, STYLES.iconColor)} />,
    title: 'Volume Management',
    description: 'Persistent storage with backup/restore strategies and migration support.',
    href: '/docs/deployment/docker/',
  },
  {
    icon: <Server className={cn(STYLES.iconSize, STYLES.iconColor)} />,
    title: 'Production Ready',
    description: 'Health checks, monitoring, log management, and automatic database migrations.',
    href: '/docs/deployment/docker/',
  },
];

function DeploymentSectionComponent() {
  return (
    <section className={cn(SECTION_GRADIENTS.whiteToViolet, 'py-12 sm:py-20')}>
      <Container>
        <SectionHeader
          title="Flexible Deployment Options"
          description="Deploy anywhere - from desktop to cloud, with hardware-optimized variants for maximum performance"
        />

        <div className={STYLES.featureGrid}>
          {deploymentOptions.map((option, index) => (
            <FeatureCard key={option.title} {...option} index={index} />
          ))}
        </div>
      </Container>
    </section>
  );
}

export const DeploymentSection = memo(DeploymentSectionComponent);
