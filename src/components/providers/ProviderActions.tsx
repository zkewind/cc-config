import {
  BarChart3,
  Check,
  Copy,
  Edit,
  Loader2,
  Play,
  Terminal,
  TestTube2,
  Trash2,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import type { AppId } from "@/lib/api";

interface ProviderSwitchButtonProps {
  isCurrent: boolean;
  onSwitch: () => void;
}

export function ProviderSwitchButton({
  isCurrent,
  onSwitch,
}: ProviderSwitchButtonProps) {
  const { t } = useTranslation();
  return (
    <Button
      size="sm"
      variant={isCurrent ? "default" : "secondary"}
      onClick={isCurrent ? undefined : onSwitch}
      disabled={isCurrent}
      className={cn(
        "w-[4.5rem] px-2.5",
        isCurrent &&
          "bg-[#D97757] text-white hover:bg-[#D97757] hover:text-white disabled:opacity-100",
      )}
    >
      {isCurrent ? <Check className="h-4 w-4" /> : <Play className="h-4 w-4" />}
      {isCurrent ? t("provider.inUse") : t("provider.enable")}
    </Button>
  );
}

interface ProviderActionsProps {
  appId?: AppId;
  isCurrent: boolean;
  isInConfig?: boolean;
  isTesting?: boolean;
  onEdit: () => void;
  onDuplicate: () => void;
  onTest?: () => void;
  onConfigureUsage?: () => void;
  onDelete: () => void;
  onRemoveFromConfig?: () => void;
  onOpenTerminal?: () => void;
}

export function ProviderActions({
  isCurrent,
  isTesting,
  onEdit,
  onDuplicate,
  onTest,
  onConfigureUsage,
  onDelete,
  onOpenTerminal,
}: ProviderActionsProps) {
  const { t } = useTranslation();
  const iconButtonClass = "h-8 w-8 p-1";
  const canDelete = !isCurrent;

  return (
    <div className="flex items-center gap-1">
      <Button
        size="icon"
        variant="ghost"
        onClick={onEdit}
        title={t("common.edit")}
        className={iconButtonClass}
      >
        <Edit className="h-4 w-4" />
      </Button>

      <Button
        size="icon"
        variant="ghost"
        onClick={onDuplicate}
        title={t("provider.duplicate")}
        className={iconButtonClass}
      >
        <Copy className="h-4 w-4" />
      </Button>

      <Button
        size="icon"
        variant="ghost"
        onClick={onTest || undefined}
        disabled={isTesting}
        title={t("modelTest.testProvider", "测试模型")}
        className={cn(
          iconButtonClass,
          !onTest && "opacity-40 cursor-not-allowed text-muted-foreground",
        )}
      >
        {isTesting ? (
          <Loader2 className="h-4 w-4 animate-spin" />
        ) : (
          <TestTube2 className="h-4 w-4" />
        )}
      </Button>

      <Button
        size="icon"
        variant="ghost"
        onClick={onConfigureUsage || undefined}
        title={t("provider.configureUsage")}
        className={cn(
          iconButtonClass,
          !onConfigureUsage &&
            "opacity-40 cursor-not-allowed text-muted-foreground",
        )}
      >
        <BarChart3 className="h-4 w-4" />
      </Button>

      {onOpenTerminal && (
        <Button
          size="icon"
          variant="ghost"
          onClick={onOpenTerminal}
          title={t("provider.openTerminal", "打开终端")}
          className={cn(
            iconButtonClass,
            "hover:text-emerald-600 dark:hover:text-emerald-400",
          )}
        >
          <Terminal className="h-4 w-4" />
        </Button>
      )}

      <Button
        size="icon"
        variant="ghost"
        onClick={canDelete ? onDelete : undefined}
        title={t("common.delete")}
        className={cn(
          iconButtonClass,
          canDelete && "hover:text-red-500 dark:hover:text-red-400",
          !canDelete && "opacity-40 cursor-not-allowed text-muted-foreground",
        )}
      >
        <Trash2 className="h-4 w-4" />
      </Button>
    </div>
  );
}
