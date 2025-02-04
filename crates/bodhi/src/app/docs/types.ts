export interface NavItem {
  title: string;
  slug: string;
  children?: NavItem[];
  parentSlug?: string;
}

export interface NavigationProps {
  items: NavItem[];
}

export interface SidebarNavProps extends React.HTMLAttributes<HTMLDivElement> {
  items: {
    title: string;
    href?: string;
    disabled?: boolean;
    external?: boolean;
    label?: string;
    selected?: boolean;
    children?: {
      title: string;
      href: string;
      disabled?: boolean;
      external?: boolean;
      label?: string;
      selected?: boolean;
    }[];
  }[];
}

export interface DocDetails {
  title: string;
  description: string;
  slug: string;
  order: number;
}

export interface DocGroup {
  title: string;
  items: DocDetails[];
  order: number;
  key?: string;
}
