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

// Entrance motion is TRANSFORM-ONLY (no opacity). Resting state is fully visible so the route-level
// view-transition root cross-fade can't capture a mid-fade (opacity:0) snapshot and leave the page
// stuck faded — and reduced-motion / E2E always see content. Matches the design's capture-safe note.
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
