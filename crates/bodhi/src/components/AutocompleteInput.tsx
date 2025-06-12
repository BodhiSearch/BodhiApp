import React, { useState, useEffect, useRef, useCallback } from 'react';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';

interface AutocompleteInputProps {
  value: string;
  onChange: (value: string) => void;
  suggestions: string[];
  loading: boolean;
  inputRef: React.RefObject<HTMLInputElement>;
}

export const AutocompleteInput: React.FC<AutocompleteInputProps> = ({
  value,
  onChange,
  suggestions,
  loading,
  inputRef,
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const [filteredSuggestions, setFilteredSuggestions] = useState<string[]>([]);
  const [activeSuggestionIndex, setActiveSuggestionIndex] = useState(-1);
  const wrapperRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (suggestions && value) {
      const filtered = suggestions.filter((suggestion) =>
        suggestion.toLowerCase().includes(value.toLowerCase())
      );
      setFilteredSuggestions(filtered);
    } else {
      setFilteredSuggestions(suggestions || []);
    }
    setActiveSuggestionIndex(-1);
  }, [value, suggestions]);

  const handleClickOutside = useCallback(
    (event: MouseEvent) => {
      if (
        wrapperRef.current &&
        !wrapperRef.current.contains(event.target as Node) &&
        inputRef.current &&
        !inputRef.current.contains(event.target as Node)
      ) {
        setIsOpen(false);
      }
    },
    [inputRef]
  );

  const handleFocus = useCallback(() => {
    setIsOpen(true);
  }, []);

  const handleBlur = useCallback(() => {
    setTimeout(() => setIsOpen(false), 200);
  }, []);

  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      if (!isOpen) return;

      switch (event.key) {
        case 'ArrowDown':
          event.preventDefault();
          setActiveSuggestionIndex((prevIndex) =>
            prevIndex < filteredSuggestions.length - 1
              ? prevIndex + 1
              : prevIndex
          );
          break;
        case 'ArrowUp':
          event.preventDefault();
          setActiveSuggestionIndex((prevIndex) =>
            prevIndex > 0 ? prevIndex - 1 : -1
          );
          break;
        case 'Enter':
          event.preventDefault();
          if (activeSuggestionIndex >= 0) {
            onChange(filteredSuggestions[activeSuggestionIndex]);
            setIsOpen(false);
          }
          break;
        case 'Escape':
          setIsOpen(false);
          break;
      }
    },
    [isOpen, filteredSuggestions, activeSuggestionIndex, onChange]
  );

  useEffect(() => {
    const currentInputRef = inputRef.current;

    document.addEventListener('mousedown', handleClickOutside);
    currentInputRef?.addEventListener('focus', handleFocus);
    currentInputRef?.addEventListener('blur', handleBlur);
    currentInputRef?.addEventListener('keydown', handleKeyDown);

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      currentInputRef?.removeEventListener('focus', handleFocus);
      currentInputRef?.removeEventListener('blur', handleBlur);
      currentInputRef?.removeEventListener('keydown', handleKeyDown);
    };
  }, [inputRef, handleClickOutside, handleFocus, handleBlur, handleKeyDown]);

  const handleSuggestionClick = (e: React.MouseEvent, suggestion: string) => {
    e.preventDefault();
    onChange(suggestion);
    setIsOpen(false);
    inputRef.current?.focus();
  };

  const suggestionBoxHeight = Math.min(filteredSuggestions.length * 40, 200); // 40px per item, max 200px

  return (
    <div ref={wrapperRef} className="relative w-full">
      {isOpen && !loading && filteredSuggestions.length > 0 && (
        <div
          className="absolute z-10 w-full mt-1 overflow-y-auto rounded-md border bg-background shadow-md"
          style={{ maxHeight: `${suggestionBoxHeight}px` }}
        >
          {filteredSuggestions.map((suggestion, index) => (
            <Button
              key={index}
              variant="ghost"
              className={cn(
                'flex w-full items-start justify-start px-2 py-1.5 text-sm',
                index === activeSuggestionIndex && 'bg-accent'
              )}
              onClick={(e) => handleSuggestionClick(e, suggestion)}
              type="button"
            >
              {suggestion}
            </Button>
          ))}
        </div>
      )}
    </div>
  );
};
