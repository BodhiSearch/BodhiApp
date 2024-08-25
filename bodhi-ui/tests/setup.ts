import { config } from 'dotenv';
import { resolve } from 'path';
import '@testing-library/jest-dom';

config({ path: resolve(__dirname, '../.env.test') });
