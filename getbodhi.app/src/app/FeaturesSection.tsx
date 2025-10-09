"use client";

import { motion } from "framer-motion";
import { ChevronRight, Cpu, Database, Lock, MessageSquare, Terminal, Zap } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from "@/components/ui/card";
import { Container } from "@/components/ui/container";
import Link from "next/link";
import { fadeIn } from "./animations";

const features = {
  userFeatures: [
    {
      icon: <MessageSquare className="h-6 w-6 text-violet-600" />,
      title: "Built-in Chat UI",
      description: "Intuitive chat interface with full markdown and settings.",
      href: "/docs/features/chat-ui/"
    },
    {
      icon: <Lock className="h-6 w-6 text-violet-600" />,
      title: "Privacy First",
      description: "Run everything locally on your machine with complete data control.",
      href: "/docs/intro/"
    },
    {
      icon: <Database className="h-6 w-6 text-violet-600" />,
      title: "Model Management",
      description: "One-click downloads from HuggingFace.",
      href: "/docs/features/model-downloads/"
    }
  ],
  technicalFeatures: [
    {
      icon: <Terminal className="h-6 w-6 text-violet-600" />,
      title: "API Compatibility",
      description: "Drop-in replacement for OpenAI APIs. Use your existing code and tools.",
      href: "/docs/features/openapi-docs/"
    },
    {
      icon: <Cpu className="h-6 w-6 text-violet-600" />,
      title: "Local Processing",
      description: "Run models on your hardware for enhanced privacy and control.",
      href: "/docs/install/"
    },
    {
      icon: <Zap className="h-6 w-6 text-violet-600" />,
      title: "High Performance",
      description: "Optimized inference with llama.cpp and GPU acceleration.",
      href: "/docs/install/"
    }
  ]
};

export function FeaturesSection() {
  return (
    <section className="bg-white py-12 sm:py-20">
      <Container>
        <motion.div {...fadeIn} className="mb-12 space-y-4 text-center">
          <h2 className="text-3xl font-semibold tracking-tight">
            Core Features
          </h2>
          <p className="text-xl text-muted-foreground">
            Everything you need to build AI-powered applications
          </p>
        </motion.div>

        <div className="mb-16 space-y-4">
          <h3 className="text-2xl font-semibold tracking-tight">
            User Experience
          </h3>
          <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
            {features.userFeatures.map((feature, index) => (
              <FeatureCard key={index} {...feature} />
            ))}
          </div>
        </div>

        <div className="space-y-4">
          <h3 className="text-2xl font-semibold tracking-tight">
            Technical Capabilities
          </h3>
          <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
            {features.technicalFeatures.map((feature, index) => (
              <FeatureCard key={index} {...feature} />
            ))}
          </div>
        </div>
      </Container>
    </section>
  );
}

function FeatureCard({ icon, title, description, href }: {
  icon: React.ReactNode;
  title: string;
  description: string;
  href: string;
}) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true }}
    >
      <Card className="transition-all duration-300 hover:-translate-y-1 hover:shadow-lg">
        <CardHeader>
          <div className="mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-violet-100">
            {icon}
          </div>
          <CardTitle>{title}</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-muted-foreground">{description}</p>
        </CardContent>
        <CardFooter>
          <Button variant="link" className="gap-1 p-0 hover:text-violet-600" asChild>
            <Link href={href}>
              Learn more
              <ChevronRight className="h-4 w-4" />
            </Link>
          </Button>
        </CardFooter>
      </Card>
    </motion.div>
  );
} 