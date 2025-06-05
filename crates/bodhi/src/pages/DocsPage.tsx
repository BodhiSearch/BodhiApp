import { Routes, Route } from 'react-router-dom';
import { lazy, Suspense } from 'react';

// Lazy load the docs page components
const DocsMainPage = lazy(() => import('@/pages/docs/DocsMainPage'));
const DocsSlugPage = lazy(() => import('@/pages/docs/DocsSlugPage'));

export default function DocsPage() {
  return (
    <Suspense fallback={<div>Loading...</div>}>
      <Routes>
        <Route path="/" element={<DocsMainPage />} />
        <Route path="/*" element={<DocsSlugPage />} />
      </Routes>
    </Suspense>
  );
}
