import { Inter } from 'next/font/google';
import './globals.css';
import { Header } from './Header';

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
  title: 'Bodhi App - Run LLMs locally',
  description: 'Run LLMs locally with complete privacy and control',
};
