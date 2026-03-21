import { createFileRoute } from '@tanstack/react-router';

import LlmEnginePage from '@/app/setup/llm-engine/page';

export const Route = createFileRoute('/setup/llm-engine/')({
  component: LlmEnginePage,
});
