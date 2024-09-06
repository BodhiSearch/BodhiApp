'use client'

import { useState, useEffect } from 'react';
import AppHeader from '@/components/AppHeader';
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

interface Model {
  id: string;
  name: string;
  description: string;
}

export default function ModelsPage() {
  const [models, setModels] = useState<Model[]>([]);

  useEffect(() => {
    // Fetch models data here (placeholder)
    setModels([
      { id: '1', name: 'Model 1', description: 'Description for Model 1' },
      { id: '2', name: 'Model 2', description: 'Description for Model 2' },
    ]);
  }, []);

  return (
    <div className="container mx-auto px-4 sm:px-6 lg:px-8">
      <AppHeader />
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 sm:gap-6">
        {models.map((model) => (
          <Card key={model.id} className="flex flex-col bg-white">
            <CardHeader>
              <CardTitle className="text-lg sm:text-xl">{model.name}</CardTitle>
            </CardHeader>
            <CardContent className="flex-grow">
              <p className="text-sm text-gray-600">{model.description}</p>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
}
