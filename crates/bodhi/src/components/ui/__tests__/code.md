# Code Examples

## Inline Code
Here is some \`inline code\` in a sentence.

## Code Block
```typescript
interface User {
  id: number;
  name: string;
  email?: string;
}

function greetUser(user: User) {
  console.log(\`Hello, \${user.name}!\`);
}
```

## Multiple Languages
```python
def factorial(n: int) -> int:
    return 1 if n <= 1 else n * factorial(n - 1)
```

```css
.container {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
```

## With Line Highlights
```typescript {3,5-7}
import { useState } from 'react';

// This line will be highlighted
function Counter() {
  // These lines will be highlighted
  const [count, setCount] = useState(0);
  const increment = () => setCount(c => c + 1);
  
  return (
    <button onClick={increment}>
      Count: {count}
    </button>
  );
}
```