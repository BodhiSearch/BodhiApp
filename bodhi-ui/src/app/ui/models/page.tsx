'use client'

import { useState, useEffect } from 'react';
import AppHeader from '@/components/AppHeader';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Button } from "@/components/ui/button";
import { ChevronDown, ChevronUp } from "lucide-react";
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

export default function ModelsPage() {
  const [models, setModels] = useState<Model[]>([]);
  const [loading, setLoading] = useState(true);
  const [page, setPage] = useState(1);
  const [expandedRow, setExpandedRow] = useState<string | null>(null);

  useEffect(() => {
    const fetchModels = async () => {
      setLoading(true);
      try {
        const response = await fetch(`/api/ui/models?page=${page}&page_size=30`);
        const data = await response.json();
        setModels(data.data);
      } catch (error) {
        console.error('Error fetching models:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchModels();
  }, [page]);

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
              <TableHead>Name</TableHead>
              <TableHead>Family</TableHead>
              <TableHead>Features</TableHead>
              <TableHead></TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {models.map((model) => (
              <>
                <TableRow key={model.alias}>
                  <TableCell>{model.alias}</TableCell>
                  <TableCell>{model.family || 'N/A'}</TableCell>
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
                        <p>Repo: {model.repo}</p>
                        <p>Filename: {model.filename}</p>
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
      <div className="mt-4 flex justify-between">
        <Button onClick={() => setPage(p => Math.max(1, p - 1))} disabled={page === 1}>
          Previous
        </Button>
        <Button onClick={() => setPage(p => p + 1)}>
          Next
        </Button>
      </div>
    </div>
  );
}
