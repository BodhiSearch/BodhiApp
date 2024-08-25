'use client'

import { useEffect, useState } from 'react';
import { useRouter } from 'next/navigation';
import { Alert, AlertTitle, AlertDescription } from '@/components/ui/alert';

const AppInitializer = () => {
  const router = useRouter();
  const [error, setError] = useState<string | null>(null);
  const bodhi_url = process.env.NEXT_PUBLIC_BODHI_URL;

  useEffect(() => {
    const initializeApp = async () => {
      try {
        const response = await fetch(`${bodhi_url}/app/info`);
        if (!response.ok) {
          throw new Error('Network response was not ok');
        }
        const data = await response.json();

        if (data.status === 'setup') {
          router.push('/ui/setup');
        } else if (data.status === 'ready') {
          router.push('/ui/home');
        } else {
          setError('Unexpected response from server');
        }
      } catch (error) {
        setError(`Unable to connect to backend: '${bodhi_url}'`);
      }
    };

    initializeApp();
  }, [router, bodhi_url]);

  if (error) {
    return (
      <Alert variant="destructive">
        <AlertTitle>Error</AlertTitle>
        <AlertDescription>{error}</AlertDescription>
      </Alert>
    );
  }

  return (
    <div className="flex justify-center items-center h-screen">
      <div className="animate-spin rounded-full h-32 w-32 border-t-2 border-b-2 border-gray-900"></div>
    </div>
  );
};

export default AppInitializer;