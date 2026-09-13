'use client';

import { memo } from 'react';
import { motion } from 'framer-motion';
import { fadeIn } from '@/app/home/animations';
import { STYLES } from '@/lib/constants';

interface SectionHeaderProps {
  title: string;
  description: string;
}

function SectionHeaderComponent({ title, description }: SectionHeaderProps) {
  return (
    <motion.div {...fadeIn} className="mb-12 space-y-4 text-center">
      <h2 className={STYLES.sectionHeading}>{title}</h2>
      <p className={STYLES.sectionDescription}>{description}</p>
    </motion.div>
  );
}

export const SectionHeader = memo(SectionHeaderComponent);
