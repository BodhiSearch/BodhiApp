import { lazy, Suspense } from 'react';

// Lazy load the actual page component
const ChatPageContent = lazy(() => import('@/components/chat/page'));

export default function ChatPage() {
  return (
    <Suspense fallback={<div>Loading...</div>}>
      <ChatPageContent />
    </Suspense>
  );
}
