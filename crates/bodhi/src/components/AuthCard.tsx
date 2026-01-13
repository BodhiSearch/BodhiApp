import Link from 'next/link';

import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';

interface AuthAction {
  label: string;
  href?: string;
  onClick?: () => void;
  variant?: 'default' | 'secondary' | 'destructive';
  disabled?: boolean;
  loading?: boolean;
}

interface AuthCardProps {
  title: string;
  description?: string | React.ReactNode;
  actions?: AuthAction[];
  disabled?: boolean;
  isLoading?: boolean;
}

function LoadingState() {
  return (
    <div className="animate-pulse space-y-4" data-testid="auth-card-loading">
      <div className="h-8 w-3/4 bg-muted rounded mx-auto" />
      <div className="h-24 bg-muted rounded" />
    </div>
  );
}

export function AuthCard({ title, description, actions = [], disabled = false, isLoading = false }: AuthCardProps) {
  return (
    <div className="w-full max-w-xl mx-auto px-4" data-testid="auth-card-container">
      <Card data-testid="auth-card" className="card-elevated bg-card">
        <CardHeader className="header-section" data-testid="auth-card-header">
          <CardTitle className="text-3xl text-center">{title}</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4 pt-6" data-testid="auth-card-content">
          {isLoading ? (
            <LoadingState />
          ) : (
            <>
              {description && (
                <div className="text-base text-muted-foreground text-center mb-6" data-testid="auth-card-description">
                  {description}
                </div>
              )}
              <div
                className={`space-y-4 ${disabled ? 'opacity-50 pointer-events-none' : 'opacity-100'}`}
                data-testid="auth-card-actions"
              >
                {actions.map((action, index) =>
                  action.href ? (
                    <Link key={index} href={action.href} passHref>
                      <Button
                        className="w-full text-base py-6"
                        variant={action.variant || 'default'}
                        disabled={action.disabled}
                        data-testid={`auth-card-action-${index}`}
                      >
                        {action.loading ? 'Loading...' : action.label}
                      </Button>
                    </Link>
                  ) : (
                    <Button
                      key={index}
                      className="w-full text-base py-6"
                      variant={action.variant || 'default'}
                      onClick={action.onClick}
                      disabled={action.disabled}
                      data-testid={`auth-card-action-${index}`}
                    >
                      {action.loading ? 'Loading...' : action.label}
                    </Button>
                  )
                )}
              </div>
            </>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
