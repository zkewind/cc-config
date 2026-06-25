import type { AppId } from "@/lib/api";
import SubscriptionQuotaFooter from "@/components/SubscriptionQuotaFooter";

export type QuotaFooterProps = {
  kind: "subscription";
  appId: AppId;
  inline?: boolean;
  isCurrent?: boolean;
};

export default function QuotaFooter(props: QuotaFooterProps) {
  return (
    <SubscriptionQuotaFooter
      appId={props.appId}
      inline={props.inline}
      isCurrent={props.isCurrent}
    />
  );
}
