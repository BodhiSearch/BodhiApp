export const DOCS_CONFIG = {
  rootPath: 'src/docs',
  defaultTitle: 'Documentation',
  defaultDescription:
    'Welcome to our documentation. Choose a topic below to get started.',
  errorMessages: {
    notFound: 'Documentation not found',
    loadError: 'Error loading documentation',
    markdownError: 'Error processing markdown content',
  },
} as const;

export const PROSE_CLASSES = {
  root: 'max-w-none prose prose-slate dark:prose-invert',
  heading: {
    h1: 'text-3xl font-semibold',
    h2: 'text-2xl font-bold mb-4',
    h3: 'text-lg font-semibold mb-1 mt-0',
  },
  link: 'block p-4 border rounded-lg hover:border-blue-500 transition-colors no-underline',
  description: 'text-sm text-gray-600 dark:text-gray-400 m-0',
  grid: 'grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4',
  section: 'mb-12',
} as const;
