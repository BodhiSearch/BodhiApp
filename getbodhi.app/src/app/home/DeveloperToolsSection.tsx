'use client';

import { memo } from 'react';
import { Code2, Key, BookOpen, FileJson, Package } from 'lucide-react';
import { Container } from '@/components/ui/container';
import { FeatureCard } from '@/app/home/FeatureCard';
import { SectionHeader } from '@/app/home/SectionHeader';
import { SECTION_GRADIENTS, STYLES } from '@/lib/constants';
import { cn } from '@/lib/utils';

const developerFeatures = [
  {
    icon: <Package className={cn(STYLES.iconSize, STYLES.iconColor)} />,
    title: 'TypeScript SDK',
    description: 'Production-ready npm package @bodhiapp/ts-client for seamless integration.',
    href: 'https://www.npmjs.com/package/@bodhiapp/ts-client',
    external: true,
  },
  {
    icon: <Key className={cn(STYLES.iconSize, STYLES.iconColor)} />,
    title: 'API Token Management',
    description: 'Scope-based permissions with SHA-256 hashing and database-backed security.',
    href: '/docs/features/api-tokens/',
  },
  {
    icon: <BookOpen className={cn(STYLES.iconSize, STYLES.iconColor)} />,
    title: 'OpenAPI/Swagger UI',
    description: 'Interactive API documentation with auto-generated specs and live testing.',
    href: '/docs/features/openapi-docs/',
  },
  {
    icon: <FileJson className={cn(STYLES.iconSize, STYLES.iconColor)} />,
    title: 'OpenAI Compatible',
    description: 'Drop-in replacement for OpenAI APIs - use existing libraries and tools seamlessly.',
    href: '/docs/features/openapi-docs/',
  },
  {
    icon: <Code2 className={cn(STYLES.iconSize, STYLES.iconColor)} />,
    title: 'Ollama Compatible',
    description: 'Additional API format support for Ollama chat and models endpoints.',
    href: '/docs/intro/',
  },
];

function DeveloperToolsSectionComponent() {
  return (
    <section className={cn(SECTION_GRADIENTS.violetToWhite, 'py-12 sm:py-20')}>
      <Container>
        <SectionHeader
          title="Developer Tools & SDKs"
          description="Everything developers need to integrate AI into applications with production-ready tools"
        />

        <div className={STYLES.featureGrid}>
          {developerFeatures.map((feature, index) => (
            <FeatureCard key={feature.title} {...feature} index={index} />
          ))}
        </div>
      </Container>
    </section>
  );
}

export const DeveloperToolsSection = memo(DeveloperToolsSectionComponent);
