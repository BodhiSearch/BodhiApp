import { useState, useEffect } from 'react';
import { useRouter, usePathname } from 'next/navigation';
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";
import { Button } from "@/components/ui/button";
import { ChevronDown } from "lucide-react";

const pages = [
  { name: 'Home', path: '/ui/home' },
  { name: 'Models', path: '/ui/models' },
  { name: 'Model Files', path: '/ui/modelfiles' }, // Add this line
];

export default function PageNavigation() {
  const router = useRouter();
  const pathname = usePathname();
  const [currentPage, setCurrentPage] = useState('Home');

  useEffect(() => {
    const page = pages.find(p => pathname.startsWith(p.path));
    if (page) {
      setCurrentPage(page.name);
    }
  }, [pathname]);

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="outline" className="w-full justify-between">
          {currentPage} <ChevronDown className="ml-2 h-4 w-4" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align='start' className="w-[calc(100vw-2rem)] sm:w-[200px]">
        {pages.map((page) => (
          <DropdownMenuItem
            key={page.path}
            onClick={() => router.push(page.path)}
            className="justify-center sm:justify-start"
          >
            {page.name}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
