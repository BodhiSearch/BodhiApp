import { type ClassValue, clsx } from "clsx"
import { customAlphabet } from "nanoid";
import { twMerge } from "tailwind-merge"

export const PageRoot = '/';

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export const nanoid = customAlphabet(
  '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz',
  7
)

export const userId = '29175b6f-44ed-4901-a35b-460c48c1b171';
