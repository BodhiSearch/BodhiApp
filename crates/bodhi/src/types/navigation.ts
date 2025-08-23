import { LucideIcon } from 'lucide-react';

export interface NavigationItem {
  title: string;
  href?: string;
  description?: string;
  icon?: LucideIcon;
  items?: NavigationItem[];
  skip?: boolean;
  target?: string;
}
