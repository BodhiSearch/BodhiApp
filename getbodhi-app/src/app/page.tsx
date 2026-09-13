'use client';

import { HeroSection } from '@/app/home/HeroSection';
import { SocialProofSection } from '@/app/home/SocialProofSection';
import { FeaturesSection } from '@/app/home/FeaturesSection';
import { EnterpriseSection } from '@/app/home/EnterpriseSection';
import { DeveloperToolsSection } from '@/app/home/DeveloperToolsSection';
import { DeploymentSection } from '@/app/home/DeploymentSection';
import { DownloadSection } from '@/app/home/DownloadSection';
import { DockerSection } from '@/app/home/DockerSection';
import { Footer } from '@/app/home/Footer';

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
        <EnterpriseSection />
        <DeveloperToolsSection />
        <DeploymentSection />
        <DownloadSection />
        <DockerSection />
      </main>
      <Footer />
    </div>
  );
}
