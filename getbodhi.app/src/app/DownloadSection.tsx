'use client';

import { Download } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Container } from '@/components/ui/container';
import Link from 'next/link';
import { motion } from 'framer-motion';
import { DOWNLOAD_URL } from '@/lib/constants';

export function DownloadSection() {
  return (
    <section className="py-20 bg-gradient-to-b from-violet-50 to-white">
      <Container>
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="text-center"
        >
          <h2 className="text-3xl font-bold mb-6">Ready to get started?</h2>
          <p className="text-muted-foreground mb-8">
            Download Bodhi App for macOS and start running LLMs locally today.
          </p>
          <Button size="lg" className="gap-2" asChild>
            <Link href={DOWNLOAD_URL}>
              <Download className="h-5 w-5" />
              Download for macOS (Apple Silicon)
            </Link>
          </Button>
        </motion.div>
      </Container>
    </section>
  );
}
