import type { Provider } from "@/types";
import type { AppId } from "@/lib/api";
import { ProviderActions } from "@/components/providers/ProviderActions";

interface ProviderCardMenuProps {
  provider: Provider;
  appId: AppId;
  isCurrent: boolean;
  isInConfig: boolean;
  isTesting?: boolean;
  isOfficial: boolean;
  onEdit: (provider: Provider) => void;
  onDuplicate: (provider: Provider) => void;
  onTest?: (provider: Provider) => void;
  onConfigureUsage: (provider: Provider) => void;
  onDelete: (provider: Provider) => void;
  onRemoveFromConfig?: (provider: Provider) => void;
  onOpenTerminal?: (provider: Provider) => void;
}

export function ProviderCardMenu({
  provider,
  appId,
  isCurrent,
  isInConfig,
  isTesting,
  isOfficial,
  onEdit,
  onDuplicate,
  onTest,
  onConfigureUsage,
  onDelete,
  onRemoveFromConfig,
  onOpenTerminal,
}: ProviderCardMenuProps) {
  return (
    <div className="flex items-center gap-1.5 flex-shrink-0 opacity-0 pointer-events-none group-hover:opacity-100 group-focus-within:opacity-100 group-hover:pointer-events-auto group-focus-within:pointer-events-auto transition-opacity duration-200">
      <ProviderActions
        appId={appId}
        isCurrent={isCurrent}
        isInConfig={isInConfig}
        isTesting={isTesting}
        onEdit={() => onEdit(provider)}
        onDuplicate={() => onDuplicate(provider)}
        onTest={onTest && !isOfficial ? () => onTest(provider) : undefined}
        onConfigureUsage={
          isOfficial ? undefined : () => onConfigureUsage(provider)
        }
        onDelete={() => onDelete(provider)}
        onRemoveFromConfig={
          onRemoveFromConfig ? () => onRemoveFromConfig(provider) : undefined
        }
        onOpenTerminal={
          onOpenTerminal ? () => onOpenTerminal(provider) : undefined
        }
      />
    </div>
  );
}
