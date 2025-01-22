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

Mutation Pattern
~~~~~~~~~~~~~
The application follows a consistent pattern for handling mutations using react-query. This pattern
provides better error handling, type safety, and separation of concerns.

Hook Definition Pattern
'''''''''''''''''''''''
Mutation hooks should be defined with callback options::

    export function useSomeMutation(options?: {
      onSuccess?: (response: ResponseType) => void;
      onError?: (message: string) => void;
    }): UseMutationResult<
      AxiosResponse<ResponseType>,
      AxiosError<ErrorResponse>,
      RequestType
    > {
      const queryClient = useQueryClient();
      return useMutationQuery<ResponseType, RequestType>(
        ENDPOINT,
        'post',
        {
          onSuccess: (response) => {
            queryClient.invalidateQueries(['queryKey']);
            options?.onSuccess?.(response.data);
          },
          onError: (error: AxiosError<ErrorResponse>) => {
            const message =
              error?.response?.data?.error?.message || 'Failed to perform action';
            options?.onError?.(message);
          },
        }
      );
    }

Component Usage Pattern
''''''''''''''''''''''
Components should use mutations by providing callbacks::

    const { mutate, isLoading } = useSomeMutation({
      onSuccess: (response) => {
        toast({
          title: 'Success',
          description: 'Operation completed successfully',
        });
        // Additional success handling
      },
      onError: (message) => {
        toast({
          title: 'Error',
          description: message,
          variant: 'destructive',
        });
        // Additional error handling
      },
    });

    const handleAction = (data: RequestType) => {
      mutate(data);
    };

Benefits of this Pattern
''''''''''''''''''''''''
- Consistent error handling across the application
- Type-safe callbacks
- Clear separation of concerns
- Simpler component code
- Centralized error message handling
- Automatic query invalidation
- Better maintainability

Example Implementation
'''''''''''''''''''''
Here's a complete example with a mutation hook and its usage::

    // Hook definition
    export function useCreateToken(options?: {
      onSuccess?: (response: TokenResponse) => void;
      onError?: (message: string) => void;
    }): UseMutationResult<
      AxiosResponse<TokenResponse>,
      AxiosError<ErrorResponse>,
      CreateTokenRequest
    > {
      const queryClient = useQueryClient();
      return useMutationQuery<TokenResponse, CreateTokenRequest>(
        API_TOKENS_ENDPOINT,
        'post',
        {
          onSuccess: (response) => {
            queryClient.invalidateQueries(['tokens']);
            options?.onSuccess?.(response.data);
          },
          onError: (error: AxiosError<ErrorResponse>) => {
            const message =
              error?.response?.data?.error?.message || 'Failed to generate token';
            options?.onError?.(message);
          },
        }
      );
    }

    // Component usage
    export function TokenForm({ onTokenCreated }: TokenFormProps) {
      const { toast } = useToast();
      const { mutate: createToken, isLoading } = useCreateToken({
        onSuccess: (response) => {
          onTokenCreated(response);
          form.reset();
          toast({
            title: 'Success',
            description: 'API token successfully generated',
          });
        },
        onError: (message) => {
          toast({
            title: 'Error',
            description: message,
            variant: 'destructive',
          });
        },
      });

      const onSubmit = (data: FormData) => {
        createToken(data);
      };
    }

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


API Error Format
''''''''''''''''
The application expects API errors in a consistent format::

    interface ApiError {
      error: {
        message: string;
      }
    }

Error handling patterns:

1. API Client Level (apiClient.ts)::

    apiClient.interceptors.response.use(
      (response) => response,
      (error) => {
        console.error('Error:', error.response?.status, error.config?.url);
        return Promise.reject(error);
      }
    );

2. Hook Level::
    // Hooks pass through the error to be handled by components
    export function useMutationQuery<T, V>(
      endpoint: string | ((variables: V) => string),
      method: 'post' | 'put' | 'delete' = 'post',
      options?: UseMutationOptions<AxiosResponse<T>, AxiosError, V>
    ) {
      // ... mutation logic
    }

