'use client';

import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { invoke } from '@tauri-apps/api/core';

export default function Home() {
  const [downloadState, setDownloadState] = useState<'idle' | 'downloading'>('idle');

  const handleDownload = async () => {
    try {
      const downloadId = await invoke('download');
      setDownloadState('downloading');
      // Here you can add logic to track the download progress using the downloadId
    } catch (error) {
      console.error('Download failed:', error);
    }
  };

  return (
    <div className="flex h-screen items-center justify-center">
      {downloadState === 'idle' ? (
        <Button onClick={handleDownload}>
          Download Llama 3.2
        </Button>
      ) : (
        <div>Downloading...</div>
      )}
    </div>
  );
}