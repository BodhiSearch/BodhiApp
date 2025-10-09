"use client";

import { Github, Disc as Discord, ExternalLink } from "lucide-react";
import Image from "next/image";
import Link from "next/link";
import { Container } from "@/components/ui/container";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";

export function Footer() {
  return (
    <footer className="border-t bg-gray-50">
      <Container className="py-12">
        <div className="grid grid-cols-1 gap-8 md:grid-cols-3">
          {/* Brand Section */}
          <div className="space-y-4">
            <Link href="/" className="flex items-center gap-2">
              <Image
                src="/bodhi-logo/bodhi-logo-60.svg"
                alt="Bodhi Logo"
                width={24}
                height={24}
                className="h-6 w-6"
              />
              <span className="font-semibold text-foreground">Bodhi</span>
            </Link>
            <p className="text-sm text-muted-foreground">
              Run LLMs locally with complete privacy and control.
            </p>
          </div>

          {/* Resources */}
          <div className="space-y-4">
            <h3 className="font-semibold text-foreground">Resources</h3>
            <nav className="flex flex-col space-y-3">
              <Link 
                href="/docs/install/" 
                className="text-sm text-muted-foreground hover:text-violet-600"
              >
                Installation Guide
              </Link>
              <Link 
                href="/docs/features/chat-ui/" 
                className="text-sm text-muted-foreground hover:text-violet-600"
              >
                Chat Interface
              </Link>
              <Link 
                href="/docs/features/model-downloads/" 
                className="text-sm text-muted-foreground hover:text-violet-600"
              >
                Model Management
              </Link>
              <Link 
                href="/docs/troubleshooting/" 
                className="text-sm text-muted-foreground hover:text-violet-600"
              >
                Troubleshooting
              </Link>
            </nav>
          </div>

          {/* Community */}
          <div className="space-y-4">
            <h3 className="font-semibold text-foreground">Community</h3>
            <nav className="flex flex-col space-y-3">
              <Button variant="link" className="h-auto justify-start p-0" asChild>
                <Link 
                  href="https://github.com/BodhiSearch/BodhiApp" 
                  target="_blank"
                  rel="noopener noreferrer"
                  className="flex items-center gap-2 text-sm text-muted-foreground hover:text-violet-600"
                >
                  <Github className="h-4 w-4" />
                  GitHub
                  <ExternalLink className="h-3 w-3" />
                </Link>
              </Button>
              <Button variant="link" className="h-auto justify-start p-0" asChild>
                <Link 
                  href="https://discord.gg/3vur28nz82" 
                  target="_blank"
                  rel="noopener noreferrer"
                  className="flex items-center gap-2 text-sm text-muted-foreground hover:text-violet-600"
                >
                  <Discord className="h-4 w-4" />
                  Discord Community
                  <ExternalLink className="h-3 w-3" />
                </Link>
              </Button>
            </nav>
          </div>
        </div>

        <Separator className="my-8" />

        <div className="flex flex-col items-center justify-between gap-4 sm:flex-row">
          <p className="text-sm text-muted-foreground">
            © {new Date().getFullYear()} Bodhi App. All rights reserved.
          </p>
          <div className="flex items-center gap-6">
            <Button variant="ghost" size="icon" asChild>
              <Link
                href="https://github.com/BodhiSearch/BodhiApp"
                target="_blank"
                rel="noopener noreferrer"
                className="text-muted-foreground hover:text-violet-600"
              >
                <Github className="h-5 w-5" />
              </Link>
            </Button>
            <Button variant="ghost" size="icon" asChild>
              <Link
                href="https://discord.gg/3vur28nz82"
                target="_blank"
                rel="noopener noreferrer"
                className="text-muted-foreground hover:text-violet-600"
              >
                <Discord className="h-5 w-5" />
              </Link>
            </Button>
          </div>
        </div>
      </Container>
    </footer>
  );
} 