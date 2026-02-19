import { describe, expect, it } from 'vitest';
import { extractSecondLevelDomain } from './urlUtils';

describe('extractSecondLevelDomain', () => {
  it('extracts second-level domain from standard URL', () => {
    expect(extractSecondLevelDomain('https://mcp.asana.com/mcp')).toBe('asana');
  });

  it('extracts second-level domain from multi-level TLD', () => {
    expect(extractSecondLevelDomain('https://api.example.co.uk/path')).toBe('example');
  });

  it('extracts hostname from localhost URL', () => {
    expect(extractSecondLevelDomain('http://localhost:3000')).toBe('localhost');
  });

  it('extracts first octet from IP address', () => {
    expect(extractSecondLevelDomain('https://192.168.1.1/api')).toBe('192');
  });

  it('returns empty string for invalid URL', () => {
    expect(extractSecondLevelDomain('not-a-valid-url')).toBe('');
  });

  it('handles URLs without path', () => {
    expect(extractSecondLevelDomain('https://github.com')).toBe('github');
  });

  it('handles subdomain URLs', () => {
    expect(extractSecondLevelDomain('https://api.github.com/repos')).toBe('github');
  });

  it('handles single-part hostname', () => {
    expect(extractSecondLevelDomain('http://server')).toBe('server');
  });

  it('returns empty string for empty string input', () => {
    expect(extractSecondLevelDomain('')).toBe('');
  });
});
