export const docsConfig = {
  // Lower number = higher priority in ordering
  order: {
    intro: 0,
    install: 101,
    features: 200,
    'features/chat-ui': 201,
    'features/model-alias': 210,
    'features/api-tokens': 220,
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
