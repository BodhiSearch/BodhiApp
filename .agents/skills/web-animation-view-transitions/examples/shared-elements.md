# Shared Element Transitions

> Hero animations and shared element patterns. See [core.md](core.md) for basic patterns.

**Prerequisites**: Understand basic transitions from [core.md](core.md) first.

---

## Basic Shared Element

### Good Example - Product List to Detail

```typescript
// shared-element-basic.ts

const HERO_TRANSITION_NAME = "product-hero";

interface Product {
  id: string;
  name: string;
  imageUrl: string;
}

let currentView: "list" | "detail" = "list";
let selectedProduct: Product | null = null;

function setSharedElementName(element: HTMLElement | null, name: string): void {
  if (element) {
    element.style.viewTransitionName = name;
  }
}

function clearSharedElementName(element: HTMLElement | null): void {
  if (element) {
    element.style.viewTransitionName = "";
  }
}

function renderList(products: Product[]): void {
  const container = document.getElementById("content");
  if (!container) return;

  container.innerHTML = `
    <ul class="product-grid">
      ${products
        .map(
          (p) => `
        <li>
          <button data-product-id="${p.id}" class="product-card">
            <img
              src="${p.imageUrl}"
              alt="${p.name}"
              data-product-image="${p.id}"
            />
            <span>${p.name}</span>
          </button>
        </li>
      `,
        )
        .join("")}
    </ul>
  `;
}

function renderDetail(product: Product): void {
  const container = document.getElementById("content");
  if (!container) return;

  container.innerHTML = `
    <article class="product-detail">
      <img
        src="${product.imageUrl}"
        alt="${product.name}"
        data-product-image="${product.id}"
      />
      <h1>${product.name}</h1>
      <p>Product details...</p>
      <button id="back-btn">Back to list</button>
    </article>
  `;
}

export function showProductDetail(product: Product, products: Product[]): void {
  selectedProduct = product;

  // Get the source image element
  const sourceImage = document.querySelector(
    `[data-product-image="${product.id}"]`,
  ) as HTMLElement | null;

  const updateFn = () => {
    currentView = "detail";
    renderDetail(product);

    // Set name on destination image
    const destImage = document.querySelector(
      `[data-product-image="${product.id}"]`,
    ) as HTMLElement | null;
    setSharedElementName(destImage, HERO_TRANSITION_NAME);
  };

  if (!document.startViewTransition) {
    updateFn();
    return;
  }

  // Set name on source image before transition
  setSharedElementName(sourceImage, HERO_TRANSITION_NAME);

  const transition = document.startViewTransition(updateFn);

  transition.finished.then(() => {
    // Clean up names after transition
    const image = document.querySelector(
      `[data-product-image="${product.id}"]`,
    ) as HTMLElement | null;
    clearSharedElementName(image);
  });
}

export function backToList(products: Product[]): void {
  const sourceImage = document.querySelector(
    `[data-product-image="${selectedProduct?.id}"]`,
  ) as HTMLElement | null;

  const updateFn = () => {
    currentView = "list";
    renderList(products);

    // Set name on destination image in list
    if (selectedProduct) {
      const destImage = document.querySelector(
        `[data-product-image="${selectedProduct.id}"]`,
      ) as HTMLElement | null;
      setSharedElementName(destImage, HERO_TRANSITION_NAME);
    }
  };

  if (!document.startViewTransition) {
    updateFn();
    selectedProduct = null;
    return;
  }

  // Set name on source image before transition
  setSharedElementName(sourceImage, HERO_TRANSITION_NAME);

  const transition = document.startViewTransition(updateFn);

  transition.finished.then(() => {
    // Clean up
    if (selectedProduct) {
      const image = document.querySelector(
        `[data-product-image="${selectedProduct.id}"]`,
      ) as HTMLElement | null;
      clearSharedElementName(image);
    }
    selectedProduct = null;
  });
}
```

**CSS:**

