// React Router equivalent for Next.js Link
import {
  Link as RouterLink,
  LinkProps as RouterLinkProps,
} from 'react-router-dom';
import { forwardRef } from 'react';

interface LinkProps extends Omit<RouterLinkProps, 'to'> {
  href: string;
  children: React.ReactNode;
  className?: string;
  target?: string;
  rel?: string;
}

const Link = forwardRef<HTMLAnchorElement, LinkProps>(
  ({ href, children, className, target, rel, ...props }, ref) => {
    // Handle external links
    if (
      href.startsWith('http') ||
      href.startsWith('mailto:') ||
      href.startsWith('tel:')
    ) {
      return (
        <a
          href={href}
          className={className}
          target={target}
          rel={rel}
          ref={ref}
          {...props}
        >
          {children}
        </a>
      );
    }

    // Handle internal links with React Router
    return (
      <RouterLink
        to={href}
        className={className}
        target={target}
        ref={ref}
        {...props}
      >
        {children}
      </RouterLink>
    );
  }
);

Link.displayName = 'Link';

export default Link;
