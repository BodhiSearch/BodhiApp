'use client';

import React from 'react';
import AliasForm from '@/components/AliasForm';
import AppInitializer from '@/components/AppInitializer';

export default function CreateAliasPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <AliasForm isEditMode={false} />
    </AppInitializer>
  );
}