```css
:root {
  --hero-duration: 300ms;
  --hero-easing: cubic-bezier(0.4, 0, 0.2, 1);
}

::view-transition-group(product-hero) {
  animation-duration: var(--hero-duration);
  animation-timing-function: var(--hero-easing);
}

/* Prevent aspect ratio distortion during transition */
::view-transition-old(product-hero),
::view-transition-new(product-hero) {
  object-fit: cover;
}
```

**Why good:** Names set before transition, cleaned up after, handles both directions

---

## Multiple Shared Elements

### Good Example - Card with Multiple Animated Parts

```typescript
// multi-shared-element.ts

interface CardData {
  id: string;
  title: string;
  imageUrl: string;
  description: string;
}

const TRANSITION_NAMES = {
  image: (id: string) => `card-image-${id}`,
  title: (id: string) => `card-title-${id}`,
  container: (id: string) => `card-container-${id}`,
} as const;

function setMultipleTransitionNames(
  cardId: string,
  elements: {
    image?: HTMLElement | null;
    title?: HTMLElement | null;
    container?: HTMLElement | null;
  },
): void {
  if (elements.image) {
    elements.image.style.viewTransitionName = TRANSITION_NAMES.image(cardId);
  }
  if (elements.title) {
    elements.title.style.viewTransitionName = TRANSITION_NAMES.title(cardId);
  }
  if (elements.container) {
    elements.container.style.viewTransitionName =
      TRANSITION_NAMES.container(cardId);
  }
}

function clearMultipleTransitionNames(elements: (HTMLElement | null)[]): void {
  elements.forEach((el) => {
    if (el) {
      el.style.viewTransitionName = "";
    }
  });
}

export async function expandCard(cardId: string): Promise<void> {
  // Find source elements
  const sourceContainer = document.querySelector(
    `[data-card="${cardId}"]`,
  ) as HTMLElement | null;
  const sourceImage = sourceContainer?.querySelector(
    "[data-card-image]",
  ) as HTMLElement | null;
  const sourceTitle = sourceContainer?.querySelector(
    "[data-card-title]",
  ) as HTMLElement | null;

  const updateFn = () => {
    renderExpandedCard(cardId);

    // Set names on destination elements
    const destContainer = document.querySelector(
      `[data-card="${cardId}"]`,
    ) as HTMLElement | null;
    const destImage = destContainer?.querySelector(
      "[data-card-image]",
    ) as HTMLElement | null;
    const destTitle = destContainer?.querySelector(
      "[data-card-title]",
    ) as HTMLElement | null;

    setMultipleTransitionNames(cardId, {
      image: destImage,
      title: destTitle,
      container: destContainer,
    });
  };

  if (!document.startViewTransition) {
    updateFn();
    return;
  }

  // Set names on source elements
  setMultipleTransitionNames(cardId, {
    image: sourceImage,
    title: sourceTitle,
    container: sourceContainer,
  });

  const transition = document.startViewTransition(updateFn);

  await transition.finished;

  // Clean up all names
  const container = document.querySelector(
    `[data-card="${cardId}"]`,
  ) as HTMLElement | null;
  const image = container?.querySelector(
    "[data-card-image]",
  ) as HTMLElement | null;
  const title = container?.querySelector(
    "[data-card-title]",
  ) as HTMLElement | null;

  clearMultipleTransitionNames([container, image, title]);
}

function renderExpandedCard(cardId: string): void {
  // Render expanded card view
}
```

**CSS:**

```css
:root {
  --card-duration: 350ms;
  --card-easing: cubic-bezier(0.4, 0, 0.2, 1);
}

/* Style all card transitions with view-transition-class */
[style*="view-transition-name: card-"] {
  view-transition-class: card-element;
}

::view-transition-group(.card-element) {
  animation-duration: var(--card-duration);
  animation-timing-function: var(--card-easing);
}
```

**Why good:** Multiple elements transition together, factory functions for consistent naming, grouped CSS styling

---

