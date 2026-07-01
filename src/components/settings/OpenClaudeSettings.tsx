import { useTranslation } from "react-i18next";
import { ShieldAlert } from "lucide-react";
import { ToggleRow } from "@/components/ui/toggle-row";

export interface OpenClaudeSettingsProps {
  value?: boolean;
  onChange: (value: boolean) => void;
}

/// 「用 Claude Code 打开」右键命令是否附带 --dangerously-skip-permissions
export function OpenClaudeSettings({
  value,
  onChange,
}: OpenClaudeSettingsProps) {
  const { t } = useTranslation();

  return (
    <section className="space-y-3">
      <ToggleRow
        icon={<ShieldAlert className="h-4 w-4 text-amber-500" />}
        title={t("settings.openClaude.title")}
        description={t("settings.openClaude.description")}
        checked={!!value}
        onCheckedChange={onChange}
      />
    </section>
  );
}
