export function validateMcpServerForm(name: string, url: string, description: string): Record<string, string> {
  const errors: Record<string, string> = {};
  if (!name.trim()) errors.name = 'Name is required';
  if (name.length > 100) errors.name = 'Name cannot exceed 100 characters';
  if (!url.trim()) errors.url = 'URL is required';
  if (url.length > 2048) errors.url = 'URL cannot exceed 2048 characters';
  try {
    if (url.trim()) new URL(url.trim());
  } catch {
    errors.url = 'URL is not valid';
  }
  if (description.length > 255) errors.description = 'Description cannot exceed 255 characters';
  return errors;
}
