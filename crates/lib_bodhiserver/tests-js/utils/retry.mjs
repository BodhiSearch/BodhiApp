// Retry a flaky action at the Page Object Model level. `action` receives the
// 1-based attempt number so it can reset state (e.g. re-navigate) on retries.
export async function withRetry(action, { retries = 2, label = 'action' } = {}) {
  let lastError;
  for (let attempt = 1; attempt <= retries + 1; attempt++) {
    try {
      return await action(attempt);
    } catch (error) {
      lastError = error;
      if (attempt <= retries) {
        console.log(`${label} attempt ${attempt} failed, retrying: ${error.message}`);
      }
    }
  }
  throw lastError;
}
