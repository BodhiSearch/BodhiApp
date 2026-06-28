/**
 * Barrel for the shell-chrome pieces. Each component now lives in its own file; this re-export
 * keeps the historical `@/components/shell/ShellChrome` import path working for existing callers.
 */
export { GlobalTooltip } from './GlobalTooltip';
export { AnchoredPopover, type AnchoredPopoverProps } from './AnchoredPopover';
export { ShellBrand, type ShellBrandProps } from './ShellBrand';
export { ShellFooter, type ShellFooterProps, type ShellFooterUser } from './ShellFooter';
export { ShellBreadcrumb, type ShellBreadcrumbProps, type ShellBreadcrumbItem } from './ShellBreadcrumb';
