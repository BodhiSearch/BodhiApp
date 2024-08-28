'use client'

import { useState } from 'react'
import { useRouter, useSearchParams } from 'next/navigation'
import Image from 'next/image'
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Loader2 } from 'lucide-react'
import AppInitializer from '@/components/AppInitializer'
import { BodhiBackend } from '@/services/BodhiBackend'

export default function Setup() {
  const router = useRouter()
  const searchParams = useSearchParams()
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(searchParams.get('error'))
  const bodhi_url = process.env.NEXT_PUBLIC_BODHI_URL || '';
  const bodhiBackend = new BodhiBackend(bodhi_url);

  const handleSetup = async (authz: boolean) => {
    setIsLoading(true)
    setError(null)

    try {
      const data = await bodhiBackend.setupApp(authz)
      if (data.status === 'ready') {
        router.push('/ui/home')
      } else if (data.status === 'resource-admin') {
        router.push('/ui/setup/resource-admin')
      }
    } catch (err) {
      setError('An unexpected error occurred')
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <AppInitializer allowedStatus="setup">
      <div className="flex flex-col items-center justify-center min-h-screen p-4 bg-gray-100">
        <Image src="/bodhi-logo.png" alt="Bodhi App Logo" width={150} height={150} className="mb-8" />
        <Card className="w-full max-w-md">
          <CardHeader>
            <CardTitle>Bodhi App Setup</CardTitle>
            <CardDescription>Choose your setup mode</CardDescription>
          </CardHeader>
          <CardContent>
            {error && (
              <Alert variant="destructive" className="mb-4">
                <AlertTitle>Error</AlertTitle>
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            )}
            <Alert className="mb-4">
              <AlertTitle>Warning</AlertTitle>
              <AlertDescription>
                Setting up in non-authenticated mode may compromise system resources.
                We recommend choosing the authenticated mode for better security.
              </AlertDescription>
            </Alert>
            <div className="space-y-4">
              <Button
                className="w-full"
                onClick={() => handleSetup(true)}
                disabled={isLoading}
              >
                {isLoading ? (
                  <>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    Setting up...
                  </>
                ) : (
                  'Setup Authenticated Instance →'
                )}
              </Button>
              <Button
                variant="outline"
                className="w-full"
                onClick={() => handleSetup(false)}
                disabled={isLoading}
              >
                {isLoading ? (
                  <>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    Setting up...
                  </>
                ) : (
                  'Setup Unauthenticated Instance →'
                )}
              </Button>
            </div>
          </CardContent>
          <CardFooter className="justify-center">
            <p className="text-sm text-gray-500">For more information, visit our documentation.</p>
          </CardFooter>
        </Card>
      </div>
    </AppInitializer>
  )
}