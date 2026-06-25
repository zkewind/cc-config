import { render, screen, fireEvent } from "@testing-library/react";
import type { ReactNode } from "react";
import { useForm } from "react-hook-form";
import { describe, expect, it, vi } from "vitest";
import { Form } from "@/components/ui/form";
import { ClaudeConnectionFields } from "@/components/providers/forms/shared/ClaudeConnectionFields";
import type { ProviderFormData } from "@/lib/schemas/provider";

function FormWrapper({ children }: { children: ReactNode }) {
  const form = useForm<ProviderFormData>({
    defaultValues: {
      name: "",
      notes: "",
      websiteUrl: "",
      settingsConfig: "{}",
      icon: "",
      iconColor: "",
    },
  });

  return <Form {...form}>{children}</Form>;
}

describe("ClaudeConnectionFields", () => {
  it("renders shared API key and endpoint controls in direct mode", () => {
    const handleApiKeyChange = vi.fn();
    const handleEndpointChange = vi.fn();

    render(
      <ClaudeConnectionFields
        apiKey="sk-test"
        onApiKeyChange={handleApiKeyChange}
        baseUrl="https://api.example.com"
        onBaseUrlChange={handleEndpointChange}
        showEndpointTools={false}
      />,
      { wrapper: FormWrapper },
    );

    fireEvent.change(screen.getByLabelText("API Key"), {
      target: { value: "sk-next" },
    });
    fireEvent.change(screen.getByLabelText("providerForm.apiEndpoint"), {
      target: { value: "https://next.example.com" },
    });

    expect(handleApiKeyChange).toHaveBeenCalledWith("sk-next");
    expect(handleEndpointChange).toHaveBeenCalledWith(
      "https://next.example.com",
    );
    expect(
      screen.queryByRole("button", { name: "providerForm.manageAndTest" }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("switch", { name: "providerForm.fullUrlLabel" }),
    ).not.toBeInTheDocument();
  });
});
