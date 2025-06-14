import { motion } from 'framer-motion';
import Image from 'next/image';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { itemVariants } from './types';

export const WelcomeCard = () => {
  return (
    <motion.div variants={itemVariants}>
      <motion.div variants={itemVariants} className="text-center mb-8">
        <Image
          src="/bodhi-logo/bodhi-logo-240.svg"
          alt="Bodhi App Logo"
          width={120}
          height={120}
          className="mx-auto"
          priority
        />
      </motion.div>
      <Card>
        <CardHeader>
          <CardTitle className="text-center text-3xl font-bold">Welcome to Bodhi App</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <p className="text-center text-muted-foreground">Run AI Models Locally, Privately, and Completely Free</p>
          <div className="prose dark:prose-invert mx-auto text-center">
            <p>
              &quot;Bodhi&quot; (बोधि) comes from ancient Sanskrit/Pali, it means deep wisdom/intelligence, the ultimate
              goal of every being. <br />
              We believe the ongoing revolution of Aritificial Intelligence should be the same: <br />
              private, free, and <strong>accessible to everyone</strong>. <br />
              <br />
              Bodhi App is our step towards democratizing AI.
            </p>
          </div>
        </CardContent>
      </Card>
    </motion.div>
  );
};
