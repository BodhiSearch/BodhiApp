'use client';

import { HeroSection } from './HeroSection';
import { SocialProofSection } from './SocialProofSection';
import { FeaturesSection } from './FeaturesSection';
import { DownloadSection } from './DownloadSection';
import { Footer } from './Footer';

export default function Home() {
  return (
    <div className="min-h-screen bg-gradient-to-b from-white to-violet-50">
      <a
        href="#main-content"
        className="sr-only focus:not-sr-only focus:absolute focus:z-50 focus:p-4 focus:bg-white focus:text-violet-600"
      >
        Skip to main content
      </a>
      <main id="main-content">
        <HeroSection />
        <SocialProofSection />
        <FeaturesSection />
        <DownloadSection />
      </main>
      <Footer />
    </div>
  );
}
