# Application Navigation Design

## Information Architecture

### 1. Core Application Structure
```
Bodhi App
├── Home
│   ├── Feature Highlights
│   ├── Learning Resources
│   └── News & Updates
├── Chat
│   ├── Chat Interface
│   ├── Chat History
│   └── Settings
├── Models
│   ├── Model Aliases
│   ├── Model Files
│   └── Downloads
└── System
    ├── Setup
    ├── User Management
    └── Settings
```

### 2. Feature Organization

#### Primary Features
1. **Chat System**
   - Active chat
   - Chat history
   - Model settings

2. **Model Management**
   - Alias configuration
   - File management
   - Downloads

3. **Learning Hub**
   - Resources
   - Tutorials
   - News

#### Administrative Features
1. **User Management**
   - Access requests
   - User administration
   - Role management

2. **System Configuration**
   - Authentication setup
   - Provider integration
   - System settings

## Navigation Design

### 1. Primary Navigation
```
┌─────────────────────────────────────┐
│ Logo + App Menu    User    Settings │
├─────────────────────────────────────┤
│                                     │
│           Content Area              │
│                                     │
└─────────────────────────────────────┘
```

Implementation:
```tsx
// Using shadcn NavigationMenu
<div className="min-h-screen">
  <header className="sticky top-0 z-50 w-full border-b">
    <div className="container flex h-14 items-center">
      <NavigationMenu>
        <NavigationMenuList>
          <NavigationMenuItem>
            <NavigationMenuTrigger>Menu</NavigationMenuTrigger>
            {/* Menu content */}
          </NavigationMenuItem>
        </NavigationMenuList>
      </NavigationMenu>
      <div className="ml-auto flex items-center space-x-4">
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" className="relative h-8 w-8">
              <Avatar />
            </Button>
          </DropdownMenuTrigger>
          {/* User menu content */}
        </DropdownMenu>
      </div>
    </div>
  </header>
  <main className="flex-1">{/* Content */}</main>
</div>
```

#### Desktop Header
```
┌───────────┬──────────────┬─────────┐
│ Logo Menu │   Breadcrumb │ Profile │
└───────────┴──────────────┴─────────┘
```

#### Mobile Header
```
┌─────┬──────────────────────┬───────┐
│ Menu│       Title          │Profile│
└─────┴──────────────────────┴───────┘
```

### 2. Navigation Components

#### App Menu (Desktop)
```
┌─────────────────┐
│ • Home          │
│ • Chat          │
│ • Models        │
│   • Aliases     │
│   • Files       │
│   • Downloads   │
│ • System        │
│   • Users       │
│   • Settings    │
└─────────────────┘
```

Implementation:
```tsx
// Using shadcn components
<NavigationMenu>
  <NavigationMenuList>
    {menuItems.map((item) => (
      <NavigationMenuItem key={item.id}>
        {item.children ? (
          <>
            <NavigationMenuTrigger>{item.label}</NavigationMenuTrigger>
            <NavigationMenuContent>
              <ul className="grid w-[400px] gap-3 p-4 md:w-[500px] md:grid-cols-2">
                {item.children.map((child) => (
                  <ListItem key={child.id} {...child} />
                ))}
              </ul>
            </NavigationMenuContent>
          </>
        ) : (
          <NavigationMenuLink href={item.href}>
            {item.label}
          </NavigationMenuLink>
        )}
      </NavigationMenuItem>
    ))}
  </NavigationMenuList>
</NavigationMenu>
```

#### Mobile Menu (Slide-out)
```
┌─────────────────┐
│ User Profile    │
├─────────────────┤
│ Primary Nav     │
├─────────────────┤
│ Secondary Nav   │
└─────────────────┘
```

Mobile Implementation:
```tsx
<Sheet>
  <SheetTrigger asChild>
    <Button variant="ghost" size="icon" className="md:hidden">
      <Menu className="h-6 w-6" />
    </Button>
  </SheetTrigger>
  <SheetContent side="left" className="w-[300px] sm:w-[400px]">
    <Accordion type="single" collapsible>
      {menuItems.map((item) => (
        <AccordionItem key={item.id} value={item.id}>
          <AccordionTrigger>{item.label}</AccordionTrigger>
          <AccordionContent>
            {/* Menu items */}
          </AccordionContent>
        </AccordionItem>
      ))}
    </Accordion>
  </SheetContent>
</Sheet>
```

### 3. Page-Specific Layouts

#### Chat Interface
```
┌────────┬────────────┬────────┐
│History │            │Settings│
│List    │  Chat Area │Panel   │
│        │            │        │
└────────┴────────────┴────────┘

Mobile:
┌────────────────────┐
│Chat Area           │
└────────────────────┘
↓ Expandable panels
```

Implementation:
```tsx
<div className="flex h-screen">
  {/* Left Sidebar */}
  <div className="w-[300px] border-r hidden md:block">
    <div className="p-4 h-full flex flex-col">
      <ScrollArea className="flex-1">
        {/* Chat history */}
      </ScrollArea>
    </div>
  </div>

  {/* Main Chat */}
  <div className="flex-1 flex flex-col">
    <ScrollArea className="flex-1">
      {/* Messages */}
    </ScrollArea>
    <div className="border-t p-4">
      {/* Input area */}
    </div>
  </div>

  {/* Right Settings - Using Sheet on mobile */}
  <div className="w-[300px] border-l hidden lg:block">
    <ScrollArea className="h-full">
      {/* Settings content */}
    </ScrollArea>
  </div>
</div>
```

