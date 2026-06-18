import { createContext, useContext } from 'react';

export interface ShellContextValue {
  collapsed: boolean;
  isMobile: boolean;
  openPop: string | null;
  setOpenPop: (id: string | null) => void;
  openRail: () => void;
  closeRail: () => void;
  collapseRail: () => void;
}

export const ShellContext = createContext<ShellContextValue>({
  collapsed: false,
  isMobile: false,
  openPop: null,
  setOpenPop: () => {},
  openRail: () => {},
  closeRail: () => {},
  collapseRail: () => {},
});

export const useShell = (): ShellContextValue => useContext(ShellContext);
