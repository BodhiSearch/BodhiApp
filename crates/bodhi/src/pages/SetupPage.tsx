import { Routes, Route } from 'react-router-dom';
import { lazy, Suspense } from 'react';

// Lazy load the setup page components
const SetupMainPage = lazy(() => import('@/components/setup/SetupPage'));
const SetupCompletePage = lazy(() => import('@/components/setup/complete/SetupCompletePage'));
const SetupDownloadModelsPage = lazy(() => import('@/components/setup/download-models/DownloadModelsPage'));
const SetupLlmEnginePage = lazy(() => import('@/components/setup/llm-engine/LlmEnginePage'));
const SetupResourceAdminPage = lazy(() => import('@/components/setup/resource-admin/ResourceAdminPage'));

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
