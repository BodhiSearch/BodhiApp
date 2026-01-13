'use client';

import { motion } from 'framer-motion';

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';

interface BenefitCardProps {
  title: string;
  description: string;
  icon: string;
  isNew?: boolean;
}

export function BenefitCard({ title, description, icon, isNew }: BenefitCardProps) {
  const testId = `benefit-card-${title.toLowerCase().replace(/\s+/g, '-')}`;

  return (
    <motion.div whileHover={{ scale: 1.02 }} whileTap={{ scale: 0.98 }} transition={{ type: 'spring', stiffness: 300 }}>
      <Card className="h-full relative" data-testid={testId}>
        {isNew && (
          <span className="absolute top-2 right-2 text-xs bg-primary text-primary-foreground px-2 py-1 rounded">
            NEW
          </span>
        )}
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <span className="text-2xl">{icon}</span>
            <span>{title}</span>
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-muted-foreground">{description}</p>
        </CardContent>
      </Card>
    </motion.div>
  );
}
