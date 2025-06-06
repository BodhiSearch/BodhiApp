/// <reference types="vite/client" />
/// <reference types="vite-plugin-pwa/client" />

declare module '*.mdx' {
  let MDXComponent: (props: any) => JSX.Element;
  export default MDXComponent;
}

declare module 'virtual:docs-data' {
  import { DocGroup } from '@/components/docs/types';

  interface DocsData {
    allSlugs: string[];
    docGroups: Record<string, DocGroup[]>;
    docContents: Record<string, { content: string; data: any }>;
  }

  const docsData: DocsData;
  export default docsData;
}
