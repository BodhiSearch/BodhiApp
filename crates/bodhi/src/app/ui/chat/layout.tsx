import { Metadata } from 'next';
import { ChatSettingsProvider } from '@/hooks/use-chat-settings';

export const metadata: Metadata = {
  title: 'Chat | AI Assistant',
  description: 'Chat with AI Assistant'
};

export default function ChatLayout({
  children
}: {
  children: React.ReactNode;
}) {
  return <ChatSettingsProvider>{children}</ChatSettingsProvider>;
} 