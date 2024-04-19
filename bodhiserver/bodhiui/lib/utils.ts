import { type ClassValue, clsx } from "clsx"
import { twMerge } from "tailwind-merge"

export const Root = '/';

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}
