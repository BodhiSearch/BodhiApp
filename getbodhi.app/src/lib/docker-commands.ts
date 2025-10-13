/**
 * Generate Docker run commands from variant metadata
 */

interface DockerRunCommandOptions {
  containerName?: string;
  hostPort?: number;
  containerPort?: number;
  publicHost?: string;
  publicPort?: number;
  dataVolume?: string;
  dockerFlags?: string[];
  registry: string;
  tag: string;
}

/**
 * Generate a Docker run command from metadata
 * @param options - Command generation options
 * @returns Multi-line Docker run command with line continuations
 */
export function generateDockerRunCommand(options: DockerRunCommandOptions): string {
  const {
    containerName = 'bodhiapp',
    hostPort = 1135,
    containerPort = 8080,
    publicHost = '0.0.0.0',
    publicPort = 1135,
    dataVolume = '$(pwd)/docker-data:/data',
    dockerFlags = [],
    registry,
    tag,
  } = options;

  const flags = [
    `docker run --name ${containerName}`,
    `-p ${hostPort}:${containerPort}`,
    `-e BODHI_PUBLIC_HOST=${publicHost}`,
    `-e BODHI_PUBLIC_PORT=${publicPort}`,
    '-e BODHI_ENCRYPTION_KEY=your-strong-encryption-key-here',
    `-v ${dataVolume}`,
    ...dockerFlags,
    `${registry}:${tag}`,
  ];

  return flags.join(' \\\n  ');
}