## MPA Cross-Document Shared Elements

### Good Example - Product Gallery MPA

**list-page.html:**

```html
<!DOCTYPE html>
<html>
  <head>
    <style>
      @view-transition {
        navigation: auto;
      }

      .product-thumbnail {
        view-transition-name: product-hero;
      }
    </style>
    <script>
      // Set dynamic name before navigation
      window.addEventListener("pageswap", (e) => {
        if (!e.viewTransition) return;

        const targetUrl = new URL(e.activation.entry.url);
        const productId = targetUrl.searchParams.get("id");

        if (productId) {
          const thumbnail = document.querySelector(
            `[data-product="${productId}"] img`,
          );
          if (thumbnail) {
            thumbnail.style.viewTransitionName = "product-hero";
          }
        }

        e.viewTransition.finished.then(() => {
          // Cleanup happens on old page, but good practice
        });
      });
    </script>
  </head>
  <body>
    <ul class="product-grid">
      <li data-product="1">
        <a href="/product.html?id=1">
          <img src="/images/product-1.jpg" alt="Product 1" />
        </a>
      </li>
      <li data-product="2">
        <a href="/product.html?id=2">
          <img src="/images/product-2.jpg" alt="Product 2" />
        </a>
      </li>
    </ul>
  </body>
</html>
```

**product-page.html:**

```html
<!DOCTYPE html>
<html>
  <head>
    <style>
      :root {
        --hero-duration: 300ms;
        --hero-easing: cubic-bezier(0.4, 0, 0.2, 1);
      }

      @view-transition {
        navigation: auto;
      }

      .product-hero {
        view-transition-name: product-hero;
      }

      ::view-transition-group(product-hero) {
        animation-duration: var(--hero-duration);
        animation-timing-function: var(--hero-easing);
      }
    </style>
    <script>
      // Register early for pagereveal
      window.addEventListener("pagereveal", (e) => {
        if (!e.viewTransition) return;

        const heroImage = document.querySelector(".product-hero");
        if (heroImage) {
          heroImage.style.viewTransitionName = "product-hero";
        }

        e.viewTransition.ready.then(() => {
          // Names can be removed after snapshots taken
          heroImage?.style.removeProperty("view-transition-name");
        });
      });
    </script>
  </head>
  <body>
    <article>
      <img class="product-hero" src="/images/product-1.jpg" alt="Product 1" />
      <h1>Product Name</h1>
      <p>Description...</p>
      <a href="/list.html">Back to list</a>
    </article>
  </body>
</html>
```

**Why good:** Names set on both pages, pageswap on source and pagereveal on destination, cleanup after ready

---

## Dynamic Name Assignment with match-element

### Good Example - Auto-Naming for Lists (Chrome 137+)

```css
:root {
  --product-transition-duration: 300ms;
}

/* Browser generates unique internal names */
.product-card {
  view-transition-name: match-element;
  view-transition-class: product;
}

/* Style all product transitions together */
::view-transition-group(.product) {
  animation-duration: var(--product-transition-duration);
  animation-timing-function: ease-out;
}
```

**Why good:** No manual naming for large lists, browser tracks element identity

### Good Example - Dynamic Names with attr() (Chrome 133+)

```html
<div class="card" id="product-001">Product 1</div>
<div class="card" id="product-002">Product 2</div>
```

```css
.card[id] {
  view-transition-name: attr(id type(<custom-ident>), none);
  view-transition-class: card;
}

::view-transition-group(.card) {
  animation-duration: var(--product-transition-duration);
}
```

**Why good:** Uses element ID as transition name automatically, fallback to none

---

## Scoped Shared Elements

### Good Example - Isolated Card Transitions

