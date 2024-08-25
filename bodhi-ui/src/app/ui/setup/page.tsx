'use client'

import { useEffect, useState } from 'react'
import { useRouter } from 'next/navigation'
import Image from 'next/image'
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"

export default function Setup() {
  const [isReady, setIsReady] = useState(false)
  const router = useRouter()
  const bodhi_url = process.env.NEXT_PUBLIC_BODHI_URL;
  useEffect(() => {
    const checkSetupStatus = async () => {
      const response = await fetch(`${bodhi_url}/app/info`)
      const data = await response.json()
      if (data.status === 'ready') {
        router.push('/ui/home')
      } else {
        setIsReady(true)
      }
    }
    checkSetupStatus()
  }, [router])

  if (!isReady) {
    return <div className="flex items-center justify-center h-screen">Loading...</div>
  }

  return (
    <div className="flex flex-col items-center justify-center min-h-screen p-4 bg-gray-100">
      <Image src="/bodhi-logo.png" alt="Bodhi App Logo" width={150} height={150} className="mb-8" />
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle>Bodhi App Setup</CardTitle>
          <CardDescription>Choose your setup mode</CardDescription>
        </CardHeader>
        <CardContent>
          <Alert className="mb-4">
            <AlertTitle>Warning</AlertTitle>
            <AlertDescription>
              Setting up in non-authenticated mode may compromise system resources.
              We recommend choosing the authenticated mode for better security.
            </AlertDescription>
          </Alert>
          <div className="space-y-4">
            <Button className="w-full" onClick={() => router.push('/ui/setup/authenticated')}>
              Setup Authenticated Instance →
            </Button>
            <Button variant="outline" className="w-full" onClick={() => router.push('/ui/setup/unauthenticated')}>
              Setup Unauthenticated Instance →
            </Button>
          </div>
        </CardContent>
        <CardFooter className="justify-center">
          <p className="text-sm text-gray-500">For more information, visit our documentation.</p>
        </CardFooter>
      </Card>
    </div>
  )
}