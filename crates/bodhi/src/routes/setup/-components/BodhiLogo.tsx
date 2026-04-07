import { motion } from 'framer-motion';

import { itemVariants } from '../-shared/types';

export const BodhiLogoImage = () => (
  <img src="/ui/bodhi-logo/bodhi-logo-240.svg" alt="Bodhi App Logo" width={80} height={80} className="mx-auto" />
);

export const BodhiLogo = () => {
  return (
    <motion.div variants={itemVariants} className="text-center pt-4 mb-4">
      <BodhiLogoImage />
    </motion.div>
  );
};
