/**
 * Query-key factory for external reference-API data (https://api.getbodhi.app/).
 * Domain hooks added per-batch (e.g. the MCP Discover catalog) extend this.
 */
export const referenceKeys = {
  all: ['reference'] as const,
};
