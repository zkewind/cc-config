import React from "react";
import { cn } from "@/lib/utils";
import type { HealthStatus } from "@/lib/api/model-test";
import { useTranslation } from "react-i18next";

interface HealthStatusIndicatorProps {
  status: HealthStatus;
  responseTimeMs?: number;
  className?: string;
}

const statusConfig = {
  operational: {
    color: "bg-emerald-500",
    labelKey: "health.operational",
    labelFallback: "正常",
    textColor: "text-emerald-600 dark:text-emerald-400",
    animate: false,
  },
  degraded: {
    color: "bg-yellow-500",
    labelKey: "health.degraded",
    labelFallback: "降级",
    textColor: "text-yellow-600 dark:text-yellow-400",
    animate: false,
  },
  failed: {
    color: "bg-red-500",
    labelKey: "health.failed",
    labelFallback: "失败",
    textColor: "text-red-600 dark:text-red-400",
    animate: false,
  },
  checking: {
    color: "bg-blue-400",
    labelKey: "health.checking",
    labelFallback: "检测中",
    textColor: "text-blue-500 dark:text-blue-400",
    animate: true,
  },
};

export const HealthStatusIndicator: React.FC<HealthStatusIndicatorProps> = ({
  status,
  responseTimeMs,
  className,
}) => {
  const { t } = useTranslation();
  const config = statusConfig[status];
  const label = t(config.labelKey, { defaultValue: config.labelFallback });

  return (
    <div className={cn("flex items-center gap-2", className)}>
      <div
        className={cn(
          "w-2 h-2 rounded-full",
          config.color,
          config.animate && "animate-pulse",
        )}
      />
      <span className={cn("text-xs font-medium", config.textColor)}>
        {label}
        {responseTimeMs !== undefined && ` (${responseTimeMs}ms)`}
      </span>
    </div>
  );
};
