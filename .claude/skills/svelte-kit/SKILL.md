---
name: svelte-kit
description: Svelte 5 and SvelteKit syntax expert. Use when working with .svelte files, runes syntax ($state, $derived, $effect), SvelteKit routing, SSR, or component design.
---

# Svelte/SvelteKit Expert

Expert assistant for Svelte 5 runes syntax, SvelteKit routing, SSR/SSG strategies, and component design patterns.

## Thinking Process

When activated, follow this structured thinking approach to solve Svelte/SvelteKit problems:

### Step 1: Problem Classification

**Goal:** Understand what type of Svelte challenge this is.

**Key Questions to Ask:**
- Is this a reactivity problem? (state updates not reflecting, derived values)
- Is this a rendering problem? (SSR vs CSR, hydration mismatch)
- Is this a routing problem? (navigation, params, layouts)
- Is this a data loading problem? (load functions, form actions)
- Is this a component design problem? (props, slots, events)

**Decision Point:** Classify to select appropriate solutions:
- Reactivity → Check runes usage ($state, $derived, $effect)
- Rendering → Consider SSR/CSR implications
- Routing → Review SvelteKit conventions
- Data Loading → Differentiate +page.ts vs +page.server.ts
- Components → Apply composition patterns

### Step 2: Version and Context Check

**Goal:** Ensure solutions match the project's Svelte version.

**Key Questions to Ask:**
- Is this Svelte 5 (runes) or Svelte 4 (stores)?
- What SvelteKit version is in use?
- What rendering mode is configured? (SSR, SPA, SSG)

**Actions:**
1. Check `package.json` for svelte and @sveltejs/kit versions
2. Look for `svelte.config.js` adapter configuration
3. Note any prerender settings

**Version-Specific Syntax:**
| Concept | Svelte 4 | Svelte 5 |
|---------|----------|----------|
| Reactive state | `let x = 0` | `let x = $state(0)` |
| Derived | `$: doubled = x * 2` | `let doubled = $derived(x * 2)` |
| Effects | `$: console.log(x)` | `$effect(() => console.log(x))` |
| Props | `export let name` | `let { name } = $props()` |

**Decision Point:** Always default to Svelte 5 runes syntax unless explicitly working with Svelte 4.

### Step 3: SSR/CSR Analysis

**Goal:** Understand the rendering context and its implications.

**Thinking Framework:**
- "When does this code run?" (server, client, or both)
- "What data is available at each stage?"
- "Could this cause a hydration mismatch?"

**SSR Decision Matrix:**

| Code Location | Runs On | Use For |
|---------------|---------|---------|
| +page.server.ts | Server only | DB access, secrets, auth |
| +page.ts | Server + Client | Public API calls, URL-dependent data |
| +page.svelte | Server + Client | UI rendering |
| $effect() | Client only | DOM manipulation, subscriptions |

**Common SSR Pitfalls:**
- Browser APIs (window, document) in SSR context
- Different content between server and client render
- Accessing cookies/headers incorrectly

**SSR Safety Pattern:**
```svelte
<script>
  import { browser } from '$app/environment';

  $effect(() => {
    if (browser) {
      // Safe to use browser APIs here
    }
  });
</script>
```

### Step 4: Data Flow Design

**Goal:** Design correct data loading and mutation patterns.

**Thinking Framework:**
- "Where does this data come from?" (server, client, URL)
- "When should it be fetched?" (navigation, action, interval)
- "Who can access this data?" (public, authenticated, authorized)

**Load Function Selection:**

| Need | Use | Why |
|------|-----|-----|
| Access secrets/DB | +page.server.ts | Never exposed to client |
| Public API call | +page.ts | Runs on both, good for caching |
| SEO-critical data | +page.server.ts | Guaranteed in initial HTML |
| Client-side only | fetch in $effect | Avoid SSR overhead |

**Form Action Thinking:**
- "What mutation does this form perform?"
- "What validation is needed?"
- "What should happen on success/failure?"

### Step 5: Reactivity Design

**Goal:** Apply correct reactivity patterns for the use case.

**Thinking Framework - Runes Selection:**

| Need | Rune | Example |
|------|------|---------|
| Mutable state | $state | `let count = $state(0)` |
| Computed value | $derived | `let double = $derived(count * 2)` |
| Side effects | $effect | `$effect(() => save(data))` |
| Component props | $props | `let { name } = $props()` |
| Two-way binding | $bindable | `let { value = $bindable() } = $props()` |

**Reactivity Rules:**
1. Only use `$state` for values that need to trigger updates
2. Use `$derived` for any computed values (not manual updates)
3. Use `$effect` sparingly - prefer declarative patterns
4. Never mutate $derived values

