import { motion } from 'framer-motion';

import { itemVariants } from '@/routes/setup/-shared/types';

export const WelcomeCard = () => {
  return (
    <motion.header variants={itemVariants} data-testid="welcome-card" className="mb-9 text-center">
      <h1 className="mb-3 text-4xl font-bold leading-tight tracking-tight md:text-[44px]">
        Welcome to Bodhi App <span className="font-medium text-[hsl(var(--primary-hover))]">बोधि</span>
      </h1>
      <p className="mx-auto max-w-[46ch] text-[17px] leading-relaxed text-muted-foreground">
        Your Personal AI Hub — local, remote, and everywhere. &quot;Bodhi&quot; (बोधि) comes from ancient Sanskrit,
        meaning deep wisdom and awakening.
      </p>
    </motion.header>
  );
};
