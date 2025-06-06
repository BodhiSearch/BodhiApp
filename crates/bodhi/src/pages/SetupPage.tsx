import { Routes, Route } from 'react-router-dom';
import { lazy, Suspense } from 'react';

// Lazy load the setup page components
const SetupMainPage = lazy(() => import('@/components/setup/page'));
const SetupCompletePage = lazy(() => import('@/components/setup/complete/page'));
const SetupDownloadModelsPage = lazy(() => import('@/components/setup/download-models/page'));
const SetupLlmEnginePage = lazy(() => import('@/components/setup/llm-engine/page'));
const SetupResourceAdminPage = lazy(() => import('@/components/setup/resource-admin/page'));

export default function SetupPage() {
  return (
    <Suspense fallback={<div>Loading...</div>}>
      <Routes>
        <Route path="/" element={<SetupMainPage />} />
        <Route path="/complete" element={<SetupCompletePage />} />
        <Route path="/download-models" element={<SetupDownloadModelsPage />} />
        <Route path="/llm-engine" element={<SetupLlmEnginePage />} />
        <Route path="/resource-admin" element={<SetupResourceAdminPage />} />
      </Routes>
    </Suspense>
  );
}
