'use client';

import { useSearchParams } from 'next/navigation';

import AppInitializer from '@/components/AppInitializer';
import { ErrorPage } from '@/components/ui/ErrorPage';

import { ToolsetConfigForm } from '../ToolsetConfigForm';

function EditToolsetContent() {
  const searchParams = useSearchParams();
  const toolsetId = searchParams?.get('toolset_id');

  if (!toolsetId) {
    return <ErrorPage title="Not Found" message="Toolset ID is required" />;
  }

  const handleSuccess = () => {
    // Stay on page after save, the form refetches data automatically
  };

  return (
    <div className="container mx-auto p-4" data-testid="toolset-edit-page">
      <ToolsetConfigForm toolsetId={toolsetId} onSuccess={handleSuccess} />
    </div>
  );
}

export default function EditToolsetPage() {
  return (
    <AppInitializer authenticated={true} allowedStatus="ready">
      <EditToolsetContent />
    </AppInitializer>
  );
}
