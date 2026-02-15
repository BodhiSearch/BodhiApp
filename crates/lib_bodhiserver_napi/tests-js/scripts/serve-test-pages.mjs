import { createStaticServer } from '../utils/static-server.mjs';

const port = parseInt(process.env.STATIC_SERVER_PORT || '55173', 10);
const server = createStaticServer(port);

await server.startServer();

process.on('SIGINT', async () => {
  await server.stopServer();
  process.exit(0);
});

process.on('SIGTERM', async () => {
  await server.stopServer();
  process.exit(0);
});
