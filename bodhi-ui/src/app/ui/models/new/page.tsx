'use client';

import React from 'react';
import AppHeader from '@/components/AppHeader';
import AliasForm from '@/components/AliasForm';
import AppInitializer from '@/components/AppInitializer';

export default function CreateAliasPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <div className="container mx-auto px-4 sm:px-6 lg:px-8">
        <AppHeader />
        <AliasForm isEditMode={false} />
      </div>
    </AppInitializer>
  );
}
