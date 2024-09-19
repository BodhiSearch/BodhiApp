'use client';

import { Suspense } from 'react';
import { useState } from 'react';
import AppInitializer from '@/components/AppInitializer';
import AppHeader from '@/components/AppHeader';
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import Image from 'next/image';

interface FeaturedModel {
  owner: string;
  repo_name: string;
  filename: string;
  modelname: string;
  logo: string;
  short_description: string;
  tags: string[];
  downloads: number;
  last_updated: string;
}

function HomeContent() {
  const [featuredModels] = useState<FeaturedModel[]>([]);

  // TODO: commenting for time being
  // useEffect(() => {
  //   fetch('https://api.getbodhi.app/featured-models')
  //     .then((response) => response.json())
  //     .then((data) => setFeaturedModels(data));
  // }, []);

  return (
    <div className="container mx-auto px-4 sm:px-6 lg:px-8">
      <AppHeader />
      <h2 className="text-2xl sm:text-3xl font-bold mb-4 sm:mb-6 mt-8">
        Featured Models
      </h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 sm:gap-6">
        {featuredModels.map((model, index) => (
          <Card key={index} className="flex flex-col bg-white">
            <CardHeader>
              <div className="flex items-center space-x-4">
                <Image
                  src={model.logo}
                  alt={model.modelname}
                  width={64}
                  height={64}
                  className="w-12 h-12 sm:w-16 sm:h-16"
                />
                <CardTitle className="text-lg sm:text-xl">
                  {model.modelname}
                </CardTitle>
              </div>
            </CardHeader>
            <CardContent className="flex-grow">
              <p className="text-sm text-gray-600 mb-2">
                {model.short_description}
              </p>
              <div className="flex flex-wrap gap-1 sm:gap-2 mb-2">
                {model.tags.map((tag, i) => (
                  <Badge
                    key={i}
                    variant="secondary"
                    className="text-xs sm:text-sm"
                  >
                    {tag}
                  </Badge>
                ))}
              </div>
              <p className="text-xs text-gray-500">
                Downloads: {model.downloads} | Last updated:{' '}
                {new Date(model.last_updated).toLocaleDateString()}
              </p>
            </CardContent>
            <CardFooter>
              <Button
                onClick={() => console.log(`Downloading ${model.modelname}`)}
                className="w-full"
              >
                Download
              </Button>
            </CardFooter>
          </Card>
        ))}
      </div>
    </div>
  );
}

export default function HomePage() {
  return (
    <Suspense fallback={<div>Loading...</div>}>
      <AppInitializer allowedStatus="ready">
        <HomeContent />
      </AppInitializer>
    </Suspense>
  );
}
