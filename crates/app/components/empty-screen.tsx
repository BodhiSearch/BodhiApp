import { ExternalLink } from "@/components/external-link";

export function EmptyScreen() {
  return (
    <div className="mx-auto max-w-2xl px-4">
      <div className="flex flex-col gap-2 rounded-lg border bg-background p-8">
        <h1 className="text-lg font-semibold">
          Welcome to Bodhi App Chatbot!
        </h1>
        <p className="leading-normal text-muted-foreground">
          This is an open source AI chatbot app template built with{' '}
          <ExternalLink href="https://nextjs.org">Next.js</ExternalLink>, and{' '}
          <ExternalLink href="https://sdk.vercel.ai">
            Vercel AI SDK
          </ExternalLink>.
        </p>
        <p className="leading-normal text-muted-foreground">
          Bodh App helps you to run and chat with Large Language Models on your
          own laptop/desktop. 
          It is powered by open source library <ExternalLink href="https://github.com/ggerganov/llama.cpp">llama.cpp</ExternalLink>.
        </p>
      </div>
    </div>
  )
}