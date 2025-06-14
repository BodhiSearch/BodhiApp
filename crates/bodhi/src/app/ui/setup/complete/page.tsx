'use client';

import AppInitializer from '@/components/AppInitializer';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { ROUTE_CHAT } from '@/lib/constants';
import { motion } from 'framer-motion';
import { ArrowRight, BookOpen } from 'lucide-react';
import { useRouter } from 'next/navigation';
import { useEffect, useState } from 'react';
import { siDiscord, siGithub, siX, siYoutube } from 'simple-icons';
import { BodhiLogo } from '@/app/ui/setup/BodhiLogo';

const containerVariants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: { staggerChildren: 0.1 },
  },
};

const itemVariants = {
  hidden: { y: 20, opacity: 0 },
  visible: { y: 0, opacity: 1 },
};

// Simple Icon component
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
    url: 'https://docs.getbodhi.app/getting-started',
  },
];

// Magic UI Confetti component
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
      <style jsx>{`
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
  const router = useRouter();
  const [showConfetti, setShowConfetti] = useState(true);

  useEffect(() => {
    const timer = setTimeout(() => setShowConfetti(false), 5000);
    return () => clearTimeout(timer);
  }, []);

  return (
    <main className="min-h-screen bg-background">
      <motion.div
        className="mx-auto max-w-4xl space-y-8 p-4 md:p-8"
        variants={containerVariants}
        initial="hidden"
        animate="visible"
      >
        {showConfetti && <Confetti />}
        <BodhiLogo />

        {/* Completion Message */}
        <motion.div variants={itemVariants} className="text-center space-y-4">
          <h1 className="text-4xl font-bold">🎉 Setup Complete!</h1>
          <p className="text-muted-foreground">
            Your Bodhi App is ready to use. Join our community to get the most out of it!
          </p>
        </motion.div>

        {/* Social Links */}
        <motion.div variants={itemVariants}>
          <Card>
            <CardHeader>
              <CardTitle className="text-center">Join Our Community</CardTitle>
            </CardHeader>
            <CardContent className="grid gap-4">
              {socialLinks.map((link) => (
                <motion.a
                  key={link.title}
                  href={link.url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className={`flex items-center gap-4 p-4 rounded-lg transition-colors ${link.color}`}
                  whileHover={{ scale: 1.02 }}
                  whileTap={{ scale: 0.98 }}
                >
                  {link.icon}
                  <div className="flex-1">
                    <h3 className="font-medium flex items-center gap-2">
                      {link.title}
                      {link.stats && <span className="text-sm text-muted-foreground">{link.stats}</span>}
                    </h3>
                    <p className="text-sm text-muted-foreground">{link.description}</p>
                  </div>
                  <ArrowRight className="h-4 w-4 text-muted-foreground" />
                </motion.a>
              ))}
            </CardContent>
          </Card>
        </motion.div>

        {/* Resources */}
        <motion.div variants={itemVariants}>
          <Card>
            <CardHeader>
              <CardTitle className="text-center">Quick Resources</CardTitle>
            </CardHeader>
            <CardContent className="grid gap-4">
              {resourceLinks.map((link) => (
                <motion.a
                  key={link.title}
                  href={link.url}
                  className="flex items-center gap-4 p-4 rounded-lg hover:bg-muted"
                  whileHover={{ scale: 1.02 }}
                  whileTap={{ scale: 0.98 }}
                >
                  {link.icon}
                  <div>
                    <h3 className="font-medium">{link.title}</h3>
                    <p className="text-sm text-muted-foreground">{link.description}</p>
                  </div>
                  <ArrowRight className="ml-auto h-4 w-4 text-muted-foreground" />
                </motion.a>
              ))}
            </CardContent>
          </Card>
        </motion.div>

        {/* Start Using App Button */}
        <motion.div variants={itemVariants} className="flex justify-center pt-4">
          <Button size="lg" onClick={() => router.push(ROUTE_CHAT)} className="px-8">
            Start Using Bodhi App →
          </Button>
        </motion.div>
      </motion.div>
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
