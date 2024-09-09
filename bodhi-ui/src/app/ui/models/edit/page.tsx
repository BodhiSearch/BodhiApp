'use client'

import { useSearchParams } from 'next/navigation';
import { useQuery } from 'react-query';
import axios from 'axios';
import AliasForm from '@/components/AliasForm';
import AppHeader from '@/components/AppHeader';
import { Model } from '@/types/models';

export default function EditAliasPage() {
  const searchParams = useSearchParams();
  const alias = searchParams.get('alias');

  const { data: modelData, isLoading, error } = useQuery<Model>(
    ['model', alias],
    async () => {
      const response = await axios.get(`/api/ui/models/${alias}`);
      return response.data;
    },
    {
      enabled: !!alias,
      refetchOnMount: true,
      refetchOnWindowFocus: false,
    }
  );

  if (isLoading) return <div>Loading...</div>;
  if (error) return <div>Error loading model data</div>;
  if (!modelData) return <div>No model data found</div>;

  return (
    <div className="container mx-auto px-4 sm:px-6 lg:px-8">
      <AppHeader />
      {modelData && (
        <AliasForm 
          key={modelData.alias} 
          isEditMode={true} 
          initialData={modelData} 
        />
      )}
    </div>
  );
}
