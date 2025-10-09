"use client";

import { Mail } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Container } from "@/components/ui/container";
import Link from "next/link";
import { motion } from "framer-motion";

export function DownloadSection() {
  return (
    <section className="py-20 bg-gradient-to-b from-violet-50 to-white">
      <Container>
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="text-center"
        >
          <h2 className="text-3xl font-bold mb-6">Stay in the loop</h2>
          <p className="text-muted-foreground mb-8">
            We are working on a new version of Bodhi App that will be available for download soon. Sign up to get notified.
          </p>
          <Button size="lg" className="gap-2" asChild>
            <Link href="https://tally.so/r/mVyxQa" target="_blank" rel="noopener noreferrer">
              <Mail className="h-5 w-5" />
              Get notified
            </Link>
          </Button>
        </motion.div>
      </Container>
    </section>
  );
} 