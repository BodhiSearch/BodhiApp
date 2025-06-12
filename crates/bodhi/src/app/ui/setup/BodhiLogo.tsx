import Image from 'next/image';
import { motion } from 'framer-motion';
import { itemVariants } from '@/app/ui/setup/types';

export const BodhiLogo = () => {
  return (
    <motion.div variants={itemVariants} className="text-center mb-8">
      <Image
        src="/bodhi-logo/bodhi-logo-240.svg"
        alt="Bodhi App Logo"
        width={80}
        height={80}
        className="mx-auto"
        priority
      />
    </motion.div>
  );
};
