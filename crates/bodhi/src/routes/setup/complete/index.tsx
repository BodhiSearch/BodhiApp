import { useEffect, useState } from 'react';

import { useNavigate } from '@tanstack/react-router';
import { createFileRoute } from '@tanstack/react-router';
import { motion } from 'framer-motion';
import { ArrowRight, BookOpen } from 'lucide-react';
import { siDiscord, siGithub, siX, siYoutube } from 'simple-icons';

import AppInitializer from '@/components/AppInitializer';
import { Button } from '@/components/ui/button';
import { ROUTE_CHAT } from '@/lib/constants';
import { SetupContainer, SetupCard } from '@/routes/setup/-components';
import { itemVariants } from '@/routes/setup/-shared/types';

export const Route = createFileRoute('/setup/complete/')({
  component: SetupCompletePage,
});

function SimpleIcon({ icon, className }: { icon: { path: string }; className?: string }) {
  return (
    <svg role="img" viewBox="0 0 24 24" className={className} fill="currentColor">
      <path d={icon.path} />
    </svg>
  );
}

const socialLinks = [
  {
    title: 'Star on GitHub',
    icon: <SimpleIcon icon={siGithub} className="h-5 w-5" />,
    description: 'Support the project, track updates, and contribute to development',
    url: 'https://github.com/BodhiSearch/BodhiApp',
    stats: '',
    color: 'hover:bg-zinc-100 dark:hover:bg-zinc-800',
  },
  {
    title: 'Join Discord',
    icon: <SimpleIcon icon={siDiscord} className="h-5 w-5" />,
    description: 'Connect with community, get help, and share your experience',
    url: 'https://discord.gg/3vur28nz82',
    stats: '',
    color: 'hover:bg-indigo-100 dark:hover:bg-indigo-900/30',
  },
  {
    title: 'Follow on X',
    icon: <SimpleIcon icon={siX} className="h-5 w-5" />,
    description: 'Stay updated with latest news and announcements',
    url: 'https://x.com/getbodhiapp',
    color: 'hover:bg-blue-100 dark:hover:bg-blue-900/30',
  },
  {
    title: 'Watch Tutorials',
    icon: <SimpleIcon icon={siYoutube} className="h-5 w-5" />,
    description: 'Learn tips, tricks and best practices',
    url: 'https://www.youtube.com/@anagri83',
    color: 'hover:bg-red-100 dark:hover:bg-red-900/30',
  },
];

const resourceLinks = [
  {
    title: 'Getting Started Guide',
    icon: <BookOpen className="h-5 w-5" />,
    description: 'Learn the basics and get up to speed quickly',
    url: 'https://getbodhi.app/docs/',
  },
];

function Confetti() {
  return (
    <div className="fixed inset-0 flex items-center justify-center pointer-events-none" aria-hidden="true">
      <div className="w-full h-full max-w-7xl mx-auto flex justify-center">
        <div className="w-full h-full grid grid-cols-7 gap-4">
          {Array.from({ length: 70 }).map((_, i) => (
            <div
              key={i}
              className="relative -top-full flex items-center justify-center"
              style={{
                animation: `confetti ${Math.random() * 3 + 2}s ${Math.random() * 2}s linear forwards`,
              }}
            >
              <div
                className="w-2 h-2 rotate-45 animate-spin"
                style={{
                  backgroundColor: ['#ff0000', '#00ff00', '#0000ff', '#ffff00', '#ff00ff'][
                    Math.floor(Math.random() * 5)
                  ],
                  animationDuration: `${Math.random() * 2 + 1}s`,
                }}
              />
            </div>
          ))}
        </div>
      </div>
      <style>{`
        @keyframes confetti {
          0% {
            transform: translateY(0) rotateX(0) rotateY(0);
          }
          100% {
            transform: translateY(100vh) rotateX(360deg) rotateY(360deg);
          }
        }
      `}</style>
    </div>
  );
}

function SetupCompleteContent() {
  const navigate = useNavigate();
  const [showConfetti, setShowConfetti] = useState(true);

  useEffect(() => {
    const timer = setTimeout(() => setShowConfetti(false), 5000);
    return () => clearTimeout(timer);
  }, []);

  return (
    <main className="min-h-screen bg-background">
      {showConfetti && <Confetti />}
      <SetupContainer showProgress={false}>
        <motion.div variants={itemVariants} className="mb-9 space-y-3 text-center">
          <h1 className="text-4xl font-bold tracking-tight md:text-[44px]">Setup Complete</h1>
          <p className="mx-auto max-w-[46ch] text-[17px] leading-relaxed text-muted-foreground">
            Your Bodhi App is ready to use. Join our community to get the most out of it.
          </p>
        </motion.div>

        <SetupCard title="Join our community">
          <div className="flex flex-col">
            {socialLinks.map((link, idx) => (
              <a
                key={link.title}
                href={link.url}
                target="_blank"
                rel="noopener noreferrer"
                className={`group flex items-center gap-4 py-4 transition-colors hover:bg-muted/40 ${
                  idx > 0 ? 'border-t border-border' : ''
                }`}
              >
                <span className="flex h-10 w-10 flex-none items-center justify-center rounded-[var(--radius-md)] bg-muted text-foreground">
                  {link.icon}
                </span>
                <span className="flex-1">
                  <span className="block font-medium">{link.title}</span>
                  <span className="block text-sm text-muted-foreground">{link.description}</span>
                </span>
                <ArrowRight className="h-[18px] w-[18px] flex-none text-muted-foreground transition-transform group-hover:translate-x-0.5" />
              </a>
            ))}
          </div>
        </SetupCard>

        <SetupCard title="Quick resources">
          <div className="flex flex-col">
            {resourceLinks.map((link) => (
              <a
                key={link.title}
                href={link.url}
                target="_blank"
                rel="noopener noreferrer"
                className="group flex items-center gap-4 py-4 transition-colors hover:bg-muted/40"
              >
                <span className="flex h-10 w-10 flex-none items-center justify-center rounded-[var(--radius-md)] bg-muted text-foreground">
                  {link.icon}
                </span>
                <span className="flex-1">
                  <span className="block font-medium">{link.title}</span>
                  <span className="block text-sm text-muted-foreground">{link.description}</span>
                </span>
                <ArrowRight className="h-[18px] w-[18px] flex-none text-muted-foreground transition-transform group-hover:translate-x-0.5" />
              </a>
            ))}
          </div>
        </SetupCard>

        <motion.div variants={itemVariants} className="mt-6">
          <Button size="lg" onClick={() => navigate({ to: ROUTE_CHAT })} className="w-full gap-2">
            Start Using Bodhi App
            <ArrowRight className="h-4 w-4" />
          </Button>
        </motion.div>
      </SetupContainer>
    </main>
  );
}

export default function SetupCompletePage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <SetupCompleteContent />
    </AppInitializer>
  );
}
