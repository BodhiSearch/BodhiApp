import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { ArrowUpDown, ChevronDown, ChevronUp } from "lucide-react";
import React, { useState } from 'react';

export interface SortState {
  column: string;
  direction: 'asc' | 'desc';
}

interface Column {
  id: string;
  name: string;
  sorted: boolean;
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
  getItemId // New prop
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
      <div className="space-y-2">
        {[...Array(5)].map((_, i) => (
          <Skeleton key={i} className="h-12 w-full" />
        ))}
      </div>
    );
  }

  return (
    <Table>
      <TableHeader>
        <TableRow>
          {columns.map((column) => (
            <TableHead key={column.id}>
              {column.sorted ? (
                <Button
                  variant="ghost"
                  onClick={() => onSortChange(column.id)}
                  className="font-bold"
                >
                  {column.name}
                  {renderSortIcon(column.id)}
                </Button>
              ) : (
                <span className="font-bold">{column.name}</span>
              )}
            </TableHead>
          ))}
          {renderExpandedRow && <TableHead></TableHead>}
        </TableRow>
      </TableHeader>
      <TableBody>
        {data.map((item) => (
          <React.Fragment key={getItemId(item)}>
            <TableRow>
              {renderRow(item)}
              {renderExpandedRow && (
                <TableCell>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => toggleRowExpansion(getItemId(item))}
                  >
                    {expandedRow === getItemId(item) ? <ChevronUp /> : <ChevronDown />}
                  </Button>
                </TableCell>
              )}
            </TableRow>
            {renderExpandedRow && expandedRow === getItemId(item) && (
              <TableRow>
                <TableCell colSpan={columns.length + 1}>
                  {renderExpandedRow(item)}
                </TableCell>
              </TableRow>
            )}
          </React.Fragment>
        ))}
      </TableBody>
    </Table>
  );
}

export function Pagination({
  page,
  totalPages,
  onPageChange
}: {
  page: number;
  totalPages: number;
  onPageChange: (newPage: number) => void;
}) {
  return (
    <div className="flex items-center space-x-4">
      <Button
        onClick={() => onPageChange(Math.max(1, page - 1))}
        disabled={page === 1}
      >
        Previous
      </Button>
      <span>Page {page} of {totalPages}</span>
      <Button
        onClick={() => onPageChange(page + 1)}
        disabled={page === totalPages}
      >
        Next
      </Button>
    </div>
  );
}
