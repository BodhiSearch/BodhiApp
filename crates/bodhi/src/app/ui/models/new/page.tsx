'use client';

import React from 'react';
import AliasForm from '@/components/AliasForm';
import AppInitializer from '@/components/AppInitializer';
import { MainLayout } from '@/components/layout/MainLayout';

export default function CreateAliasPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <MainLayout>
        <div className="container mx-auto px-4 sm:px-6 lg:px-8">
          <AliasForm isEditMode={false} />
        </div>
      </MainLayout>
    </AppInitializer>
  );
}
