import { motion } from 'framer-motion';

import { itemVariants } from '@/routes/setup/-shared/types';

export const BodhiLogoImage = () => (
  <img
    src="/ui/bodhi-logo/bodhi-logo-240.svg"
    alt="Bodhi App Logo"
    width={60}
    height={60}
    className="setup-lotus mx-auto h-[60px] w-[60px]"
  />
);

export const BodhiLogo = () => {
  return (
    <motion.div variants={itemVariants} className="mb-7 text-center">
      <BodhiLogoImage />
    </motion.div>
  );
};
