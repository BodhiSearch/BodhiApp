import * as React from 'react';

export function useLocalStorage<T>(key: string, defaultValue: T): [T, (value: T | ((prevValue: T) => T)) => void] {
  const getStoredValue = React.useCallback((): T => {
    if (typeof window === 'undefined') return defaultValue;
    try {
      const item = localStorage.getItem(key);
      return item ? JSON.parse(item) : defaultValue;
    } catch (error) {
      console.warn(`Error reading localStorage key "${key}":`, error);
      return defaultValue;
    }
  }, [key, defaultValue]);

  const [storedValue, setStoredValue] = React.useState<T>(getStoredValue);

  const setValue = React.useCallback(
    (value: T | ((prevValue: T) => T)) => {
      try {
        const valueToStore = value instanceof Function ? value(storedValue) : value;

        setStoredValue(valueToStore);

        if (typeof window !== 'undefined') {
          localStorage.setItem(key, JSON.stringify(valueToStore));
        }
      } catch (error) {
        console.warn(`Error setting localStorage key "${key}":`, error);
      }
    },
    [key, storedValue]
  );

  return [storedValue, setValue];
}
