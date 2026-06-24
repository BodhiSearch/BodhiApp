import type { ResourceRole } from '@bodhiapp/ts-client';
export type Role = ResourceRole;

export const roleHierarchy: Record<Role, number> = {
  resource_anonymous: 0,
  resource_guest: 1,
  resource_user: 2,
  resource_power_user: 3,
  resource_manager: 4,
  resource_admin: 5,
};

export const ROLE_OPTIONS = [
  { value: 'resource_user' as Role, label: 'User' },
  { value: 'resource_power_user' as Role, label: 'Power User' },
  { value: 'resource_manager' as Role, label: 'Manager' },
  { value: 'resource_admin' as Role, label: 'Admin' },
];

export function getRoleLabel(role: string): string {
  const option = ROLE_OPTIONS.find((r) => r.value === role);
  return option?.label || role;
}

export function getRoleLevel(role: string): number {
  return roleHierarchy[role as Role] || 0;
}

export function canAccessRole(userRole: string, requiredRole: string): boolean {
  const userLevel = getRoleLevel(userRole);
  const requiredLevel = getRoleLevel(requiredRole);
  return userLevel >= requiredLevel;
}

export function meetsMinRole(userRole: string, minRole: string): boolean {
  return canAccessRole(userRole, minRole);
}

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

export function getAvailableRoles(userRole: string): typeof ROLE_OPTIONS {
  const userMaxLevel = getRoleLevel(userRole);
  return ROLE_OPTIONS.filter((role) => getRoleLevel(role.value) <= userMaxLevel);
}

export function isValidRole(role: string): role is Role {
  return role in roleHierarchy;
}

export function getCleanRoleName(role: string): string {
  return role.replace('resource_', '');
}

export function isAdminRole(role: string): boolean {
  return role === 'resource_admin';
}

export function isManagerOrAbove(role: string): boolean {
  return getRoleLevel(role) >= getRoleLevel('resource_manager');
}

export function isPowerUserOrAbove(role: string): boolean {
  return getRoleLevel(role) >= getRoleLevel('resource_power_user');
}
