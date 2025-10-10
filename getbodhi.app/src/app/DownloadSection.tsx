'use client';

import { Container } from '@/components/ui/container';
import { Card } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Download } from 'lucide-react';
import { PlatformIcon } from '@/components/PlatformIcon';
import Link from 'next/link';
import { motion } from 'framer-motion';
import { useDetectedOS } from '@/hooks/usePlatformDetection';
import { PLATFORMS } from '@/lib/constants';

export function DownloadSection() {
  const detectedOS = useDetectedOS();

  return (
    <section id="download-section" className="py-20 bg-gradient-to-b from-violet-50 to-white">
      <Container>
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="text-center"
        >
          <h2 className="text-3xl font-bold mb-4">Download for your platform</h2>
          <p className="text-muted-foreground mb-12 max-w-2xl mx-auto">
            Choose your operating system to download BodhiApp. All platforms support running LLMs locally with full
            privacy.
          </p>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-6 max-w-5xl mx-auto">
            {(Object.entries(PLATFORMS) as [keyof typeof PLATFORMS, (typeof PLATFORMS)[keyof typeof PLATFORMS]][]).map(
              ([key, platform]) => {
                const isDetected = detectedOS === key;

                return (
                  <motion.div
                    key={key}
                    initial={{ opacity: 0, y: 20 }}
                    whileInView={{ opacity: 1, y: 0 }}
                    viewport={{ once: true }}
                    transition={{ delay: 0.1 * ['macos', 'windows', 'linux'].indexOf(key) }}
                  >
                    <Card
                      className={`p-6 h-full flex flex-col transition-all hover:shadow-lg ${
                        isDetected ? 'ring-2 ring-violet-500 shadow-lg' : ''
                      }`}
                    >
                      <div className="flex items-center gap-3 mb-4">
                        <PlatformIcon platform={key} className="h-8 w-8 text-violet-600" />
                        <div className="text-left">
                          <h3 className="font-semibold text-lg">{platform.name}</h3>
                          <p className="text-sm text-muted-foreground">{platform.arch}</p>
                        </div>
                        {isDetected && (
                          <span className="ml-auto text-xs font-medium text-violet-600 bg-violet-100 px-2 py-1 rounded">
                            Detected
                          </span>
                        )}
                      </div>

                      <div className="flex-1" />

                      <div className="space-y-2">
                        <p className="text-sm text-muted-foreground">File type: {platform.fileType}</p>

                        {platform.downloadUrl ? (
                          <Button className="w-full gap-2" asChild>
                            <Link href={platform.downloadUrl}>
                              <Download className="h-4 w-4" />
                              Download for {platform.name}
                            </Link>
                          </Button>
                        ) : (
                          <Button className="w-full gap-2" disabled>
                            <Download className="h-4 w-4" />
                            Not Available
                          </Button>
                        )}
                      </div>
                    </Card>
                  </motion.div>
                );
              }
            )}
          </div>

          <p className="text-sm text-muted-foreground mt-8">
            Looking for other installation methods? Check our{' '}
            <Link href="/docs" className="text-violet-600 hover:underline">
              documentation
            </Link>{' '}
            for Docker and other deployment options.
          </p>
        </motion.div>
      </Container>
    </section>
  );
}
