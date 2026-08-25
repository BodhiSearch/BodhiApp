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

// Entrance motion is TRANSFORM-ONLY (no opacity): a mid-fade snapshot captured by the route-level
// view-transition cross-fade would otherwise leave the page stuck faded.
export const containerVariants = {
  hidden: {},
  visible: {
    transition: {
      staggerChildren: 0.08,
    },
  },
};

export const itemVariants = {
  hidden: { y: 12 },
  visible: {
    y: 0,
  },
};