```typescript
// scoped-transitions.ts

const CARD_SCOPES = new Map<string, string>();

function generateScopeId(): string {
  return `scope-${Math.random().toString(36).slice(2, 9)}`;
}

function getScopedName(baseId: string, name: string): string {
  let scopeId = CARD_SCOPES.get(baseId);
  if (!scopeId) {
    scopeId = generateScopeId();
    CARD_SCOPES.set(baseId, scopeId);
  }
  return `${name}-${scopeId}`;
}

export function transitionScopedCard(
  cardId: string,
  updateFn: () => void,
): void {
  const scopedName = getScopedName(cardId, "card-hero");

  const sourceImage = document.querySelector(
    `[data-card="${cardId}"] img`,
  ) as HTMLElement | null;

  if (sourceImage) {
    sourceImage.style.viewTransitionName = scopedName;
  }

  if (!document.startViewTransition) {
    updateFn();
    return;
  }

  const transition = document.startViewTransition(() => {
    updateFn();

    const destImage = document.querySelector(
      `[data-card="${cardId}"] img`,
    ) as HTMLElement | null;

    if (destImage) {
      destImage.style.viewTransitionName = scopedName;
    }
  });

  transition.finished.then(() => {
    const image = document.querySelector(
      `[data-card="${cardId}"] img`,
    ) as HTMLElement | null;

    if (image) {
      image.style.viewTransitionName = "";
    }
  });
}
```

**Why good:** Scoped names prevent conflicts between multiple card instances

---

## Modal with Shared Origin

### Good Example - Button to Modal Expansion

```typescript
// modal-shared-transition.ts

const MODAL_ORIGIN_NAME = "modal-origin";
const MODAL_DURATION_MS = 300;

let currentOriginElement: HTMLElement | null = null;

export function openModalFromElement(
  triggerElement: HTMLElement,
  modalContent: string,
): void {
  currentOriginElement = triggerElement;

  const updateFn = () => {
    // Render modal
    const modal = document.createElement("div");
    modal.className = "modal-overlay";
    modal.innerHTML = `
      <div class="modal-content" data-modal-content>
        ${modalContent}
        <button data-close-modal>Close</button>
      </div>
    `;
    document.body.appendChild(modal);

    // Set name on modal content
    const modalContentEl = modal.querySelector(
      "[data-modal-content]",
    ) as HTMLElement;
    if (modalContentEl) {
      modalContentEl.style.viewTransitionName = MODAL_ORIGIN_NAME;
    }
  };

  if (!document.startViewTransition) {
    updateFn();
    return;
  }

  // Set name on trigger before transition
  triggerElement.style.viewTransitionName = MODAL_ORIGIN_NAME;

  const transition = document.startViewTransition(updateFn);

  transition.finished.then(() => {
    // Clean up origin name
    triggerElement.style.viewTransitionName = "";
  });
}

export function closeModal(): void {
  const modalContentEl = document.querySelector(
    "[data-modal-content]",
  ) as HTMLElement;
  const modal = document.querySelector(".modal-overlay");

  if (!modal) return;

  const updateFn = () => {
    modal.remove();
  };

  if (!document.startViewTransition || !currentOriginElement) {
    updateFn();
    currentOriginElement = null;
    return;
  }

  // Set name on origin element for return transition
  currentOriginElement.style.viewTransitionName = MODAL_ORIGIN_NAME;

  if (modalContentEl) {
    modalContentEl.style.viewTransitionName = MODAL_ORIGIN_NAME;
  }

  const transition = document.startViewTransition(updateFn);

  transition.finished.then(() => {
    if (currentOriginElement) {
      currentOriginElement.style.viewTransitionName = "";
    }
    currentOriginElement = null;
  });
}
```

**CSS:**

```css
:root {
  --modal-duration: 300ms;
}

::view-transition-group(modal-origin) {
  animation-duration: var(--modal-duration);
  animation-timing-function: cubic-bezier(0.2, 0, 0, 1);
}

.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: grid;
  place-items: center;
}

.modal-content {
  background: white;
  padding: 2rem;
  border-radius: 8px;
  max-width: 500px;
}
```

**Why good:** Tracks origin element for return transition, cleans up names bidirectionally
