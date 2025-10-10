'use client';

import { Button } from '@/components/ui/button';
import { BookOpen, Disc as Discord, Github } from 'lucide-react';
import Image from 'next/image';
import Link from 'next/link';

export function Header() {
  return (
    <header className="sticky top-0 z-50 bg-white/80 border-b backdrop-blur-sm">
      <nav className="flex h-16 items-center justify-between max-w-7xl mx-auto px-4">
        {/* Logo */}
        <Link href="/" className="flex items-center gap-2">
          <Image src="/bodhi-logo/bodhi-logo-60.svg" alt="Bodhi Logo" width={32} height={32} className="w-8 h-8" />
          <span className="text-xl font-bold text-gray-900">Bodhi</span>
        </Link>

        {/* Action Buttons */}
        <div className="flex items-center gap-2 sm:gap-4">
          <Button variant="ghost" size="sm" className="gap-2" asChild>
            <Link href="/docs">
              <BookOpen className="w-4 h-4" />
              <span className="hidden sm:inline">Docs</span>
            </Link>
          </Button>
          <Button variant="ghost" size="sm" className="gap-2" asChild>
            <Link href="https://github.com/BodhiSearch/BodhiApp" target="_blank" rel="noopener noreferrer">
              <Github className="w-4 h-4" />
              <span className="hidden sm:inline">GitHub</span>
            </Link>
          </Button>
          <Button variant="ghost" size="sm" className="gap-2" asChild>
            <Link href="https://discord.gg/3vur28nz82" target="_blank" rel="noopener noreferrer">
              <Discord className="w-4 h-4" />
              <span className="hidden sm:inline">Discord</span>
            </Link>
          </Button>
        </div>
      </nav>
    </header>
  );
}
