import { Circle, type LucideIcon } from 'lucide-react';
import * as LucideIcons from 'lucide-react';

export interface ShellIconProps {
  /** kebab-case lucide name, e.g. 'message-circle', 'key-round', 'panel-left'. */
  name: string;
  size?: number;
  color?: string;
  strokeWidth?: number;
}

const lucideRegistry = LucideIcons as unknown as Record<string, LucideIcon | undefined>;

const cache = new Map<string, LucideIcon>();

// 'key-round' -> 'KeyRound'. Matches lucide-react's PascalCase named exports,
// which include both canonical names and deprecated numeric-suffix aliases
// (e.g. 'globe-2' -> 'Globe2', 'plus-circle' -> 'PlusCircle').
function toPascalCase(name: string): string {
  return name
    .split('-')
    .map((part) => (part ? part.charAt(0).toUpperCase() + part.slice(1) : ''))
    .join('');
}

function resolveIcon(name: string): LucideIcon {
  const cached = cache.get(name);
  if (cached) return cached;
  const icon = lucideRegistry[toPascalCase(name)] ?? Circle;
  cache.set(name, icon);
  return icon;
}

export function ShellIcon({ name, size = 14, color, strokeWidth }: ShellIconProps) {
  const Icon = resolveIcon(name);
  return <Icon size={size} color={color} strokeWidth={strokeWidth} />;
}
