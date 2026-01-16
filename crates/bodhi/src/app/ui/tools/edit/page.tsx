'use client';

import { useSearchParams } from 'next/navigation';

import AppInitializer from '@/components/AppInitializer';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { ToolConfigForm } from '../ToolConfigForm';

function EditToolContent() {
  const searchParams = useSearchParams();
  const toolId = searchParams?.get('toolid');

  if (!toolId) {
    return <ErrorPage title="Not Found" message="Tool ID is required" />;
  }

  const handleSuccess = () => {
    // Stay on page after save, the form refetches data automatically
  };

  return (
    <div className="container mx-auto p-4" data-testid="tool-edit-page">
      <ToolConfigForm toolId={toolId} onSuccess={handleSuccess} />
    </div>
  );
}

export default function EditToolPage() {
  return (
    <AppInitializer authenticated={true} allowedStatus="ready">
      <EditToolContent />
    </AppInitializer>
  );
}
