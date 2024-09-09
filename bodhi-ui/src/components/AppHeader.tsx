import Image from 'next/image';
import PageNavigation from './PageNavigation';
import UserMenu from './UserMenu';

export default function AppHeader() {
  // This is a placeholder. In a real app, you'd get this from your auth state.
  const userEmail = 'user@example.com';

  // This is a placeholder function. In a real app, you'd implement actual logout logic.
  const handleLogout = () => {
    console.log('Logout clicked');
    // Implement your logout logic here
  };

  return (
    <div className="flex flex-col items-center sm:flex-row sm:justify-between my-6 space-y-6 sm:space-y-0">
      <div className="order-2 sm:order-1 w-full sm:w-auto">
        <PageNavigation />
      </div>
      <div className="flex items-center space-x-4 order-1 sm:order-2">
        <Image
          src="/bodhi-logo.png"
          alt="Bodhi Logo"
          width={50}
          height={50}
          priority
        />
        <h1 className="text-3xl sm:text-4xl font-bold text-primary">Bodhi</h1>
      </div>
      <div className="order-3 w-full sm:w-auto">
        <UserMenu email={userEmail} onLogout={handleLogout} />
      </div>
    </div>
  );
}
