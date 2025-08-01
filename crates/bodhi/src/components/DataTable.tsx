import { Button } from '@/components/ui/button';
import { Skeleton } from '@/components/ui/skeleton';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { ArrowUpDown, ChevronDown, ChevronUp } from 'lucide-react';
import React, { useState } from 'react';

export interface SortState {
  column: string;
  direction: 'asc' | 'desc';
}

interface Column {
  id: string;
  name: string;
  sorted: boolean;
  className?: string;
}

interface DataTableProps<T> {
  data: T[];
  columns: Column[];
  loading: boolean;
  sort: SortState;
  onSortChange: (column: string) => void;
  renderRow: (item: T) => React.ReactNode;
  renderExpandedRow?: (item: T) => React.ReactNode;
  getItemId: (item: T) => string; // New prop for getting unique ID
}

export function DataTable<T>({
  data,
  columns,
  loading,
  sort,
  onSortChange,
  renderRow,
  renderExpandedRow,
  getItemId, // New prop
}: DataTableProps<T>) {
  const [expandedRow, setExpandedRow] = useState<string | null>(null);

  const renderSortIcon = (columnId: string) => {
    if (sort.column !== columnId) {
      return <ArrowUpDown className="ml-2 h-4 w-4" />;
    }
    return sort.direction === 'asc' ? <ChevronUp className="ml-2 h-4 w-4" /> : <ChevronDown className="ml-2 h-4 w-4" />;
  };

  const toggleRowExpansion = (name: string) => {
    setExpandedRow(expandedRow === name ? null : name);
  };

  if (loading) {
    return (
      <Table>
        <TableBody>
          {[...Array(5)].map((_, i) => (
            <TableRow key={i}>
              <TableCell colSpan={columns.length + 1}>
                <Skeleton className="h-12 w-full" />
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    );
  }

  const renderTableRows = (item: T, showExpand: boolean) => (
    <TableRow key={getItemId(item)}>
      {renderRow(item)}
      {showExpand && (
        <TableCell>
          <Button variant="ghost" size="sm" onClick={() => toggleRowExpansion(getItemId(item))}>
            {expandedRow === getItemId(item) ? <ChevronUp /> : <ChevronDown />}
          </Button>
        </TableCell>
      )}
    </TableRow>
  );

  return (
    <Table>
      <TableHeader>
        <TableRow>
          {columns.map((column) => (
            <TableHead key={column.id} className={`min-w-8 ${column.className || ''}`}>
              {column.sorted ? (
                <Button variant="ghost" onClick={() => onSortChange(column.id)} className="h-8 px-2 font-medium">
                  {column.name}
                  {renderSortIcon(column.id)}
                </Button>
              ) : column.name === '' ? (
                <div className="min-w-10">
                  <span className="sr-only">Actions</span>
                </div>
              ) : (
                column.name
              )}
            </TableHead>
          ))}
          {renderExpandedRow && <TableHead className="w-10" data-testid="expanded-row-head" />}
        </TableRow>
      </TableHeader>
      <TableBody>
        {data.map((item) => {
          const expandedContent = renderExpandedRow?.(item);

          return expandedContent ? (
            <React.Fragment key={getItemId(item)}>
              {renderTableRows(item, true)}
              {expandedRow === getItemId(item) && (
                <TableRow>
                  <TableCell colSpan={columns.length + 1}>{expandedContent}</TableCell>
                </TableRow>
              )}
            </React.Fragment>
          ) : (
            renderTableRows(item, false)
          );
        })}
      </TableBody>
    </Table>
  );
}

export function Pagination({
  page,
  totalPages,
  onPageChange,
}: {
  page: number;
  totalPages: number;
  onPageChange: (newPage: number) => void;
}) {
  return (
    <div className="flex justify-center gap-4">
      <Button size="sm" onClick={() => onPageChange(Math.max(1, page - 1))} disabled={page === 1} className="px-6">
        Previous
      </Button>
      <span className="flex items-center">
        {page}/{totalPages}
      </span>
      <Button size="sm" onClick={() => onPageChange(page + 1)} disabled={page === totalPages} className="px-6">
        Next
      </Button>
    </div>
  );
}
