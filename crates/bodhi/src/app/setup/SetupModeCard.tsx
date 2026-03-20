import { motion } from 'framer-motion';
import { Loader2 } from 'lucide-react';

import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';

import { itemVariants } from './types';

type SetupModeCardProps = {
  onSetup: () => void;
  isLoading: boolean;
};

export const SetupModeCard = ({ onSetup, isLoading }: SetupModeCardProps) => {
  return (
    <motion.div variants={itemVariants}>
      <Card>
        <CardHeader>
          <CardTitle className="text-center">Setup Bodhi App</CardTitle>
        </CardHeader>
        <CardContent className="space-y-6">
          {/* Setup mode description */}
          <div className="space-y-4 text-center">
            <div className="flex items-center justify-center gap-2">
              <span className="text-4xl">🔐</span>
              <div>
                <h3 className="text-xl font-semibold">Authenticated Mode</h3>
                <p className="text-sm text-muted-foreground">Secure setup with user authentication</p>
              </div>
            </div>
            <ul className="space-y-2 text-sm text-left max-w-md mx-auto">
              <li className="flex items-start gap-2">
                <span className="text-primary">•</span>
                <span>User authentication and secure access control</span>
              </li>
              <li className="flex items-start gap-2">
                <span className="text-primary">•</span>
                <span>Multi-user support with RBAC</span>
              </li>
              <li className="flex items-start gap-2">
                <span className="text-primary">•</span>
                <span>API tokens for secure API access</span>
              </li>
              <li className="flex items-start gap-2">
                <span className="text-primary">•</span>
                <span>Resource usage tracking and quotas</span>
              </li>
            </ul>
          </div>

          <div className="pt-6">
            <Button className="w-full relative" size="lg" onClick={onSetup} disabled={isLoading}>
              {isLoading ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Setting up Bodhi App...
                </>
              ) : (
                'Setup Bodhi App →'
              )}
            </Button>
          </div>
        </CardContent>
      </Card>
    </motion.div>
  );
};
