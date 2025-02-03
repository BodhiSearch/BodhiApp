export const docsConfig = {
  // Lower number = higher priority in ordering
  order: {
    intro: 0,
    'getting-started': 100,
    'getting-started/index': 101,
    'getting-started/setup': 102,
    'getting-started/intro': 103,
    features: 200,
    'features/chat-ui': 201,
    'features/api-tokens': 202,
    'model-management': 300,
    'model-management/intro': 301,
    'developer-docs': 400,
    'developer-docs/authentication': 401,
    'developer-docs/model-configuration': 402,
    'developer-docs/api-reference': 403,
    'developer-docs/comparison': 404,
    faq: 500,
    troubleshooting: 600,
  } as const,
};

export const getPathOrder = (path: string): number => {
  return docsConfig.order[path as keyof typeof docsConfig.order] ?? 999;
};