#### Model Management
```
┌────────────────────────┐
│ Model Actions          │
├────────────────────────┤
│                        │
│ Content Area           │
│ (List/Form)            │
│                        │
└────────────────────────┘
```

Implementation:
```tsx
<div className="container mx-auto py-6">
  <div className="flex items-center justify-between">
    <h1 className="text-3xl font-bold tracking-tight">Models</h1>
    <Button>New Model</Button>
  </div>
  
  <Tabs defaultValue="aliases" className="mt-6">
    <TabsList className="grid w-full grid-cols-3">
      <TabsTrigger value="aliases">Aliases</TabsTrigger>
      <TabsTrigger value="files">Files</TabsTrigger>
      <TabsTrigger value="downloads">Downloads</TabsTrigger>
    </TabsList>
    <TabsContent value="aliases">
      <DataTable columns={columns} data={data} />
    </TabsContent>
  </Tabs>
</div>
```

#### Home Dashboard
```
┌────────────────────────┐
│ Quick Actions          │
├──────────┬─────────────┤
│ News     │ Learning    │
│ Updates  │ Resources   │
└──────────┴─────────────┘
```

Implementation:
```tsx
<div className="container mx-auto py-6">
  <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
    <Card>
      <CardHeader>
        <CardTitle>Quick Actions</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="grid gap-2">
          {actions.map((action) => (
            <Button key={action.id} variant="outline" className="w-full">
              {action.label}
            </Button>
          ))}
        </div>
      </CardContent>
    </Card>
    
    <Card className="md:col-span-2">
      <CardHeader>
        <CardTitle>Updates & Resources</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="grid md:grid-cols-2 gap-4">
          {/* News and Learning sections */}
        </div>
      </CardContent>
    </Card>
  </div>
</div>
```

## Implementation Guidelines

### 1. Responsive Design
- Collapsible sidebars on mobile
- Touch-friendly interactions
- Adaptive layouts
- Priority content first

### 2. Navigation Patterns
- Consistent header across pages
- Clear visual hierarchy
- Contextual sub-navigation
- Breadcrumb trails

### 3. Layout Flexibility
```
Content Adaptability:
├── Full width (Home, Lists)
├── Split view (Chat)
└── Form layouts (Settings)
```

### 4. Interaction Design
- Smooth transitions
- Clear feedback
- Loading states
- Error handling

## Mobile Considerations

### 1. Touch Interactions
- Large touch targets
- Swipe gestures
- Pull-to-refresh
- Bottom navigation

### 2. Space Management
- Collapsible elements
- Progressive disclosure
- Priority content
- Floating action buttons

### 3. Performance
- Minimal animations
- Efficient loading
- Cached resources
- Offline support

## Navigation States

### 1. Authentication States
- Unauthenticated: Setup/Login
- Authenticated: Full navigation
- Admin: Extended options

### 2. Context Awareness
- Active section highlighting
- Breadcrumb trails
- Related actions
- Quick returns

### 3. User Feedback
- Loading indicators
- Success messages
- Error states
- Progress tracking

## Future Considerations

### 1. Scalability
- New feature integration
- Plugin architecture
- Custom layouts
- Extended navigation

### 2. Accessibility
- Keyboard navigation
- Screen readers
- High contrast
- Focus management

### 3. Customization
- User preferences
- Layout options
- Theme support
- Quick access customization

## Component Library Integration

### 1. shadcn Components
```tsx
// Common component patterns
const components = {
  navigation: {
    primary: NavigationMenu,
    mobile: Sheet,
    dropdown: DropdownMenu,
  },
  layout: {
    card: Card,
    tabs: Tabs,
    scroll: ScrollArea,
    dialog: Dialog,
  },
  feedback: {
    toast: Toast,
    alert: Alert,
    progress: Progress,
  }
}
```

### 2. Tailwind Utilities
```tsx
// Common utility patterns
const utilities = {
  layout: {
    container: "container mx-auto px-4 sm:px-6 lg:px-8",
    section: "py-6 space-y-6",
    grid: "grid gap-4 md:grid-cols-2 lg:grid-cols-3",
  },
  flex: {
    center: "flex items-center justify-center",
    between: "flex items-center justify-between",
    stack: "flex flex-col space-y-4",
  },
  responsive: {
    hidden: "hidden md:block",
    mobile: "md:hidden",
    sidebar: "w-[300px] sm:w-[400px]",
  }
}
```

### 3. Theme Configuration
```tsx
// tailwind.config.js
const config = {
  darkMode: ["class"],
  content: [
    './pages/**/*.{ts,tsx}',
    './components/**/*.{ts,tsx}',
    './app/**/*.{ts,tsx}',
  ],
  theme: {
    container: {
      center: true,
      padding: "2rem",
      screens: {
        "2xl": "1400px",
      },
    },
  }
}
