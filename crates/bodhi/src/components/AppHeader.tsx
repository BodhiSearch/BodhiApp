import PageNavigation from './PageNavigation';
import UserMenu from '@/components/UserMenu';
import Logo from '@/components/Logo';

export default function AppHeader() {
  return (
    <div className="flex flex-col items-center sm:flex-row sm:justify-between my-6 space-y-6 sm:space-y-0">
      <div className="order-2 sm:order-1 w-full sm:w-auto">
        <PageNavigation />
      </div>
      <div className="order-1 sm:order-2">
        <Logo />
      </div>
      <div className="order-3 w-full sm:w-auto">
        <UserMenu />
      </div>
    </div>
  );
}
