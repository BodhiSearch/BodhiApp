'use client';

import React from 'react';
import AppHeader from '@/components/AppHeader';
import AliasForm from '@/components/AliasForm';

const CreateAliasPage: React.FC = () => {
  return (
    <div className="container mx-auto px-4 sm:px-6 lg:px-8">
      <AppHeader />
      <AliasForm isEditMode={false} />
    </div>
  );
};

export default CreateAliasPage;
