'use client';

import React from 'react';
import AppHeader from '@/components/AppHeader';
import CreateAliasForm from '@/components/CreateAliasForm';

const CreateAliasPage: React.FC = () => {
  return (
    <div className="container mx-auto px-4 sm:px-6 lg:px-8">
      <AppHeader />
      <CreateAliasForm />
    </div>
  );
};

export default CreateAliasPage;
