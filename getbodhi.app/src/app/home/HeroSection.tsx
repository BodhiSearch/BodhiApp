'use client';

import { motion } from 'framer-motion';
import { Container } from '@/components/ui/container';
import Image from 'next/image';
import { fadeIn } from '@/app/home/animations';
import { HeroCTA } from '@/app/home/HeroCTA';
import { useDetectedOS } from '@/hooks/usePlatformDetection';
import { PLATFORMS, SOCIAL_LINKS } from '@/lib/constants';

export function HeroSection() {
  const detectedOS = useDetectedOS();
  const platformData = detectedOS !== 'unknown' ? PLATFORMS[detectedOS as keyof typeof PLATFORMS] : undefined;
  return (
    <section className="py-12 sm:py-20">
      <Container>
        <motion.div {...fadeIn} className="text-center space-y-8">
          <div className="flex justify-center mb-8">
            <a href={SOCIAL_LINKS.productHunt} target="_blank" rel="noopener noreferrer">
              <img
                src="https://api.producthunt.com/widgets/embed-image/v1/featured.svg?post_id=850615&theme=light&t=1738822645521"
                alt="Bodhi App - Run LLMs Locally - Your Personal, Private, Powerful AI Assistant | Free & OSS"
                width="250"
                height="54"
                className="w-[250px] h-[54px]"
              />
            </a>
          </div>

          <h1 className="text-4xl font-extrabold tracking-tight lg:text-5xl">
            Your Complete AI Infrastructure:{' '}
            <span className="bg-gradient-to-r from-violet-600 to-purple-400 bg-clip-text text-transparent animate-gradient">
              Local Privacy, Cloud Power
            </span>
          </h1>
          <HeroCTA platform={detectedOS} platformData={platformData} />
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
