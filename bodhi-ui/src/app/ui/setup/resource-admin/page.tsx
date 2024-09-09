'use client';

import { Suspense } from 'react';
import AppInitializer from '@/components/AppInitializer';
import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import Link from 'next/link';

function ResourceAdminContent() {
  const loginUrl = `${process.env.NEXT_PUBLIC_BODHI_URL}/app/login`;

  return (
    <Card className="w-full max-w-md mx-auto mt-10">
      <CardHeader>
        <CardTitle>Resource Admin Setup</CardTitle>
        <CardDescription>
          You will be made the app admin using the account you log in with.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <Link href={loginUrl} passHref>
          <Button className="w-full">Log In</Button>
        </Link>
      </CardContent>
    </Card>
  );
}

export default function ResourceAdminPage() {
  return (
    <Suspense fallback={<div>Loading...</div>}>
      <AppInitializer allowedStatus="resource-admin">
        <ResourceAdminContent />
      </AppInitializer>
    </Suspense>
  );
}
