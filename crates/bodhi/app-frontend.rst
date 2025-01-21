Bodhi Frontend Documentation
==========================

Project Overview
---------------
Bodhi is a Next.js application built with TypeScript that provides a web interface for running LLMs (Large Language Models) locally. The project uses the App Router pattern of Next.js and follows modern React practices.

Core Technologies
---------------

Framework & Runtime
~~~~~~~~~~~~~~~~~
- **Next.js** (v14.2.6): React framework with App Router
- **React** (v18): UI library
- **TypeScript**: Programming language
- **Node.js**: Runtime environment

UI Components & Styling
~~~~~~~~~~~~~~~~~~~~~
- **Tailwind CSS**: Utility-first CSS framework
- **Shadcn/ui**: Component library built on Radix UI
- **Class Variance Authority (CVA)**: Component variant management
- **clsx/tailwind-merge**: Class name management utilities
- **Lucide React**: Icon library
- **Framer Motion**: Animation library

Data Management & API
~~~~~~~~~~~~~~~~~~~
- **React Query**: Data fetching and state management
- **Axios**: HTTP client
- **React Hook Form**: Form management
- **Zod**: Schema validation

Testing
~~~~~~~
- **Vitest**: Testing framework
- **Testing Library**: Testing utilities
- **MSW (Mock Service Worker)**: API mocking
- **Happy DOM**: DOM environment for testing

Development Tools
~~~~~~~~~~~~~~~
- **ESLint**: Code linting
- **Prettier**: Code formatting
- **Husky**: Git hooks
- **lint-staged**: Staged files linting

Project Structure
---------------

The project follows a standard Next.js App Router structure::

    crates/bodhi/
    ├── src/
    │   ├── app/                 # Next.js App Router pages
    │   ├── components/          # React components
    │   ├── hooks/              # Custom React hooks
    │   ├── lib/                # Utility functions
    │   ├── styles/             # Global styles
    │   └── types/              # TypeScript type definitions
    ├── public/                 # Static assets
    └── tests/                  # Test utilities and setup

Page Organization
~~~~~~~~~~~~~~~~
The project follows a co-location pattern for page-specific components. Each page directory can contain::

    src/app/ui/page-name/
    ├── page.tsx               # Main page component
    ├── page.test.tsx         # Page tests
    ├── ComponentA.tsx        # Page-specific components
    ├── ComponentA.test.tsx   # Component tests
    └── types.ts              # Page-specific types

Example from tokens page::

    src/app/ui/tokens/
    ├── page.tsx              # Main tokens page
    ├── page.test.tsx        # Page tests
    ├── TokenDialog.tsx      # Token display dialog
    ├── TokenDialog.test.tsx # Dialog tests
    ├── TokenForm.tsx        # Token creation form
    └── TokenForm.test.tsx   # Form tests

This organization:

- Keeps related code close together
- Makes it easy to find components specific to a page
- Improves maintainability by grouping related files
- Allows for better code splitting
- Simplifies testing related components

Coding Conventions
----------------

Component Structure
~~~~~~~~~~~~~~~~~
- Use functional components with TypeScript
- Follow the component-per-file pattern
- Place tests alongside components with ``.test.tsx`` extension
- Use named exports for components

Example component structure::

    export function ComponentName({ prop1, prop2 }: ComponentNameProps) {
      // Component logic
      return (
        // JSX
      )
    }

File Naming
~~~~~~~~~~
- Use kebab-case for file names: ``my-component.tsx``
- Use PascalCase for component names: ``MyComponent``
- Test files: ``my-component.test.tsx``
- Type files: ``my-component.types.ts``

Styling Conventions
~~~~~~~~~~~~~~~~
- Use Tailwind CSS classes for styling
- Follow utility-first CSS approach
- Use ``class:`` syntax for conditional classes
- Leverage ``cn()`` utility for class name merging

Example styling::

    <div
      className={cn(
        "flex items-center p-4",
        isActive && "bg-primary text-white"
      )}
    >

State Management
~~~~~~~~~~~~~~
- Use React Query for server state
- Use React hooks for local state
- Follow the container/presenter pattern
- Keep state as close to where it's used as possible

Form Handling
~~~~~~~~~~~
- Use React Hook Form for form state management
- Use Zod for schema validation
- Leverage shadcn/ui form components
- Follow controlled component pattern

Form Structure
'''''''''''''
Example form setup with validation::

    const createTokenSchema = z.object({
      name: z.string().optional()
    });
    
    export function TokenForm() {
      const form = useForm<TokenFormData>({
        resolver: zodResolver(createTokenSchema),
        mode: 'onSubmit',
        defaultValues: {
          name: '',
        },
      });
    }

