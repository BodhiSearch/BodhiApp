import { motion } from 'framer-motion';
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Loader2 } from 'lucide-react';
import { itemVariants, SetupMode } from './types';

type SetupModeCardProps = {
  setupModes: SetupMode[];
  onSetup: () => void;
  isLoading: boolean;
};

export const SetupModeCard = ({
  setupModes,
  onSetup,
  isLoading,
}: SetupModeCardProps) => {
  return (
    <motion.div variants={itemVariants}>
      <Card>
        <CardHeader>
          <CardTitle className="text-center">Setup Your Bodhi App</CardTitle>
        </CardHeader>
        <CardContent className="space-y-6">
          {/* Setup mode display */}
          <div className="space-y-4">
            {setupModes.map((mode) => (
              <div key={mode.title} className="space-y-4">
                <div className="flex items-center gap-2">
                  <span className="text-2xl">{mode.icon}</span>
                  <div>
                    <h3 className="font-semibold">{mode.title}</h3>
                    <p className="text-sm text-muted-foreground">
                      {mode.description}
                    </p>
                  </div>
                </div>
                <ul className="space-y-2 text-sm">
                  {mode.benefits.map((benefit, index) => (
                    <li key={index} className="flex items-start gap-2">
                      <span className="text-primary">•</span>
                      <span>{benefit}</span>
                    </li>
                  ))}
                </ul>
              </div>
            ))}
          </div>

          <div className="pt-6">
            <Button
              className="w-full relative"
              size="lg"
              onClick={() => onSetup()}
              disabled={isLoading}
            >
              {isLoading ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Setting up your secure instance...
                </>
              ) : (
                'Setup Secure Instance →'
              )}
            </Button>
          </div>
        </CardContent>
        <CardFooter className="flex flex-col gap-4">
          <div className="flex items-center gap-2 p-4 border rounded-lg bg-yellow-500/10 text-yellow-600 dark:text-yellow-400">
            <span className="text-2xl">⚠️</span>
            <div className="text-sm">
              <p>
                You cannot switch your choice later. You will need to reinstall
                the app, losing your data, in the process.
              </p>
            </div>
          </div>
          <p className="text-sm text-muted-foreground text-center w-full">
            Please review the features carefully and make your choice based on
            your requirements.
          </p>
        </CardFooter>
      </Card>
    </motion.div>
  );
};
