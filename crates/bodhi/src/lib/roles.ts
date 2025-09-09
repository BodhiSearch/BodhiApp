/**
 * Centralized role management utilities
 * Single source of truth for all role-related functionality
 */

// Role type definition - using resource_ prefix to match backend
export type Role = 'resource_user' | 'resource_power_user' | 'resource_manager' | 'resource_admin';

// Role hierarchy for access control and filtering
export const roleHierarchy: Record<Role, number> = {
  resource_admin: 4,
  resource_manager: 3,
  resource_power_user: 2,
  resource_user: 1,
};

// Role options for dropdowns and UI components
export const ROLE_OPTIONS = [
  { value: 'resource_user' as Role, label: 'User' },
  { value: 'resource_power_user' as Role, label: 'Power User' },
  { value: 'resource_manager' as Role, label: 'Manager' },
  { value: 'resource_admin' as Role, label: 'Admin' },
];

// Helper functions for role operations

/**
 * Get the display label for a role
 */
export function getRoleLabel(role: string): string {
  const option = ROLE_OPTIONS.find((r) => r.value === role);
  return option?.label || role;
}

/**
 * Get the hierarchy level for a role
 */
export function getRoleLevel(role: string): number {
  return roleHierarchy[role as Role] || 0;
}

/**
 * Check if a user role can access a required role level
 */
export function canAccessRole(userRole: string, requiredRole: string): boolean {
  const userLevel = getRoleLevel(userRole);
  const requiredLevel = getRoleLevel(requiredRole);
  return userLevel >= requiredLevel;
}

/**
 * Check if a user role meets minimum role requirement
 */
export function meetsMinRole(userRole: string, minRole: string): boolean {
  return canAccessRole(userRole, minRole);
}

/**
 * Get badge variant based on role for consistent styling
 */
export function getRoleBadgeVariant(role: string): 'default' | 'secondary' | 'outline' | 'destructive' {
  switch (role) {
    case 'resource_admin':
    case 'resource_manager':
      return 'default';
    case 'resource_power_user':
      return 'secondary';
    case 'resource_user':
      return 'outline';
    default:
      return 'secondary';
  }
}

/**
 * Filter role options based on user's maximum role level
 */
export function getAvailableRoles(userRole: string): typeof ROLE_OPTIONS {
  const userMaxLevel = getRoleLevel(userRole);
  return ROLE_OPTIONS.filter((role) => getRoleLevel(role.value) <= userMaxLevel);
}

/**
 * Check if a role is valid
 */
export function isValidRole(role: string): role is Role {
  return role in roleHierarchy;
}

/**
 * Get clean role name for display (removes resource_ prefix)
 */
export function getCleanRoleName(role: string): string {
  return role.replace('resource_', '');
}

// Type guards for specific role checks
export function isAdminRole(role: string): boolean {
  return role === 'resource_admin';
}

export function isManagerOrAbove(role: string): boolean {
  return getRoleLevel(role) >= getRoleLevel('resource_manager');
}

export function isPowerUserOrAbove(role: string): boolean {
  return getRoleLevel(role) >= getRoleLevel('resource_power_user');
}
