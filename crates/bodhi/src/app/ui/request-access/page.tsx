'use client';

import AppInitializer from '@/components/AppInitializer';
import { AuthCard } from '@/components/AuthCard';
import { useRequestStatus, useSubmitAccessRequest } from '@/hooks/useAccessRequests';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { ROUTE_DEFAULT } from '@/lib/constants';
import { useRouter } from 'next/navigation';
import { useAuthenticatedUser } from '@/hooks/useUsers';

export function RequestAccessContent() {
  const router = useRouter();
  const { data: userInfo } = useAuthenticatedUser();
  const { data: requestStatus, isLoading: statusLoading, error: statusError } = useRequestStatus();
  const { showSuccess, showError } = useToastMessages();

  const { mutate: submitRequest, isLoading: isSubmitting } = useSubmitAccessRequest({
    onSuccess: () => {
      showSuccess('Request Submitted', 'Your access request has been submitted for review');
    },
    onError: (message) => {
      showError('Request Failed', message);
    },
  });

  // If user has a role, redirect to default page
  if (userInfo?.role) {
    router.push(ROUTE_DEFAULT);
    return null;
  }

  const handleRequestAccess = () => {
    submitRequest();
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString();
  };

  if (statusLoading) {
    return <AuthCard title="Loading..." isLoading={true} />;
  }

  // Show pending state if request exists and is pending
  if (requestStatus?.status === 'pending') {
    return (
      <AuthCard
        title="Access Request Pending"
        description={`Your access request submitted on ${formatDate(requestStatus.created_at)} is pending review`}
        actions={[]} // No actions when pending
      />
    );
  }

  // No request (404 error) or rejected - show request button
  return (
    <AuthCard
      title="Request Access"
      description="Request access to application"
      actions={[
        {
          label: isSubmitting ? 'Submitting...' : 'Request Access',
          onClick: handleRequestAccess,
          disabled: isSubmitting,
        },
      ]}
    />
  );
}

export default function RequestAccessPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <div className="pt-12 sm:pt-16" data-testid="request-access-page">
        <RequestAccessContent />
      </div>
    </AppInitializer>
  );
}
