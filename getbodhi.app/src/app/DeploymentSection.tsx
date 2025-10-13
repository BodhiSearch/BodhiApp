'use client';

import { motion } from 'framer-motion';
import { ChevronRight, Monitor, Container as ContainerIcon, Cloud, Zap, HardDrive, Server } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { Container } from '@/components/ui/container';
import Link from 'next/link';
import { fadeIn } from './animations';

const deploymentOptions = [
  {
    icon: <Monitor className="h-6 w-6 text-violet-600" />,
    title: 'Multi-Platform Desktop',
    description: 'Native desktop apps for Windows, macOS (Intel/ARM), and Linux with Tauri.',
    href: '/docs/install/',
  },
  {
    icon: <ContainerIcon className="h-6 w-6 text-violet-600" />,
    title: 'Docker Variants',
    description: 'CPU (AMD64/ARM64), CUDA, ROCm, and Vulkan optimized images for every hardware.',
    href: '/docs/deployment/docker/',
  },
  {
    icon: <Cloud className="h-6 w-6 text-violet-600" />,
    title: 'Cloud Ready',
    description: 'RunPod auto-configuration and support for any Docker-compatible cloud platform.',
    href: '/docs/deployment/docker/',
  },
  {
    icon: <Zap className="h-6 w-6 text-violet-600" />,
    title: 'GPU Acceleration',
    description: '8-12x speedup with CUDA/ROCm GPU support for NVIDIA and AMD graphics cards.',
    href: '/docs/deployment/docker/',
  },
  {
    icon: <HardDrive className="h-6 w-6 text-violet-600" />,
    title: 'Volume Management',
    description: 'Persistent storage with backup/restore strategies and migration support.',
    href: '/docs/deployment/docker/',
  },
  {
    icon: <Server className="h-6 w-6 text-violet-600" />,
    title: 'Production Ready',
    description: 'Health checks, monitoring, log management, and automatic database migrations.',
    href: '/docs/deployment/docker/',
  },
];

export function DeploymentSection() {
  return (
    <section className="bg-gradient-to-b from-white to-violet-50 py-12 sm:py-20">
      <Container>
        <motion.div {...fadeIn} className="mb-12 space-y-4 text-center">
          <h2 className="text-3xl font-semibold tracking-tight">Flexible Deployment Options</h2>
          <p className="text-xl text-muted-foreground mx-auto max-w-2xl">
            Deploy anywhere - from desktop to cloud, with hardware-optimized variants for maximum performance
          </p>
        </motion.div>

        <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
          {deploymentOptions.map((option, index) => (
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
                    {option.icon}
                  </div>
                  <CardTitle>{option.title}</CardTitle>
                </CardHeader>
                <CardContent className="flex-grow">
                  <p className="text-muted-foreground">{option.description}</p>
                </CardContent>
                <CardFooter>
                  <Button variant="link" className="gap-1 p-0 hover:text-violet-600" asChild>
                    <Link href={option.href}>
                      Learn more
                      <ChevronRight className="h-4 w-4" />
                    </Link>
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
