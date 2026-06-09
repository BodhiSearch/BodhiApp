/**
 * Extracts a meaningful domain identifier from a URL string.
 *
 * Examples:
 * - https://mcp.asana.com/mcp → "asana"
 * - https://api.example.co.uk/path → "example"
 * - http://localhost:3000 → "localhost"
 * - https://192.168.1.1/api → "192"
 * - Invalid URL → ""
 */
export function extractSecondLevelDomain(urlString: string): string {
  try {
    const hostname = new URL(urlString).hostname;
    const parts = hostname.split('.');

    const isIp = parts.every((part) => /^\d+$/.test(part));
    if (isIp) {
      return parts[0];
    }

    if (parts.length >= 3) {
      // Index 1 handles both mcp.asana.com → asana and api.example.co.uk → example
      return parts[1];
    } else if (parts.length === 2) {
      return parts[0];
    } else {
      return parts[0];
    }
  } catch {
    return '';
  }
}
