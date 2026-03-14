'use client';

import React from 'react';

import ApiModelForm from '@/components/api-models/ApiModelForm';
import AppInitializer from '@/components/AppInitializer';

export default function NewApiModel() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <ApiModelForm mode="create" />
    </AppInitializer>
  );
}