3. Component Level::

    try {
      await mutation.mutateAsync(data);
    } catch (error) {
      // Components handle the error structure from the API
      const message = error?.response?.data?.error?.message || "Operation failed";
      toast({
        title: "Error",
        description: message,
        variant: "destructive"
      });
    }

Error Handling Patterns
~~~~~~~~~~~~~~~~~~~~~
There are two main patterns for handling API errors in the application:

1. Using useQuery (Automatic Error Handling)
''''''''''''''''''''''''''''''''''''''''''
The error is automatically returned as part of UseQueryResult::

    const { data, error, isLoading } = useQuery<DataType>(...);
    
    if (error) {
      // Handle error state in UI
      return <ErrorComponent message={error.message} />;
    }

This pattern is used for:
- Read operations
- Automatically retried operations
- Declarative data fetching
- Operations that happen on component mount

2. Using useMutationQuery (Manual Error Handling)
''''''''''''''''''''''''''''''''''''''''''''''
Mutations require explicit error handling using try/catch or callbacks::

    // Using try/catch with mutateAsync
    const handleSubmit = async (formData: FormData) => {
      try {
        await mutation.mutateAsync(formData);
        toast.success("Operation successful");
      } catch (error) {
        if (error instanceof Error) {
          setErrorMessage(error.message);
        }
      }
    };

    // Using callbacks
    const mutation = useMutationQuery({
      onSuccess: () => {
        toast.success("Operation successful");
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        if (error.response?.data?.error?.message) {
          setErrorMessage(error.response.data.error.message);
        }
      }
    });

This pattern is used for:
- Write operations
- User-triggered actions
- Operations needing UI feedback
- Operations that may need to roll back changes

Mutation States
'''''''''''''
Available mutation states for error handling::

    mutation.isLoading  // Is the mutation in progress?
    mutation.isError    // Did the mutation error?
    mutation.error      // The error object if present
    mutation.isSuccess  // Did the mutation succeed?

Best Practices
''''''''''''
- Use try/catch with mutateAsync for complex flows
- Use callbacks for simple success/error handling
- Always show user-friendly error messages
- Handle network errors gracefully
- Implement proper error boundaries
- Log errors appropriately for debugging

Testing Error Responses
''''''''''''''''''''''
Mock error responses in tests following the API error format::

    server.use(
      rest.put('*/endpoint', (_, res, ctx) => {
        return res(
          ctx.status(500),
          ctx.json({
            error: {
              message: 'Server error'
            }
          })
        );
      })
    );

Testing Best Practices
~~~~~~~~~~~~~~~~~~~
Component Testing Patterns
''''''''''''''''''''''''''
1. Mock child components when testing parent components::

    vi.mock('./ChildComponent', () => ({
      ChildComponent: ({ prop, onAction }: any) => (
        <div data-testid="mock-child">
          <span>Prop: {prop}</span>
          <button onClick={onAction}>Action</button>
        </div>
      )
    }));

2. Test component behavior, not implementation::

    // Good
    expect(screen.getByRole('button')).toBeInTheDocument();
    
    // Avoid
    expect(screen.getByTestId('specific-div')).toHaveClass('specific-class');

3. Use proper query priorities::

    // Priority order:
    // 1. getByRole
    // 2. getByLabelText
    // 3. getByText
    // 4. getByTestId

Toast Notification Testing
''''''''''''''''''''''''''
Mock toast notifications instead of testing DOM content::

    const mockToast = vi.fn();
    vi.mock('@/hooks/use-toast', () => ({
      useToast: () => ({ toast: mockToast })
    }));

    expect(mockToast).toHaveBeenCalledWith({
      title: "Success",
      description: "Operation completed"
    });

