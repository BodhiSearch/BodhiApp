'use client'

import AppInitializer from '@/components/AppInitializer';

export default function HomePage() {
  return (
    <div>
      <AppInitializer allowedStatus="ready" />
      <h1>Home</h1>
      {/* Add your home page content here */}
    </div>
  )
}