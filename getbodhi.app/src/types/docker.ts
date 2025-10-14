/**
 * Docker variant and release type definitions
 */

export interface DockerVariant {
  image_tag?: string;
  latest_tag: string;
  platforms: string[];
  pull_command: string;
  docker_flags: string[];
  gpu_type?: string;
  description?: string;
}

export interface DockerData {
  version: string;
  tag: string;
  released_at: string;
  registry: string;
  variants: Record<string, DockerVariant>;
}

export interface ReleasesData {
  docker: DockerData;
}
