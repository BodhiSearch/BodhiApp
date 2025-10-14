'use client';

import { memo } from 'react';
import { motion } from 'framer-motion';
import { Database, FileJson, Rocket } from 'lucide-react';
import { Container } from '@/components/ui/container';

function SocialProofSectionComponent() {
  return (
    <section className="py-8 bg-white/50">
      <Container>
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="flex justify-center items-center gap-12 flex-wrap"
        >
          <div className="flex items-center gap-2">
            <Rocket className="w-5 h-5 text-gray-600" />
            <span className="text-sm font-medium text-gray-600">Powered by llama.cpp</span>
          </div>
          <div className="flex items-center gap-2">
            <Database className="w-5 h-5 text-gray-600" />
            <span className="text-sm font-medium text-gray-600">HuggingFace Ecosystem</span>
          </div>
          <div className="flex items-center gap-2">
            <FileJson className="w-5 h-5 text-gray-600" />
            <span className="text-sm font-medium text-gray-600">OpenAI API Compatible</span>
          </div>
        </motion.div>
      </Container>
    </section>
  );
}

export const SocialProofSection = memo(SocialProofSectionComponent);
