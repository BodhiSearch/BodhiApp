'use client';

import * as Icons from 'lucide-react';
import { LucideIcon } from 'lucide-react';

interface IconMapperProps {
  name: string;
  className?: string;
}

export function IconMapper({ name, className }: IconMapperProps) {
  // @ts-ignore
  const Icon = (Icons as Record<string, LucideIcon>)[name.charAt(0).toUpperCase() + name.slice(1)];

  if (!Icon) {
    return null;
  }

  return <Icon className={className} />;
}