Form Components
'''''''''''''
Use shadcn/ui form components for consistent styling::

    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)}>
        <FormField
          control={form.control}
          name="name"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Token Name</FormLabel>
              <FormControl>
                <Input {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
      </form>
    </Form>

Form Submission
'''''''''''''
Handle form submission with error handling::

    const onSubmit = async (data: FormData) => {
      try {
        await submitData(data);
        form.reset();
        toast({
          title: 'Success',
          description: 'Form submitted successfully'
        });
      } catch (error) {
        toast({
          title: 'Error',
          description: 'Failed to submit form',
          variant: 'destructive'
        });
      }
    };

Form Testing
''''''''''
Test form validation and submission::

    it('handles form submission', async () => {
      const user = userEvent.setup();
      
      render(<TokenForm />);
      
      await user.type(
        screen.getByLabelText('Name'),
        'Test'
      );
      
      await user.click(
        screen.getByRole('button', { name: 'Submit' })
      );
      
      expect(onSubmit).toHaveBeenCalledWith({
        name: 'Test'
      });
    });

Network & API Conventions
------------------------

API Client Structure
~~~~~~~~~~~~~~~~~
- Centralized API endpoint definitions in ``useQuery.ts``
- Custom wrapper around React Query and Axios
- Base API URL configuration with ``BODHI_API_BASE``
- Standardized error handling and response types

Query Hooks Pattern
~~~~~~~~~~~~~~~~
- Use custom ``useQuery`` hook for GET requests::

    export function useModelFiles(page?: number, pageSize?: number) {
      return useQuery<PagedApiResponse<ModelFile[]>>(
        ['modelFiles', page?.toString()],
        ENDPOINT_MODEL_FILES,
        { page, page_size: pageSize }
      );
    }

- Use ``useMutationQuery`` for POST/PUT/DELETE operations::

    export function useCreateToken() {
      return useMutationQuery<TokenResponse, CreateTokenRequest>(
        API_TOKENS_ENDPOINT,
        'post',
        {
          onSuccess: () => {
            queryClient.invalidateQueries(['tokens']);
          }
        }
      );
    }

Response Types
~~~~~~~~~~~~
- Standardized paged response interface::

    type PagedApiResponse<T> = {
      data: T;
      total?: number;
      page?: number;
      page_size?: number;
    }

- Strong typing for all API responses
- Consistent error type handling with ``AxiosError``

Cache Management
~~~~~~~~~~~~~
- React Query for client-side caching
- Automatic cache invalidation on mutations
- Configurable cache time and stale time
- Query key conventions for cache management

Testing Network Calls
~~~~~~~~~~~~~~~~~~
- Mock Service Worker (MSW) for API mocking
- Test server setup with common endpoints
- Test both success and error scenarios
- Verify cache invalidation

MSW Server Setup
'''''''''''''''
- Mock Service Worker (MSW) for API mocking
- Test server setup with common endpoints
- Test both success and error scenarios
- Verify cache invalidation

Example MSW server setup::

    const server = setupServer(
      rest.get(`*${API_TOKENS_ENDPOINT}`, (_, res, ctx) => {
        return res(ctx.status(200), ctx.json(mockListResponse));
      }),
      rest.post(`*${API_TOKENS_ENDPOINT}`, (_, res, ctx) => {
        return res(ctx.status(201), ctx.json(mockTokenResponse));
      })
    );
    
    beforeAll(() => server.listen());
    afterAll(() => server.close());
    afterEach(() => server.resetHandlers());

User Interaction Testing
'''''''''''''''''''''''
- Use ``userEvent`` from @testing-library/user-event
- Setup user events at the start of each test
- Simulate real user interactions

Example user interaction test::

    describe('TokenForm', () => {
      it('handles form submission', async () => {
        const user = userEvent.setup();
        
        render(<TokenForm onTokenCreated={onTokenCreated} />);
        
        await user.type(
          screen.getByLabelText('Token Name'),
          'Test Token'
        );
        await user.click(
          screen.getByRole('button', { name: 'Generate' })
        );
      });
    });

Loading States
'''''''''''''
- Test initial loading states
- Verify loading indicators
- Test skeleton loaders

Example loading state test::

    it('shows loading skeleton initially', () => {
      render(<TokenPageContent />);
      expect(screen.getByTestId('token-page-loading'))
        .toBeInTheDocument();
    });

Error Handling
'''''''''''''
- Test API error responses
- Verify error messages
- Test error UI states

Example error test::

    it('handles api error', async () => {
      server.use(
        rest.post(`*${API_TOKENS_ENDPOINT}`, (_, res, ctx) => {
          return res(
            ctx.status(400),
            ctx.json({ message: 'Failed to generate token' })
          );
        })
      );
      
      await user.click(screen.getByRole('button'));
      
      expect(mockToast).toHaveBeenCalledWith({
        title: 'Error',
        description: 'Failed to generate token',
        variant: 'destructive'
      });
    });

Example test pattern::

    describe('useCreateToken', () => {
      it('creates token and invalidates cache', async () => {
        const { result } = renderHook(() => useCreateToken(), {
          wrapper: createWrapper()
        });
        
        await act(async () => {
          await result.current.mutateAsync({ name: 'Test' });
        });
        
        // Verify cache invalidation
        expect(queryClient.invalidateQueries).toHaveBeenCalledWith(['tokens']);
      });
    }); 

Testing Conventions
~~~~~~~~~~~~~~~~
- Write tests for all components and hooks
- Use React Testing Library for component testing
- Use MSW for API mocking
- Follow AAA pattern (Arrange, Act, Assert)
- Test user interactions and accessibility

Example test structure::

    describe('ComponentName', () => {
      it('should render successfully', () => {
        render(<ComponentName />)
        expect(screen.getByRole('button')).toBeInTheDocument()
      })
    })

Error Handling
~~~~~~~~~~~~
- Use try/catch blocks for async operations
- Implement error boundaries for component errors
- Display user-friendly error messages
- Log errors appropriately

Accessibility
~~~~~~~~~~~
- Follow WCAG guidelines
- Use semantic HTML elements
- Implement proper ARIA attributes
- Ensure keyboard navigation
- Test with screen readers

Performance Considerations
~~~~~~~~~~~~~~~~~~~~~~~
- Use React.memo for expensive components
- Implement proper code splitting
- Optimize images and assets
- Monitor bundle size
- Use proper caching strategies

Git Workflow
~~~~~~~~~~
- Use feature branches
- Follow conventional commits
- Run linting before commits (husky)
- Ensure all tests pass before merging
- Keep PRs focused and small

