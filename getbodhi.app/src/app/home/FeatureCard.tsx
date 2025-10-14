'use client';

import { memo } from 'react';
import { motion } from 'framer-motion';
import { ChevronRight } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import Link from 'next/link';
import { staggerItem } from '@/app/home/animations';

export interface FeatureCardProps {
  icon: React.ReactNode;
  title: string;
  description: string;
  href: string;
  external?: boolean;
  index?: number;
}

function FeatureCardComponent({ icon, title, description, href, external = false, index = 0 }: FeatureCardProps) {
  return (
    <motion.div {...staggerItem(index)} className="h-full">
      <Card className="transition-all duration-300 hover:-translate-y-1 hover:shadow-lg flex flex-col h-full">
        <CardHeader>
          <div className="mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-violet-100">{icon}</div>
          <CardTitle>{title}</CardTitle>
        </CardHeader>
        <CardContent className="flex-grow">
          <p className="text-muted-foreground">{description}</p>
        </CardContent>
        <CardFooter>
          <Button variant="link" className="gap-1 p-0 hover:text-violet-600" asChild>
            {external ? (
              <a href={href} target="_blank" rel="noopener noreferrer">
                Learn more
                <ChevronRight className="h-4 w-4" />
              </a>
            ) : (
              <Link href={href}>
                Learn more
                <ChevronRight className="h-4 w-4" />
              </Link>
            )}
          </Button>
        </CardFooter>
      </Card>
    </motion.div>
  );
}

export const FeatureCard = memo(FeatureCardComponent);
