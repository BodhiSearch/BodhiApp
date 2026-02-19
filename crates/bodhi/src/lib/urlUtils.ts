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

    // Check if it's an IP address (all parts are numbers)
    const isIp = parts.every((part) => /^\d+$/.test(part));
    if (isIp) {
      return parts[0]; // Return first octet for IP addresses
    }

    // For domain names
    if (parts.length >= 3) {
      // For multi-part domains, return the second part (index 1)
      // This handles: mcp.asana.com → asana, api.example.co.uk → example
      return parts[1];
    } else if (parts.length === 2) {
      // For two-part domains like asana.com, return first part
      return parts[0];
    } else {
      // Single part hostname (localhost, etc.)
      return parts[0];
    }
  } catch {
    // Invalid URL
    return '';
  }
}
