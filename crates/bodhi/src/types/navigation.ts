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

export interface NavigationSection {
  title: string;
  items: NavigationItem[];
}

export interface NavigationState {
  isOpen: boolean;
  toggle: () => void;
}
