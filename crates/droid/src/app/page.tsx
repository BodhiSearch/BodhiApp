'use client';

import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { invoke } from '@tauri-apps/api/core';

export default function Home() {
  const [downloadState, setDownloadState] = useState<string>('Download Llama 3.2');

  const handleDownload = async () => {
    try {
      const result = await invoke('download');
      setDownloadState(`downloading ${result}`);
      // Here you can add logic to track the download progress using the downloadId
    } catch (error) {
      console.error('Download failed:', error);
    }
  };

  return (
    <div className="flex h-screen items-center justify-center">
      <Button onClick={handleDownload}>
        {downloadState}
      </Button>
    </div>
  );
}