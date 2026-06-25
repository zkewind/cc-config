import React from "react";
import { QueryClientProvider } from "@tanstack/react-query";
import { ThemeProvider } from "@/components/theme-provider";
import { UpdateProvider } from "@/contexts/UpdateContext";
import { Toaster } from "@/components/ui/sonner";
import { queryClient } from "@/lib/query";

interface AppProvidersProps {
  children: React.ReactNode;
}

export function AppProviders({ children }: AppProvidersProps) {
  return (
    <QueryClientProvider client={queryClient}>
      <ThemeProvider defaultTheme="system" storageKey="cc-config-theme">
        <UpdateProvider>
          {children}
          <Toaster />
        </UpdateProvider>
      </ThemeProvider>
    </QueryClientProvider>
  );
}
