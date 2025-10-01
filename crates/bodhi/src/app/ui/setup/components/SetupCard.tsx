'use client';

import { ReactNode } from 'react';
import { motion } from 'framer-motion';
import { Card, CardContent, CardFooter, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';

interface SetupCardProps {
  title?: string | ReactNode;
  description?: string | ReactNode;
  children: ReactNode;
  footer?: ReactNode;
  className?: string;
}

const itemVariants = {
  hidden: { y: 20, opacity: 0 },
  visible: {
    y: 0,
    opacity: 1,
  },
};

export function SetupCard({ title, description, children, footer, className }: SetupCardProps) {
  return (
    <motion.div variants={itemVariants}>
      <Card className={className}>
        {title && (
          <CardHeader className="text-center">
            {typeof title === 'string' ? <CardTitle>{title}</CardTitle> : title}
            {description && (
              <CardDescription>{typeof description === 'string' ? description : <>{description}</>}</CardDescription>
            )}
          </CardHeader>
        )}
        <CardContent>{children}</CardContent>
        {footer && <CardFooter>{footer}</CardFooter>}
      </Card>
    </motion.div>
  );
}
