'use client';

import { motion } from 'framer-motion';
import { ChevronRight, Users, Shield, UserCheck, KeyRound, Building2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { Container } from '@/components/ui/container';
import Link from 'next/link';
import { fadeIn } from './animations';

const enterpriseFeatures = [
  {
    icon: <Users className="h-6 w-6 text-violet-600" />,
    title: 'User Management Dashboard',
    description: 'Comprehensive admin interface for managing users, roles, and access requests.',
    href: '/docs/features/user-management/',
  },
  {
    icon: <Shield className="h-6 w-6 text-violet-600" />,
    title: 'Role-Based Access Control',
    description: '4 role levels (User, PowerUser, Manager, Admin) with granular permission management.',
    href: '/docs/features/user-management/',
  },
  {
    icon: <UserCheck className="h-6 w-6 text-violet-600" />,
    title: 'Access Request Workflow',
    description: 'Self-service access requests with admin approval gates and audit trail.',
    href: '/docs/features/access-requests/',
  },
  {
    icon: <KeyRound className="h-6 w-6 text-violet-600" />,
    title: 'OAuth2 + JWT Security',
    description: 'Enterprise-grade authentication with PKCE, session management, and token lifecycle control.',
    href: '/docs/intro/',
  },
  {
    icon: <Building2 className="h-6 w-6 text-violet-600" />,
    title: 'Multi-User Deployment',
    description: 'Secure team collaboration with session invalidation and role change enforcement.',
    href: '/docs/features/user-management/',
  },
];

export function EnterpriseSection() {
  return (
    <section className="bg-gradient-to-b from-violet-50 to-white py-12 sm:py-20">
      <Container>
        <motion.div {...fadeIn} className="mb-12 space-y-4 text-center">
          <h2 className="text-3xl font-semibold tracking-tight">Enterprise & Team Ready</h2>
          <p className="text-xl text-muted-foreground mx-auto max-w-2xl">
            Built for secure collaboration with enterprise-grade authentication and comprehensive user management
          </p>
        </motion.div>

        <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
          {enterpriseFeatures.map((feature, index) => (
            <motion.div
              key={index}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ delay: index * 0.1 }}
              className="h-full"
            >
              <Card className="transition-all duration-300 hover:-translate-y-1 hover:shadow-lg h-full flex flex-col">
                <CardHeader>
                  <div className="mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-violet-100">
                    {feature.icon}
                  </div>
                  <CardTitle>{feature.title}</CardTitle>
                </CardHeader>
                <CardContent className="flex-grow">
                  <p className="text-muted-foreground">{feature.description}</p>
                </CardContent>
                <CardFooter>
                  <Button variant="link" className="gap-1 p-0 hover:text-violet-600" asChild>
                    <Link href={feature.href}>
                      Learn more
                      <ChevronRight className="h-4 w-4" />
                    </Link>
                  </Button>
                </CardFooter>
              </Card>
            </motion.div>
          ))}
        </div>
      </Container>
    </section>
  );
}
