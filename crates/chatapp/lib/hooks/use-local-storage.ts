import { useEffect, useState } from 'react'

export const useLocalStorage = <T>(
  key: string,
  initialValue: T
): [T, (value: T) => void] => {
  const [storedValue, setStoredValue] = useState<T>(initialValue)

  useEffect(() => {
    if (initialValue === null) {
      let item = window.localStorage.getItem(key);
      if (item) {
        const parsedValue = JSON.parse(item) as T
        setStoredValue(parsedValue)
      }
    } else {
      window.localStorage.setItem(key, JSON.stringify(initialValue))
    }
  }, [initialValue, key])

  const setValue = (value: T) => {
    // Save state
    setStoredValue(value)
    // Save to localStorage
    window.localStorage.setItem(key, JSON.stringify(value))
  }
  return [storedValue, setValue]
}
