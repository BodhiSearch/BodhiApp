import { Inter } from 'next/font/google';
import './globals.css';
import { Header } from '@/app/home/Header';

const inter = Inter({ subsets: ['latin'] });

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body className={inter.className}>
        <Header />
        {children}
      </body>
    </html>
  );
}

export const metadata = {
  title: 'Bodhi App - Your Unified AI Gateway',
  description: 'Unified AI gateway combining local LLM inference, cloud API proxying, and MCP tool integration with built-in chat UI and enterprise access control.',
};
