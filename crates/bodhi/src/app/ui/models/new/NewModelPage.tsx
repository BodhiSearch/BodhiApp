import React from 'react';
import { useSearchParams } from '@/lib/navigation';
import AliasForm from '@/app/ui/models/AliasForm';
import AppInitializer from '@/components/AppInitializer';

function NewModelContent() {
  const searchParams = useSearchParams();

  const initialData = {
    alias: '',
    repo: searchParams?.get('repo') || '',
    filename: searchParams?.get('filename') || '',
    snapshot: searchParams?.get('snapshot') || '',
    chat_template: '',
    request_params: {},
    context_params: {},
  };

  return <AliasForm isEditMode={false} initialData={initialData} />;
}

export default function NewModel() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <NewModelContent />
    </AppInitializer>
  );
}
