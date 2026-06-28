import { useEffect, useRef, useState } from 'react';

import { History, Plus, Search, SquarePen } from 'lucide-react';

import { AnchoredPopover } from '@/components/shell';
import { useShell } from '@/components/shell/ShellContext';
import { useChatStore } from '@/stores/chatStore';

import { ChatHistory } from './ChatHistory';

const HIST_POP = 'chat-history';

/**
 * The chat-history left-sidebar slot. Expanded: a "New chat" button, a History header with a
 * collapsible search, then the grouped list. Collapsed (icon-rail): two icon buttons whose
 * history button opens an AnchoredPopover list — driven by the shell's `collapsed`/`openPop` seam,
 * mirroring ShellNav.
 */
export function ChatHistorySidebar({ listOpen = true }: { listOpen?: boolean }) {
  const { collapsed, openPop, setOpenPop } = useShell();
  const createNewChat = useChatStore((s) => s.createNewChat);
  const [searchOpen, setSearchOpen] = useState(false);
  const [search, setSearch] = useState('');
  const histBtnRef = useRef<HTMLButtonElement>(null);
  const searchRef = useRef<HTMLInputElement>(null);
  const histOpen = openPop === HIST_POP;

  // Focus the input when the search field opens.
  useEffect(() => {
    if (searchOpen) searchRef.current?.focus();
  }, [searchOpen]);

  // Close the popover whenever we leave the collapsed icon-rail.
  useEffect(() => {
    if (!collapsed && histOpen) setOpenPop(null);
  }, [collapsed, histOpen, setOpenPop]);

  if (collapsed) {
    return (
      <>
        <button
          className="shell-railbtn shell-tip"
          data-tip="New chat"
          data-testid="new-chat-button"
          onClick={() => createNewChat()}
        >
          <SquarePen size={18} />
        </button>
        <button
          ref={histBtnRef}
          className={'shell-railbtn shell-tip' + (histOpen ? ' on' : '')}
          data-tip="Chat history"
          data-testid="chat-history-rail-button"
          onClick={(e) => {
            e.stopPropagation();
            setOpenPop(histOpen ? null : HIST_POP);
          }}
        >
          <History size={18} />
        </button>
        <AnchoredPopover open={histOpen} anchorRef={histBtnRef} onClose={() => setOpenPop(null)}>
          <div className="shell-pop-title">Chat history</div>
          <ChatHistory compact onSelect={() => setOpenPop(null)} />
        </AnchoredPopover>
      </>
    );
  }

  return (
    <div className="chat-hist">
      <button className="chat-new" data-testid="new-chat-button" onClick={() => createNewChat()}>
        <Plus size={14} />
        New chat
      </button>

      <div className="chat-hist-head">
        <span className="t">History</span>
        <button
          type="button"
          title="Search chats"
          aria-label="Search chats"
          data-testid="chat-history-search-toggle"
          onClick={() => setSearchOpen((o) => !o)}
        >
          <Search size={13} />
        </button>
      </div>

      <div className={'chat-hist-search' + (searchOpen ? ' open' : '')}>
        <input
          ref={searchRef}
          type="text"
          placeholder="Search conversations…"
          data-testid="chat-history-search-input"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          onBlur={() => {
            // Auto-hide the (empty) search on blur so it doesn't linger open.
            if (search.trim() === '') setSearchOpen(false);
          }}
        />
      </div>

      {listOpen && <ChatHistory search={search} />}
    </div>
  );
}
