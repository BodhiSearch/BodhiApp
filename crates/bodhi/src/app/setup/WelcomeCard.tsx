import { motion } from 'framer-motion';

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';

import { itemVariants } from './types';

export const WelcomeCard = () => {
  return (
    <motion.div variants={itemVariants}>
      <Card data-testid="welcome-card">
        <CardHeader>
          <CardTitle className="text-center text-3xl font-bold">Welcome to Bodhi App</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4 px-4">
          <p className="text-center text-muted-foreground text-lg">
            Your Personal AI Hub - Local, Remote, and Everywhere
          </p>
          <div className="prose dark:prose-invert mx-auto text-center max-w-3xl">
            <p>&quot;Bodhi&quot; (बोधि) comes from ancient Sanskrit/Pali, meaning deep wisdom and intelligence.</p>
            <p>We believe AI should be private, secure, accessible to everyone, and available everywhere.</p>
            <p>Bodhi App democratizes AI by putting you in control of how, where, and when you use AI.</p>
          </div>
        </CardContent>
      </Card>
    </motion.div>
  );
};
