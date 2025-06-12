import AppInitializer from '@/components/AppInitializer';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';

export function UsersPageContent() {
  return (
    <div className="container mx-auto py-6">
      <Card>
        <CardHeader>
          <CardTitle className="text-center">Coming Soon</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex flex-col items-center justify-center py-8">
            We&apos;re working hard to bring you these amazing features. Thanks
            and Stay Tuned!
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

export default function UsersPage() {
  return (
    <AppInitializer authenticated={true} allowedStatus="ready">
      <UsersPageContent />
    </AppInitializer>
  );
}
