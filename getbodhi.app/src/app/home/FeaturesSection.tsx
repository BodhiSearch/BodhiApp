'use client';

import { memo } from 'react';
import {
  Cpu,
  Database,
  Lock,
  MessageSquare,
  Terminal,
  Zap,
  Cloud,
  Activity,
  Download,
  Settings,
  Gauge,
  Radio,
} from 'lucide-react';
import { Container } from '@/components/ui/container';
import { FeatureCard } from '@/app/home/FeatureCard';
import { SectionHeader } from '@/app/home/SectionHeader';
import { STYLES } from '@/lib/constants';
import { cn } from '@/lib/utils';

const features = {
  userFeatures: [
    {
      icon: <MessageSquare className={cn(STYLES.iconSize, STYLES.iconColor)} />,
      title: 'Built-in Chat UI',
      description: 'Intuitive chat interface with full markdown and settings.',
      href: '/docs/features/chat-ui/',
    },
    {
      icon: <Lock className={cn(STYLES.iconSize, STYLES.iconColor)} />,
      title: 'Privacy First',
      description: 'Run everything locally on your machine with complete data control.',
      href: '/docs/intro/',
    },
    {
      icon: <Database className={cn(STYLES.iconSize, STYLES.iconColor)} />,
      title: 'Model Management',
      description: 'One-click downloads from HuggingFace with real-time progress tracking.',
      href: '/docs/features/model-downloads/',
    },
    {
      icon: <Cloud className={cn(STYLES.iconSize, STYLES.iconColor)} />,
      title: 'Hybrid AI Architecture',
      description: 'Use local GGUF models alongside API providers (OpenAI, Anthropic, Groq) in one unified interface.',
      href: '/docs/features/api-models/',
    },
    {
      icon: <Radio className={cn(STYLES.iconSize, STYLES.iconColor)} />,
      title: 'Real-time Streaming',
      description: 'Server-Sent Events provide instant response feedback with live token streaming.',
      href: '/docs/features/chat-ui/',
    },
    {
      icon: <Settings className={cn(STYLES.iconSize, STYLES.iconColor)} />,
      title: 'Advanced Configuration',
      description: '12+ parameters for fine-tuning: temperature, top-p, frequency penalty, and more.',
      href: '/docs/features/chat-ui/',
    },
  ],
  technicalFeatures: [
    {
      icon: <Terminal className={cn(STYLES.iconSize, STYLES.iconColor)} />,
      title: 'API Compatibility',
      description: 'Drop-in replacement for OpenAI APIs. Use your existing code and tools.',
      href: '/docs/features/openapi-docs/',
    },
    {
      icon: <Cpu className={cn(STYLES.iconSize, STYLES.iconColor)} />,
      title: 'Local Processing',
      description: 'Run models on your hardware for enhanced privacy and control.',
      href: '/docs/install/',
    },
    {
      icon: <Zap className={cn(STYLES.iconSize, STYLES.iconColor)} />,
      title: 'High Performance',
      description: 'Optimized inference with llama.cpp. 8-12x speedup with GPU acceleration (CUDA, ROCm).',
      href: '/docs/deployment/docker/',
    },
    {
      icon: <Download className={cn(STYLES.iconSize, STYLES.iconColor)} />,
      title: 'Model Aliases',
      description: 'Save and switch between inference configurations instantly without restarts.',
      href: '/docs/features/model-alias/',
    },
    {
      icon: <Gauge className={cn(STYLES.iconSize, STYLES.iconColor)} />,
      title: 'Performance Metrics',
      description: 'Real-time statistics showing tokens per second and processing speed.',
      href: '/docs/features/chat-ui/',
    },
    {
      icon: <Activity className={cn(STYLES.iconSize, STYLES.iconColor)} />,
      title: 'Background Downloads',
      description: 'Download models asynchronously with progress tracking and auto-resumption.',
      href: '/docs/features/model-downloads/',
    },
  ],
};

function FeaturesSectionComponent() {
  return (
    <section className="bg-white py-12 sm:py-20">
      <Container>
        <SectionHeader title="Core Features" description="Everything you need to build AI-powered applications" />

        <div className="mb-16 space-y-4">
          <h3 className="text-2xl font-semibold tracking-tight">User Experience</h3>
          <div className={STYLES.featureGrid}>
            {features.userFeatures.map((feature, index) => (
              <FeatureCard key={feature.title} {...feature} index={index} />
            ))}
          </div>
        </div>

        <div className="space-y-4">
          <h3 className="text-2xl font-semibold tracking-tight">Technical Capabilities</h3>
          <div className={STYLES.featureGrid}>
            {features.technicalFeatures.map((feature, index) => (
              <FeatureCard key={feature.title} {...feature} index={index} />
            ))}
          </div>
        </div>
      </Container>
    </section>
  );
}

export const FeaturesSection = memo(FeaturesSectionComponent);