UI Component Testing
'''''''''''''''''''
1. Test complex UI interactions with proper setup::

    // For Radix UI / ShadCN components
    Object.assign(window.HTMLElement.prototype, {
      scrollIntoView: vi.fn(),
      releasePointerCapture: vi.fn(),
      hasPointerCapture: vi.fn(),
    });

2. Use findBy* for async rendering::

    const listbox = await screen.findByRole('listbox');

3. Use within for scoped queries::

    const option = within(listbox).getByRole('option');

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

Toast Notification Testing
'''''''''''''''''''''''''
When testing components that use toast notifications, mock the toast hook instead
of checking for toast content in the DOM. This makes tests more reliable and faster::

    // Mock toast hook
    const mockToast = vi.fn();
    vi.mock('@/hooks/use-toast', () => ({
      useToast: () => ({
        toast: mockToast
      })
    }));

    // In your test
    it('shows success toast', async () => {
      await user.click(screen.getByRole('button'));
      
      expect(mockToast).toHaveBeenCalledWith({
        title: "Success",
        description: "Operation completed",
        variant: "default"
      });
    });

Benefits of mocking toast:
- Tests are more reliable (no waiting for toast animations)
- Faster test execution
- Clear verification of toast parameters
- No need for async waitFor calls

Component Testing Best Practices
'''''''''''''''''''''''''''''''
- Use ``findByRole`` instead of ``getByRole`` when element might not be immediately available
- Use ``within`` to scope element queries to a specific container
- Prefer role-based queries over text-based queries for better accessibility testing
- Mock complex UI libraries (like toast, dialogs) instead of testing their DOM presence
- Use proper cleanup in afterEach to prevent test interference

Example of scoped queries::

    const dialog = screen.getByRole('dialog');
    const submitButton = within(dialog).getByRole('button', { name: /submit/i });

This approach:
- Makes tests more reliable and maintainable
- Follows testing best practices
- Improves test readability
- Ensures proper component isolation

MSW Server Setup
'''''''''''''''
- Mock Service Worker (MSW) for API mocking
- Test server setup with common endpoints
- Test both success and error scenarios
- Verify cache invalidation

Example MSW server setup::

    // Import from msw/node for Node.js environment testing
    import { setupServer } from 'msw/node';
    
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

// Note: Always use msw/node instead of msw/browser for Vitest/Jest tests
// as they run in a Node.js environment

User Interaction Testing
'''''''''''''''''''''''
- Use ``userEvent`` from @testing-library/user-event
- Setup user events at the start of each test
- Simulate real user interactions

Testing Radix UI Components
'''''''''''''''''''''''''''
When testing components that use Radix UI (like shadcn's Select), special setup is needed
to handle pointer events and HTML element methods. Here's the required setup::

    // Mock PointerEvent for Radix UI components
    function createMockPointerEvent(
      type: string,
      props: PointerEventInit = {}
    ): PointerEvent {
      const event = new Event(type, props) as PointerEvent;
      Object.assign(event, {
        button: props.button ?? 0,
        ctrlKey: props.ctrlKey ?? false,
        pointerType: props.pointerType ?? "mouse",
      });
      return event;
    }

    // Assign mock to window
    window.PointerEvent = createMockPointerEvent as any;

    // Mock required HTMLElement methods
    Object.assign(window.HTMLElement.prototype, {
      scrollIntoView: vi.fn(),
      releasePointerCapture: vi.fn(),
      hasPointerCapture: vi.fn(),
    });

This setup is necessary because:
- Radix UI uses PointerEvent API which isn't available in test environment
- Components like Select need pointer capture methods for interaction
- The mocks allow proper event handling in tests

Example testing Select component::

    it('updates select value', async () => {
      const user = userEvent.setup();
      
      render(<SelectComponent />);
      
      // Open select dropdown
      await user.click(screen.getByRole('combobox'));
      
      // Find and click option
      const listbox = await screen.findByRole('listbox');
      const option = within(listbox).getByRole('option', { name: /option/i });
      await user.click(option);
    });

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
