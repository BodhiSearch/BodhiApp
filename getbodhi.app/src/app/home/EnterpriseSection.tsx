'use client';

import { memo } from 'react';
import { Users, Shield, UserCheck, KeyRound, Building2 } from 'lucide-react';
import { Container } from '@/components/ui/container';
import { FeatureCard } from '@/app/home/FeatureCard';
import { SectionHeader } from '@/app/home/SectionHeader';
import { SECTION_GRADIENTS, STYLES } from '@/lib/constants';
import { cn } from '@/lib/utils';

const enterpriseFeatures = [
  {
    icon: <Users className={cn(STYLES.iconSize, STYLES.iconColor)} />,
    title: 'User Management Dashboard',
    description: 'Comprehensive admin interface for managing users, roles, and access requests.',
    href: '/docs/features/user-management/',
  },
  {
    icon: <Shield className={cn(STYLES.iconSize, STYLES.iconColor)} />,
    title: 'Role-Based Access Control',
    description: '4 role levels (User, PowerUser, Manager, Admin) with granular permission management.',
    href: '/docs/features/user-management/',
  },
  {
    icon: <UserCheck className={cn(STYLES.iconSize, STYLES.iconColor)} />,
    title: 'Access Request Workflow',
    description: 'Self-service access requests with admin approval gates and audit trail.',
    href: '/docs/features/access-requests/',
  },
  {
    icon: <KeyRound className={cn(STYLES.iconSize, STYLES.iconColor)} />,
    title: 'OAuth2 + JWT Security',
    description: 'Enterprise-grade authentication with PKCE, session management, and token lifecycle control.',
    href: '/docs/intro/',
  },
  {
    icon: <Building2 className={cn(STYLES.iconSize, STYLES.iconColor)} />,
    title: 'Multi-User Deployment',
    description: 'Secure team collaboration with session invalidation and role change enforcement.',
    href: '/docs/features/user-management/',
  },
];

function EnterpriseSectionComponent() {
  return (
    <section className={cn(SECTION_GRADIENTS.violetToWhite, 'py-12 sm:py-20')}>
      <Container>
        <SectionHeader
          title="Enterprise & Team Ready"
          description="Built for secure collaboration with enterprise-grade authentication and comprehensive user management"
        />

        <div className={STYLES.featureGrid}>
          {enterpriseFeatures.map((feature, index) => (
            <FeatureCard key={feature.title} {...feature} index={index} />
          ))}
        </div>
      </Container>
    </section>
  );
}

export const EnterpriseSection = memo(EnterpriseSectionComponent);
