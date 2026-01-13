import React from 'react';

const createMotionComponent = (Component: string) => {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any, react/display-name
  return React.forwardRef(({ children, ...props }: any, ref: any) => {
    // List of framer-motion specific props to filter out
    const motionProps = [
      'animate',
      'initial',
      'exit',
      'variants',
      'transition',
      'whileHover',
      'whileTap',
      'whileFocus',
      'whileInView',
      'whileDrag',
      'drag',
      'dragConstraints',
      'dragElastic',
      'dragMomentum',
      'dragTransition',
      'onDrag',
      'onDragStart',
      'onDragEnd',
      'layout',
      'layoutId',
      'custom',
      'onAnimationStart',
      'onAnimationComplete',
      // Note: 'style' deliberately NOT filtered - it's a valid HTML prop
    ];

    // Filter out motion-specific props, preserve all HTML props
    const htmlProps = Object.keys(props).reduce((acc, key) => {
      if (!motionProps.includes(key)) {
        acc[key] = props[key];
      }
      return acc;
    }, {} as any);

    return React.createElement(Component, { ...htmlProps, ref }, children);
  });
};

export const motion = {
  div: createMotionComponent('div'),
  span: createMotionComponent('span'),
  button: createMotionComponent('button'),
  a: createMotionComponent('a'),
  section: createMotionComponent('section'),
  article: createMotionComponent('article'),
  header: createMotionComponent('header'),
  footer: createMotionComponent('footer'),
  nav: createMotionComponent('nav'),
  main: createMotionComponent('main'),
  p: createMotionComponent('p'),
  h1: createMotionComponent('h1'),
  h2: createMotionComponent('h2'),
  h3: createMotionComponent('h3'),
  // Add more as needed
};

export const AnimatePresence = ({ children }: { children?: React.ReactNode }) => <>{children}</>;
export const useAnimation = () => ({});
export const useMotionValue = (initial: any) => ({ set: () => {}, get: () => initial });
export const useTransform = () => ({});
export const useSpring = () => ({});
export const useScroll = () => ({ scrollYProgress: { get: () => 0 } });
