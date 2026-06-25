import type { Provider } from "@/types";
import type { AppId } from "@/lib/api";
import QuotaFooter from "@/components/QuotaFooter";

interface ProviderStatusProps {
  provider: Provider;
  appId: AppId;
  isCurrent: boolean;
  isInConfig: boolean;
}

export interface ProviderStatusState {
  // balance display removed; subscription quota shown via QuotaFooter
}

export function useProviderStatus(
  _props: ProviderStatusProps,
): ProviderStatusState {
  return {};
}

// ─── Inline indicator (rendered inside the flex row) ───────────────────────

interface ProviderStatusIndicatorProps {
  provider: Provider;
  appId: AppId;
  isCurrent: boolean;
  isInConfig: boolean;
  isOfficial: boolean;
  status: ProviderStatusState;
}

export function ProviderStatusIndicator({
  appId,
  isCurrent,
  isOfficial,
}: ProviderStatusIndicatorProps) {
  if (!isOfficial) return null;

  return (
    <QuotaFooter
      kind="subscription"
      appId={appId}
      inline={true}
      isCurrent={isCurrent}
    />
  );
}

// ─── Expanded panel (rendered below the flex row, full-width) ───────────────

interface ProviderStatusExpandedPanelProps {
  provider: Provider;
  appId: AppId;
  isCurrent: boolean;
  isInConfig: boolean;
  status: ProviderStatusState;
}

export function ProviderStatusExpandedPanel(
  _props: ProviderStatusExpandedPanelProps,
) {
  return null;
}
