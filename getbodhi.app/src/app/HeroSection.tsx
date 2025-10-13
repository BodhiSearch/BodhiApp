'use client';

import { motion } from 'framer-motion';
import { ChevronRight, ChevronDown } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Container } from '@/components/ui/container';
import Image from 'next/image';
import Link from 'next/link';
import { fadeIn } from './animations';
import { DownloadButton } from '@/components/DownloadButton';

export function HeroSection() {
  return (
    <section className="py-12 sm:py-20">
      <Container>
        <motion.div {...fadeIn} className="text-center space-y-8">
          <div className="flex justify-center mb-8">
            <a
              href="https://www.producthunt.com/posts/bodhi-app-run-llms-locally?embed=true&utm_source=badge-featured&utm_medium=badge&utm_souce=badge-bodhi-app-run-llms-locally"
              target="_blank"
              rel="noopener noreferrer"
            >
              <img
                src="https://api.producthunt.com/widgets/embed-image/v1/featured.svg?post_id=850615&theme=light&t=1738822645521"
                alt="Bodhi App - Run LLMs Locally - Your Personal, Private, Powerful AI Assistant | Free & OSS"
                width="250"
                height="54"
                style={{ width: '250px', height: '54px' }}
              />
            </a>
          </div>

          <h1 className="text-4xl font-extrabold tracking-tight lg:text-5xl">
            Your Complete AI Infrastructure:{' '}
            <span className="bg-gradient-to-r from-violet-600 to-purple-400 bg-clip-text text-transparent animate-gradient">
              Local Privacy, Cloud Power
            </span>
          </h1>
          <p className="text-xl text-muted-foreground mx-auto max-w-2xl">
            Unified platform combining local GGUF models with API providers (OpenAI, Anthropic, Groq). Enterprise-ready
            with user management, OAuth2 security, and production deployment options.
          </p>
          <div className="flex flex-col sm:flex-row justify-center gap-4">
            <DownloadButton />
            <Button variant="outline" size="lg" className="gap-2" asChild>
              <Link href="#download-section">
                Download for other platforms
                <ChevronDown className="h-4 w-4" />
              </Link>
            </Button>
            <Button variant="outline" size="lg" className="gap-2" asChild>
              <Link href="https://github.com/BodhiSearch/BodhiApp" target="_blank" rel="noopener noreferrer">
                View on GitHub
                <ChevronRight className="h-4 w-4" />
              </Link>
            </Button>
          </div>

          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.3 }}
            className="mx-auto mt-12 max-w-5xl overflow-hidden rounded-xl shadow-2xl"
          >
            <Image
              src="/chat-ui.jpeg"
              alt="Bodhi Chat Interface"
              width={1200}
              height={675}
              className="w-full"
              priority
            />
          </motion.div>
        </motion.div>
      </Container>
    </section>
  );
}
