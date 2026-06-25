import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import QuotaFooter from "@/components/QuotaFooter";

const { subscriptionFooterSpy } = vi.hoisted(() => ({
  subscriptionFooterSpy: vi.fn(),
}));

vi.mock("@/components/SubscriptionQuotaFooter", () => ({
  default: (props: unknown) => {
    subscriptionFooterSpy(props);
    return <div data-testid="subscription-quota-footer" />;
  },
}));

describe("QuotaFooter", () => {
  beforeEach(() => {
    subscriptionFooterSpy.mockClear();
  });

  it("routes official subscription quota through the unified footer entry", () => {
    render(<QuotaFooter kind="subscription" appId="claude" inline isCurrent />);

    expect(screen.getByTestId("subscription-quota-footer")).toBeInTheDocument();
    expect(subscriptionFooterSpy).toHaveBeenCalledWith({
      appId: "claude",
      inline: true,
      isCurrent: true,
    });
  });
});
