export type Benefit = {
  title: string;
  description: string;
  icon: string;
};

export type SetupRequirement = {
  title: string;
  description: string;
  icon: string;
  details: string;
};

export const containerVariants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: {
      staggerChildren: 0.1,
    },
  },
};

export const itemVariants = {
  hidden: { y: 20, opacity: 0 },
  visible: {
    y: 0,
    opacity: 1,
  },
};
