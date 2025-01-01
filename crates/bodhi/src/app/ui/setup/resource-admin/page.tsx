'use client';

import AppInitializer from '@/components/AppInitializer';
import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { path_app_login } from '@/lib/utils';
import Link from 'next/link';

function ResourceAdminContent() {
  return (
    <Card className="w-full max-w-md mx-auto mt-10">
      <CardHeader>
        <CardTitle>Resource Admin Setup</CardTitle>
        <CardDescription>
          You will be made the app admin using the account you log in with.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <Link href={path_app_login} passHref>
          <Button className="w-full">Log In</Button>
        </Link>
      </CardContent>
    </Card>
  );
}

export default function ResourceAdminPage() {
  return (
    <AppInitializer allowedStatus="resource-admin" authenticated={false}>
      <ResourceAdminContent />
    </AppInitializer>
  );
}
