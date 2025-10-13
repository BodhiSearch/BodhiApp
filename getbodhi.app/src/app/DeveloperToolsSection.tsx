'use client';

import { motion } from 'framer-motion';
import { ChevronRight, Code2, Key, BookOpen, FileJson, Package } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { Container } from '@/components/ui/container';
import Link from 'next/link';
import { fadeIn } from './animations';

const developerFeatures = [
  {
    icon: <Package className="h-6 w-6 text-violet-600" />,
    title: 'TypeScript SDK',
    description: 'Production-ready npm package @bodhiapp/ts-client for seamless integration.',
    href: 'https://www.npmjs.com/package/@bodhiapp/ts-client',
    external: true,
  },
  {
    icon: <Key className="h-6 w-6 text-violet-600" />,
    title: 'API Token Management',
    description: 'Scope-based permissions with SHA-256 hashing and database-backed security.',
    href: '/docs/features/api-tokens/',
  },
  {
    icon: <BookOpen className="h-6 w-6 text-violet-600" />,
    title: 'OpenAPI/Swagger UI',
    description: 'Interactive API documentation with auto-generated specs and live testing.',
    href: '/docs/features/openapi-docs/',
  },
  {
    icon: <FileJson className="h-6 w-6 text-violet-600" />,
    title: 'OpenAI Compatible',
    description: 'Drop-in replacement for OpenAI APIs - use existing libraries and tools seamlessly.',
    href: '/docs/features/openapi-docs/',
  },
  {
    icon: <Code2 className="h-6 w-6 text-violet-600" />,
    title: 'Ollama Compatible',
    description: 'Additional API format support for Ollama chat and models endpoints.',
    href: '/docs/intro/',
  },
];

export function DeveloperToolsSection() {
  return (
    <section className="bg-gradient-to-b from-violet-50 to-white py-12 sm:py-20">
      <Container>
        <motion.div {...fadeIn} className="mb-12 space-y-4 text-center">
          <h2 className="text-3xl font-semibold tracking-tight">Developer Tools & SDKs</h2>
          <p className="text-xl text-muted-foreground mx-auto max-w-2xl">
            Everything developers need to integrate AI into applications with production-ready tools
          </p>
        </motion.div>

        <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
          {developerFeatures.map((feature, index) => (
            <motion.div
              key={index}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ delay: index * 0.1 }}
              className="h-full"
            >
              <Card className="transition-all duration-300 hover:-translate-y-1 hover:shadow-lg h-full flex flex-col">
                <CardHeader>
                  <div className="mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-violet-100">
                    {feature.icon}
                  </div>
                  <CardTitle>{feature.title}</CardTitle>
                </CardHeader>
                <CardContent className="flex-grow">
                  <p className="text-muted-foreground">{feature.description}</p>
                </CardContent>
                <CardFooter>
                  <Button variant="link" className="gap-1 p-0 hover:text-violet-600" asChild>
                    {feature.external ? (
                      <a href={feature.href} target="_blank" rel="noopener noreferrer">
                        Learn more
                        <ChevronRight className="h-4 w-4" />
                      </a>
                    ) : (
                      <Link href={feature.href}>
                        Learn more
                        <ChevronRight className="h-4 w-4" />
                      </Link>
                    )}
                  </Button>
                </CardFooter>
              </Card>
            </motion.div>
          ))}
        </div>
      </Container>
    </section>
  );
}
