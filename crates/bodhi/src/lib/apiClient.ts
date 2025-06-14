import axios from 'axios';

const isTest = typeof process !== 'undefined' && process.env.NODE_ENV === 'test';
const apiClient = axios.create({
  baseURL: isTest ? 'http://localhost:3000' : '',
  maxRedirects: 0,
});

apiClient.interceptors.request.use((config) => {
  return config;
});

apiClient.interceptors.response.use(
  (response) => {
    return response;
  },
  (error) => {
    // Breakpoint: You can add a breakpoint here to inspect errors
    console.error('Error:', error.response?.status, error.config?.url);
    return Promise.reject(error);
  }
);

export default apiClient;
