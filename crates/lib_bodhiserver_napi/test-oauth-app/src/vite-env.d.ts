/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly INTEG_TEST_MAIN_AUTH_URL?: string;
  readonly INTEG_TEST_AUTH_REALM?: string;
  readonly INTEG_TEST_APP_CLIENT_ID?: string;
  readonly INTEG_TEST_USERNAME?: string;
  readonly INTEG_TEST_PASSWORD?: string;
  readonly INTEG_TEST_DEV_CONSOLE_CLIENT_SECRET?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