**Common Mistakes:**
```svelte
<script>
  // WRONG: Derived values should use $derived
  let count = $state(0);
  let doubled = count * 2; // Won't update when count changes!

  // RIGHT: Use $derived for computed values
  let doubled = $derived(count * 2);
</script>
```

### Step 6: Component Design

**Goal:** Design reusable, composable components.

**Thinking Framework:**
- "What is the single responsibility of this component?"
- "What props does it need?"
- "How flexible should slot composition be?"

**Component Interface Design:**
```svelte
<script>
  // Required props
  let { title, items } = $props();

  // Optional props with defaults
  let { variant = 'default', disabled = false } = $props();

  // Callback props
  let { onClick = () => {} } = $props();

  // Bindable props for two-way binding
  let { value = $bindable() } = $props();
</script>
```

**Slot Patterns:**
- Default slot: Main content area
- Named slots: Header, footer, sidebar
- Slot props: Passing data to slot content

### Step 7: Performance Optimization

**Goal:** Ensure optimal rendering performance.

**Thinking Framework:**
- "How often does this reactive value change?"
- "What is the cost of re-rendering?"
- "Can this be memoized or debounced?"

**Performance Checklist:**
- [ ] Avoid expensive computations in $derived
- [ ] Use {#key} block for forced re-renders
- [ ] Implement virtualization for long lists
- [ ] Lazy load heavy components
- [ ] Preload critical routes

### Step 8: Error Handling

**Goal:** Provide good error experiences.

**Error Boundaries:**
- +error.svelte for route-level errors
- try/catch in load functions
- Form action error handling

**Error Pattern:**
```typescript
// +page.server.ts
export async function load({ params }) {
  const item = await db.get(params.id);
  if (!item) {
    throw error(404, 'Item not found');
  }
  return { item };
}
```

## Project Setup

**Preferred Package Manager:** bun

```bash
# Create new SvelteKit project
bunx sv create my-app
cd my-app
bun install
bun run dev
```

## Documentation Resources

**Context7 Library ID:** `/websites/svelte_dev` (5523 snippets, Score: 91)

**Official llms.txt Resources:**
- `https://svelte.dev/docs/llms` - Documentation index
- `https://svelte.dev/docs/llms-full.txt` - Complete documentation
- `https://svelte.dev/docs/llms-small.txt` - Compressed (~120KB)

## Quick Reference

### Svelte 5 Runes

```svelte
<script>
  // Reactive state
  let count = $state(0);

  // Derived values (auto-updates when dependencies change)
  let doubled = $derived(count * 2);

  // Side effects
  $effect(() => {
    console.log(`Count is now ${count}`);
  });

  // Props with defaults
  let { name = 'World', onClick } = $props();

  // Bindable props (two-way binding)
  let { value = $bindable() } = $props();
</script>
```

### SvelteKit Routing

```
src/routes/
├── +page.svelte          # /
├── +page.server.ts       # Server load function
├── +layout.svelte        # Root layout
├── about/+page.svelte    # /about
├── blog/
│   ├── +page.svelte      # /blog
│   └── [slug]/
│       ├── +page.svelte  # /blog/:slug
│       └── +page.ts      # Universal load
└── api/posts/+server.ts  # API endpoint
```

### Load Functions

```typescript
// +page.server.ts - Server-only
export async function load({ params, locals, fetch }) {
  const post = await fetch(`/api/posts/${params.slug}`);
  return { post: await post.json() };
}

// +page.ts - Universal (server + client)
export async function load({ params, fetch }) {
  const res = await fetch(`/api/posts/${params.slug}`);
  return { post: await res.json() };
}
```

### Form Actions

```typescript
// +page.server.ts
export const actions = {
  default: async ({ request }) => {
    const data = await request.formData();
    const email = data.get('email');
    return { success: true };
  },
  delete: async ({ params }) => {
    // Handle delete
  }
};
```

## Present Results to User

When answering Svelte/SvelteKit questions:
- Provide complete, runnable code examples
- Use Svelte 5 runes syntax by default
- Explain the difference between server and universal load functions
- Note any breaking changes between SvelteKit versions
- Include TypeScript types when applicable

## Troubleshooting

**"Cannot use $state outside of component"**
- Runes only work inside `.svelte` files or `.svelte.ts` files

**"Hydration mismatch"**
- Ensure server and client render the same content initially
- Check for browser-only code running during SSR

**"Load function not running"**
- Verify file naming: `+page.ts` or `+page.server.ts`
- Check if `load` function is properly exported
