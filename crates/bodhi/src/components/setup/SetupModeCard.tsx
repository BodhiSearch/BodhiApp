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
  onSetup: (authz: boolean) => void;
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
          <CardTitle className="text-center">Choose Your Setup Mode</CardTitle>
        </CardHeader>
        <CardContent className="space-y-6">
          {/* Setup modes grid */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
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
                  {mode.recommended && (
                    <span className="ml-auto inline-flex items-center rounded-full bg-primary/10 px-2.5 py-0.5 text-xs font-medium text-primary">
                      Recommended
                    </span>
                  )}
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

          <div className="pt-6 space-y-4">
            <Button
              className="w-full relative"
              size="lg"
              onClick={() => onSetup(true)}
              disabled={isLoading}
            >
              {isLoading ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Setting up authenticated instance...
                </>
              ) : (
                'Setup Authenticated Instance →'
              )}
            </Button>
            <div className="relative">
              <div className="absolute inset-0 flex items-center">
                <span className="w-full border-t" />
              </div>
              <div className="relative flex justify-center text-xs uppercase">
                <span className="bg-background px-2 text-muted-foreground">
                  Or
                </span>
              </div>
            </div>
            <Button
              variant="outline"
              className="w-full relative"
              size="lg"
              onClick={() => onSetup(false)}
              disabled={isLoading}
            >
              {isLoading ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Setting up unauthenticated instance...
                </>
              ) : (
                'Setup Unauthenticated Instance →'
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
