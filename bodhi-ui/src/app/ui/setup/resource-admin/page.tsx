'use client'

import AppInitializer from '@/components/AppInitializer'
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import Link from 'next/link'

export default function ResourceAdminPage() {
  const loginUrl = `${process.env.NEXT_PUBLIC_BODHI_URL}/app/login`

  return (
    <AppInitializer allowedStatus="resource-admin">
      <Card className="w-full max-w-md mx-auto mt-10">
        <CardHeader>
          <CardTitle>Resource Admin Setup</CardTitle>
          <CardDescription>You will be made the app admin using the account you log in with.</CardDescription>
        </CardHeader>
        <CardContent>
          <Link href={loginUrl} passHref>
            <Button className="w-full">Log In</Button>
          </Link>
        </CardContent>
      </Card>
    </AppInitializer>
  )
}
