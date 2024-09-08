'use client'

import { useState, useEffect } from 'react';
import AppHeader from '@/components/AppHeader';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Button } from "@/components/ui/button";
import { ChevronDown, ChevronUp, ArrowUpDown } from "lucide-react";
import { Skeleton } from "@/components/ui/skeleton";

interface Model {
  alias: string;
  family?: string;
  repo: string;
  filename: string;
  snapshot: string;
  features: string[];
  chat_template: string;
  model_params: Record<string, any>;
  request_params: Record<string, any>;
  context_params: Record<string, any>;
}

interface ModelsResponse {
  data: Model[];
  total: number;
  page: number;
  page_size: number;
}

type SortDirection = 'asc' | 'desc';

interface SortState {
  column: string;
  direction: SortDirection;
}

export default function ModelsPage() {
  const [models, setModels] = useState<Model[]>([]);
  const [loading, setLoading] = useState(true);
  const [page, setPage] = useState(1);
  const [totalPages, setTotalPages] = useState(1);
  const [totalItems, setTotalItems] = useState(0);
  const [pageSize, setPageSize] = useState(30);
  const [expandedRow, setExpandedRow] = useState<string | null>(null);
  const [sort, setSort] = useState<SortState>({ column: 'alias', direction: 'asc' });

  useEffect(() => {
    const fetchModels = async () => {
      setLoading(true);
      try {
        const response = await fetch(`/api/ui/models?page=${page}&page_size=${pageSize}&sort=${sort.column}&sort_order=${sort.direction}`);
        const data: ModelsResponse = await response.json();
        setModels(data.data);
        setTotalPages(Math.ceil(data.total / data.page_size));
        setTotalItems(data.total);
        setPageSize(data.page_size);
      } catch (error) {
        console.error('Error fetching models:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchModels();
  }, [page, pageSize, sort]);

  const toggleSort = (column: string) => {
    setSort(prevSort => ({
      column,
      direction: prevSort.column === column && prevSort.direction === 'asc' ? 'desc' : 'asc'
    }));
    setPage(1); // Reset to first page when sorting
  };

  const renderSortIcon = (column: string) => {
    if (sort.column !== column) {
      return <ArrowUpDown className="ml-2 h-4 w-4" />;
    }
    return sort.direction === 'asc' ? <ChevronUp className="ml-2 h-4 w-4" /> : <ChevronDown className="ml-2 h-4 w-4" />;
  };

  const toggleRowExpansion = (name: string) => {
    setExpandedRow(expandedRow === name ? null : name);
  };

  return (
    <div className="container mx-auto px-4 sm:px-6 lg:px-8">
      <AppHeader />
      {loading ? (
        <div className="space-y-2">
          {[...Array(5)].map((_, i) => (
            <Skeleton key={i} className="h-12 w-full" />
          ))}
        </div>
      ) : (
        <Table>
          <TableHeader>
            <TableRow>
              {['Name', 'Family', 'Repo', 'Filename'].map((header) => (
                <TableHead key={header}>
                  <Button
                    variant="ghost"
                    onClick={() => toggleSort(header.toLowerCase())}
                    className="font-bold"
                  >
                    {header}
                    {renderSortIcon(header.toLowerCase())}
                  </Button>
                </TableHead>
              ))}
              <TableHead>Features</TableHead>
              <TableHead></TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {models.map((model) => (
              <>
                <TableRow key={model.alias}>
                  <TableCell>{model.alias}</TableCell>
                  <TableCell>{model.family || ''}</TableCell>
                  <TableCell>{model.repo}</TableCell>
                  <TableCell>{model.filename}</TableCell>
                  <TableCell>{model.features.join(', ')}</TableCell>
                  <TableCell>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => toggleRowExpansion(model.alias)}
                    >
                      {expandedRow === model.alias ? <ChevronUp /> : <ChevronDown />}
                    </Button>
                  </TableCell>
                </TableRow>
                {expandedRow === model.alias && (
                  <TableRow>
                    <TableCell colSpan={5}>
                      <div className="p-4 bg-gray-50">
                        <h4 className="font-semibold">Additional Details:</h4>
                        <p>SHA: {model.snapshot}</p>
                        <p>Template: {model.chat_template}</p>
                        <h5 className="font-semibold mt-2">Parameters:</h5>
                        <p>Model: {JSON.stringify(model.model_params)}</p>
                        <p>Request: {JSON.stringify(model.request_params)}</p>
                        <p>Context: {JSON.stringify(model.context_params)}</p>
                      </div>
                    </TableCell>
                  </TableRow>
                )}
              </>
            ))}
          </TableBody>
        </Table>
      )}
      <div className="mt-4 flex flex-col sm:flex-row justify-between items-center">
        <div className="mb-2 sm:mb-0">
          Displaying {models.length} items of {totalItems}
        </div>
        <div className="flex items-center space-x-4">
          <Button
            onClick={() => setPage(p => Math.max(1, p - 1))}
            disabled={page === 1}
          >
            Previous
          </Button>
          <span>Page {page} of {totalPages}</span>
          <Button
            onClick={() => setPage(p => p + 1)}
            disabled={page === totalPages}
          >
            Next
          </Button>
        </div>
      </div>
    </div>
  );
}
