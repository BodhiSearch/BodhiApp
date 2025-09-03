'use client';

import React from 'react';
import { useSearchParams } from 'next/navigation';
import ApiModelForm from '@/app/ui/api-models/ApiModelForm';
import AppInitializer from '@/components/AppInitializer';
import { useApiModel } from '@/hooks/useApiModels';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Loading } from '@/components/ui/Loading';

function EditApiModelContent() {
  const searchParams = useSearchParams();
  const id = searchParams?.get('id');

  const {
    data: apiModel,
    isLoading,
    error,
  } = useApiModel(id || '', {
    enabled: !!id,
  });

  if (!id) {
    return <ErrorPage message="No API model ID provided" />;
  }

  if (isLoading) {
    return <Loading message="Loading API model..." />;
  }

  if (error) {
    const errorMessage = error.response?.data?.error?.message || error.message || 'An unexpected error occurred';
    return <ErrorPage message={errorMessage} />;
  }

  if (!apiModel) {
    return <ErrorPage message="API model not found" />;
  }

  return <ApiModelForm isEditMode={true} initialData={apiModel} />;
}

export default function EditApiModel() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <EditApiModelContent />
    </AppInitializer>
  );
}
