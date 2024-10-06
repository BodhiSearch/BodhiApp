import { Html, Head, Main, NextScript } from "next/document";
import { GeistSans } from 'geist/font/sans'
import { GeistMono } from 'geist/font/mono'
import { cn } from "@/lib/utils";

export default function Document() {
  return (
    <Html lang="en">
      <Head />
      <body className={cn(
        'font-sans antialiased',
        GeistSans.variable,
        GeistMono.variable
      )}>
        <Main />
        <NextScript />
      </body>
    </Html>
  );
}
