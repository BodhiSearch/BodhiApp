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
    children?: {
      title: string;
      href: string;
      disabled?: boolean;
      external?: boolean;
      label?: string;
    }[];
  }[];
}
