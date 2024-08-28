'use client'

import AppInitializer from '@/components/AppInitializer'
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"

export default function ResourceAdminPage() {
  const handleLogin = () => {
    // Redirect to the external Keycloak login URL
    window.location.href = process.env.NEXT_PUBLIC_KEYCLOAK_LOGIN_URL || ''
  }

  return (
    <AppInitializer allowedStatus="resource-admin">
      <Card className="w-full max-w-md mx-auto mt-10">
        <CardHeader>
          <CardTitle>Resource Admin Setup</CardTitle>
          <CardDescription>You will be made the app admin using the account you log in with.</CardDescription>
        </CardHeader>
        <CardContent>
          <Button onClick={handleLogin} className="w-full">Log In</Button>
        </CardContent>
      </Card>
    </AppInitializer>
  )
}
