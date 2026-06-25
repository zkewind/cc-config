import { useEffect, useMemo, useState, useCallback } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Form, FormField, FormItem, FormMessage } from "@/components/ui/form";
import { providerSchema, type ProviderFormData } from "@/lib/schemas/provider";
import { settingsApi, type AppId } from "@/lib/api";
import type {
  ProviderCategory,
  ProviderMeta,
  ProviderTestConfig,
  ClaudeApiFormat,
  ClaudeApiKeyField,
} from "@/types";
import {
  providerPresets,
  type ProviderPreset,
} from "@/config/claudeProviderPresets";
import {
  applyTemplateValues,
  hasApiKeyField,
} from "@/utils/providerConfigUtils";
import { mergeProviderMeta } from "@/utils/providerMetaUtils";
import { CommonConfigEditor } from "./CommonConfigEditor";
import { ProviderPresetSelector } from "./ProviderPresetSelector";
import { BasicFormFields } from "./BasicFormFields";
import { ClaudeFormFields } from "./ClaudeFormFields";
import {
  ProviderAdvancedConfig,
  type PricingModelSourceOption,
} from "./ProviderAdvancedConfig";
import {
  useProviderCategory,
  useApiKeyState,
  useBaseUrlState,
  useModelState,
  useApiKeyLink,
  useTemplateValues,
  useCommonConfigSnippet,
  useSpeedTestEndpoints,
} from "./hooks";
import { ConfirmDialog } from "@/components/ConfirmDialog";
import { useSettingsQuery } from "@/lib/query";
import {
  CLAUDE_DEFAULT_CONFIG,
  normalizePricingSource,
} from "./helpers/providerFormUtils";

type PresetEntry = {
  id: string;
  preset: ProviderPreset;
};

export interface ProviderFormProps {
  appId: AppId;
  providerId?: string;
  submitLabel: string;
  onSubmit: (values: ProviderFormValues) => Promise<void> | void;
  onCancel: () => void;
  onSubmittingChange?: (isSubmitting: boolean) => void;
  initialData?: {
    name?: string;
    websiteUrl?: string;
    notes?: string;
    settingsConfig?: Record<string, unknown>;
    category?: ProviderCategory;
    meta?: ProviderMeta;
    icon?: string;
    iconColor?: string;
  };
  showButtons?: boolean;
}

export function ProviderForm(props: ProviderFormProps) {
  return <ProviderFormFull {...props} />;
}

function ProviderFormFull({
  appId,
  providerId,
  submitLabel,
  onSubmit,
  onCancel,
  onSubmittingChange,
  initialData,
  showButtons = true,
}: ProviderFormProps) {
  const { t } = useTranslation();
  const isEditMode = Boolean(initialData);
  const queryClient = useQueryClient();
  const { data: settingsData } = useSettingsQuery();
  const showCommonConfigNotice =
    settingsData != null && settingsData.commonConfigConfirmed !== true;

  const handleCommonConfigConfirm = async () => {
    try {
      if (settingsData) {
        const { webdavSync: _, ...rest } = settingsData;
        await settingsApi.save({ ...rest, commonConfigConfirmed: true });
        await queryClient.invalidateQueries({ queryKey: ["settings"] });
      }
    } catch (error) {
      console.error("Failed to save commonConfigConfirmed:", error);
    }
  };

  const [selectedPresetId, setSelectedPresetId] = useState<string | null>(
    initialData ? null : "custom",
  );
  const [activePreset, setActivePreset] = useState<{
    id: string;
    category?: ProviderCategory;
    isPartner?: boolean;
    partnerPromotionKey?: string;
  } | null>(null);
  const [isEndpointModalOpen, setIsEndpointModalOpen] = useState(false);

  const [draftCustomEndpoints, setDraftCustomEndpoints] = useState<string[]>(
    () => {
      if (initialData) return [];
      return [];
    },
  );
  const [endpointAutoSelect, setEndpointAutoSelect] = useState<boolean>(
    () => initialData?.meta?.endpointAutoSelect ?? true,
  );
  const supportsFullUrl = true;
  const [localIsFullUrl, setLocalIsFullUrl] = useState<boolean>(() => {
    if (!supportsFullUrl) return false;
    return initialData?.meta?.isFullUrl ?? false;
  });

  const [testConfig, setTestConfig] = useState<ProviderTestConfig>(
    () => initialData?.meta?.testConfig ?? { enabled: false },
  );
  const [pricingConfig, setPricingConfig] = useState<{
    enabled: boolean;
    costMultiplier?: string;
    pricingModelSource: PricingModelSourceOption;
  }>(() => ({
    enabled:
      initialData?.meta?.costMultiplier !== undefined ||
      initialData?.meta?.pricingModelSource !== undefined,
    costMultiplier: initialData?.meta?.costMultiplier,
    pricingModelSource: normalizePricingSource(
      initialData?.meta?.pricingModelSource,
    ),
  }));

  const { category } = useProviderCategory({
    appId,
    selectedPresetId,
    isEditMode,
    initialCategory: initialData?.category,
  });

  useEffect(() => {
    setSelectedPresetId(initialData ? null : "custom");
    setActivePreset(null);

    if (!initialData) {
      setDraftCustomEndpoints([]);
    }
    setEndpointAutoSelect(initialData?.meta?.endpointAutoSelect ?? true);
    setLocalIsFullUrl(
      supportsFullUrl ? (initialData?.meta?.isFullUrl ?? false) : false,
    );
    setTestConfig(initialData?.meta?.testConfig ?? { enabled: false });
    setPricingConfig({
      enabled:
        initialData?.meta?.costMultiplier !== undefined ||
        initialData?.meta?.pricingModelSource !== undefined,
      costMultiplier: initialData?.meta?.costMultiplier,
      pricingModelSource: normalizePricingSource(
        initialData?.meta?.pricingModelSource,
      ),
    });
  }, [appId, initialData, supportsFullUrl]);

  const defaultValues: ProviderFormData = useMemo(
    () => ({
      name: initialData?.name ?? "",
      websiteUrl: initialData?.websiteUrl ?? "",
      notes: initialData?.notes ?? "",
      settingsConfig: initialData?.settingsConfig
        ? JSON.stringify(initialData.settingsConfig, null, 2)
        : CLAUDE_DEFAULT_CONFIG,
      icon: initialData?.icon ?? "",
      iconColor: initialData?.iconColor ?? "",
    }),
    [initialData, appId],
  );

  const form = useForm<ProviderFormData>({
    resolver: zodResolver(providerSchema),
    defaultValues,
    mode: "onSubmit",
  });
  const { isSubmitting } = form.formState;

  const handleSettingsConfigChange = useCallback(
    (config: string) => {
      form.setValue("settingsConfig", config);
    },
    [form],
  );

  const [localApiKeyField, setLocalApiKeyField] = useState<ClaudeApiKeyField>(
    () => {
      if (appId !== "claude") return "ANTHROPIC_AUTH_TOKEN";
      if (initialData?.meta?.apiKeyField) return initialData.meta.apiKeyField;
      // Infer from existing config env
      const env = (initialData?.settingsConfig as Record<string, unknown>)
        ?.env as Record<string, unknown> | undefined;
      if (env?.ANTHROPIC_API_KEY !== undefined) return "ANTHROPIC_API_KEY";
      return "ANTHROPIC_AUTH_TOKEN";
    },
  );

  // 软校验：收集"业务约束"类问题（空值/缺项），由用户决定是否仍要保存
  const [softIssues, setSoftIssues] = useState<string[] | null>(null);
  const [pendingFormValues, setPendingFormValues] =
    useState<ProviderFormData | null>(null);
  // 确认框走的提交路径绕过了 react-hook-form 的 isSubmitting，单独追踪
  const [isConfirmSubmitting, setIsConfirmSubmitting] = useState(false);

  useEffect(() => {
    onSubmittingChange?.(isSubmitting || isConfirmSubmitting);
  }, [isSubmitting, isConfirmSubmitting, onSubmittingChange]);

  const {
    apiKey,
    handleApiKeyChange,
    showApiKey: shouldShowApiKey,
  } = useApiKeyState({
    initialConfig: form.getValues("settingsConfig"),
    onConfigChange: handleSettingsConfigChange,
    selectedPresetId,
    category,
    apiKeyField: localApiKeyField,
  });

  const { baseUrl, handleClaudeBaseUrlChange } = useBaseUrlState({
    appType: appId,
    category,
    settingsConfig: form.getValues("settingsConfig"),
    onSettingsConfigChange: handleSettingsConfigChange,
  });

  const {
    claudeModel,
    defaultHaikuModel,
    defaultSonnetModel,
    defaultOpusModel,
    handleModelChange,
  } = useModelState({
    settingsConfig: form.getValues("settingsConfig"),
    onConfigChange: handleSettingsConfigChange,
  });

  const [localApiFormat, setLocalApiFormat] = useState<ClaudeApiFormat>(() => {
    if (appId !== "claude") return "anthropic";
    return initialData?.meta?.apiFormat ?? "anthropic";
  });

  const handleApiFormatChange = useCallback((format: ClaudeApiFormat) => {
    setLocalApiFormat(format);
  }, []);

  const handleApiKeyFieldChange = useCallback(
    (field: ClaudeApiKeyField) => {
      const prev = localApiKeyField;
      setLocalApiKeyField(field);

      // Swap the env key name in settingsConfig
      try {
        const raw = form.getValues("settingsConfig");
        const config = JSON.parse(raw || "{}");
        if (config?.env && prev in config.env) {
          const value = config.env[prev];
          delete config.env[prev];
          config.env[field] = value;
          const updated = JSON.stringify(config, null, 2);
          form.setValue("settingsConfig", updated);
          handleSettingsConfigChange(updated);
        }
      } catch {
        // ignore parse errors during editing
      }
    },
    [localApiKeyField, form, handleSettingsConfigChange],
  );

  useEffect(() => {
    form.reset(defaultValues);
  }, [defaultValues, form]);

  const presetCategoryLabels: Record<string, string> = useMemo(
    () => ({
      official: t("providerForm.categoryOfficial", {
        defaultValue: "官方",
      }),
      cn_official: t("providerForm.categoryCnOfficial", {
        defaultValue: "国内官方",
      }),
      aggregator: t("providerForm.categoryAggregation", {
        defaultValue: "聚合服务",
      }),
      third_party: t("providerForm.categoryThirdParty", {
        defaultValue: "第三方",
      }),
    }),
    [t],
  );

  const presetEntries = useMemo(() => {
    return providerPresets
      .filter((p) => !p.hidden)
      .map<PresetEntry>((preset, index) => ({
        id: `claude-${index}`,
        preset,
      }));
  }, []);

  const {
    templateValues,
    templateValueEntries,
    selectedPreset: templatePreset,
    handleTemplateValueChange,
    validateTemplateValues,
  } = useTemplateValues({
    selectedPresetId,
    presetEntries,
    settingsConfig: form.getValues("settingsConfig"),
    onConfigChange: handleSettingsConfigChange,
  });

  const {
    useCommonConfig,
    commonConfigSnippet,
    commonConfigError,
    handleCommonConfigToggle,
    handleCommonConfigSnippetChange,
    isExtracting: isClaudeExtracting,
    handleExtract: handleClaudeExtract,
  } = useCommonConfigSnippet({
    settingsConfig: form.getValues("settingsConfig"),
    onConfigChange: handleSettingsConfigChange,
    initialData,
    initialEnabled: initialData?.meta?.commonConfigEnabled,
    selectedPresetId: selectedPresetId ?? undefined,
    enabled: true,
  });

  const [isCommonConfigModalOpen, setIsCommonConfigModalOpen] = useState(false);

  const handleSubmit = async (values: ProviderFormData) => {
    // 硬性校验：非官方 Claude 供应商必须填写端点和 API Key，不可绕过
    if (
      appId === "claude" &&
      category !== "official" &&
      category !== "cloud_provider"
    ) {
      if (!baseUrl.trim()) {
        toast.error(
          t("providerForm.endpointRequired", {
            defaultValue: "非官方供应商请填写 API 端点",
          }),
        );
        return;
      }
      if (!apiKey.trim()) {
        toast.error(
          t("providerForm.apiKeyRequired", {
            defaultValue: "非官方供应商请填写 API Key",
          }),
        );
        return;
      }
    }

    // 名称为空时根据 base_url 域名 + key 末两位 + 备注自动生成
    if (!values.name.trim()) {
      try {
        const parsed = new URL(baseUrl.trim());
        const domain = parsed.host; // 含自定义端口，如 localhost:3000
        const keyTail = apiKey.trim().slice(-2);
        const note = values.notes?.trim();
        const noteSuffix = note ? `_${note}` : "";
        values = { ...values, name: `${domain}_*${keyTail}${noteSuffix}` };
        form.setValue("name", values.name);
      } catch {
        // base_url 无效时保持空名称，后续软校验会提示
      }
    }

    // 软性问题（业务约束，用户可选择仍要保存）
    const issues: string[] = [];

    // 模板变量未填：A 类（空值）
    if (appId === "claude" && templateValueEntries.length > 0) {
      const validation = validateTemplateValues();
      if (!validation.isValid && validation.missingField) {
        issues.push(
          t("providerForm.fillParameter", {
            label: validation.missingField.label,
            defaultValue: `请填写 ${validation.missingField.label}`,
          }),
        );
      }
    }

    // 名称仍为空（自动生成失败时）
    if (!values.name.trim()) {
      issues.push(
        t("providerForm.fillSupplierName", {
          defaultValue: "请填写供应商名称",
        }),
      );
    }

    if (issues.length > 0) {
      // 弹确认框让用户决定是否仍要保存
      setSoftIssues(issues);
      setPendingFormValues(values);
      return;
    }

    await performSubmit(values);
  };

  const performSubmit = async (values: ProviderFormData) => {
    let settingsConfig: string;

    settingsConfig = values.settingsConfig.trim();

    const payload: ProviderFormValues = {
      ...values,
      name: values.name.trim(),
      websiteUrl: values.websiteUrl?.trim() ?? "",
      settingsConfig,
    };

    if (activePreset) {
      payload.presetId = activePreset.id;
      if (activePreset.category) {
        payload.presetCategory = activePreset.category;
      }
      if (activePreset.isPartner) {
        payload.isPartner = activePreset.isPartner;
      }
    }

    if (!isEditMode && draftCustomEndpoints.length > 0) {
      const customEndpointsToSave: Record<
        string,
        import("@/types").CustomEndpoint
      > = draftCustomEndpoints.reduce(
        (acc, url) => {
          const now = Date.now();
          acc[url] = { url, addedAt: now, lastUsed: undefined };
          return acc;
        },
        {} as Record<string, import("@/types").CustomEndpoint>,
      );

      const hadEndpoints =
        initialData?.meta?.custom_endpoints &&
        Object.keys(initialData.meta.custom_endpoints).length > 0;
      const needsClearEndpoints =
        hadEndpoints && draftCustomEndpoints.length === 0;

      let mergedMeta = needsClearEndpoints
        ? mergeProviderMeta(initialData?.meta, {})
        : mergeProviderMeta(initialData?.meta, customEndpointsToSave);

      if (activePreset?.isPartner) {
        mergedMeta = {
          ...(mergedMeta ?? {}),
          isPartner: true,
        };
      }

      if (activePreset?.partnerPromotionKey) {
        mergedMeta = {
          ...(mergedMeta ?? {}),
          partnerPromotionKey: activePreset.partnerPromotionKey,
        };
      }

      if (mergedMeta !== undefined) {
        payload.meta = mergedMeta;
      }
    }

    const baseMeta: ProviderMeta | undefined =
      payload.meta ?? (initialData?.meta ? { ...initialData.meta } : undefined);

    // 确定 providerType（新建时从预设获取，编辑时从现有数据获取）
    const providerType =
      templatePreset?.providerType || initialData?.meta?.providerType;

    const nextMeta: ProviderMeta = {
      ...(baseMeta ?? {}),
      commonConfigEnabled: useCommonConfig,
      endpointAutoSelect,
      // 保存 providerType（用于识别特殊供应商）
      providerType,
      testConfig: testConfig.enabled ? testConfig : undefined,
      costMultiplier: pricingConfig.enabled
        ? pricingConfig.costMultiplier
        : undefined,
      pricingModelSource:
        pricingConfig.enabled && pricingConfig.pricingModelSource !== "inherit"
          ? pricingConfig.pricingModelSource
          : undefined,
      apiFormat:
        appId === "claude" && category !== "official"
          ? localApiFormat
          : undefined,
      apiKeyField:
        appId === "claude" &&
        category !== "official" &&
        localApiKeyField !== "ANTHROPIC_AUTH_TOKEN"
          ? localApiKeyField
          : undefined,
      isFullUrl:
        supportsFullUrl && category !== "official" && localIsFullUrl
          ? true
          : undefined,
    };

    payload.meta = nextMeta;

    await onSubmit(payload);
  };

  const groupedPresets = useMemo(() => {
    return presetEntries.reduce<Record<string, PresetEntry[]>>((acc, entry) => {
      const category = entry.preset.category ?? "others";
      if (!acc[category]) {
        acc[category] = [];
      }
      acc[category].push(entry);
      return acc;
    }, {});
  }, [presetEntries]);

  const categoryKeys = useMemo(() => {
    return Object.keys(groupedPresets).filter(
      (key) => key !== "custom" && groupedPresets[key]?.length,
    );
  }, [groupedPresets]);

  const shouldShowSpeedTest =
    category !== "official" && category !== "cloud_provider";

  const {
    shouldShowApiKeyLink: shouldShowClaudeApiKeyLink,
    websiteUrl: claudeWebsiteUrl,
    isPartner: isClaudePartner,
    partnerPromotionKey: claudePartnerPromotionKey,
  } = useApiKeyLink({
    appId: "claude",
    category,
    selectedPresetId,
    presetEntries,
    formWebsiteUrl: form.watch("websiteUrl") || "",
  });

  const speedTestEndpoints = useSpeedTestEndpoints({
    appId,
    selectedPresetId,
    presetEntries,
    baseUrl,
    initialData,
  });

  const handlePresetChange = (value: string) => {
    setSelectedPresetId(value);
    if (value === "custom") {
      setActivePreset(null);
      form.reset(defaultValues);
      return;
    }

    const entry = presetEntries.find((item) => item.id === value);
    if (!entry) {
      return;
    }

    setActivePreset({
      id: value,
      category: entry.preset.category,
      isPartner: entry.preset.isPartner,
      partnerPromotionKey: entry.preset.partnerPromotionKey,
    });

    const preset = entry.preset as ProviderPreset;
    const config = applyTemplateValues(
      preset.settingsConfig,
      preset.templateValues,
    );

    if (preset.apiFormat) {
      setLocalApiFormat(preset.apiFormat);
    } else {
      setLocalApiFormat("anthropic");
    }

    setLocalApiKeyField(preset.apiKeyField ?? "ANTHROPIC_AUTH_TOKEN");
    setLocalIsFullUrl(false);

    form.reset({
      name: preset.nameKey ? t(preset.nameKey) : preset.name,
      websiteUrl: preset.websiteUrl ?? "",
      settingsConfig: JSON.stringify(config, null, 2),
      icon: preset.icon ?? "",
      iconColor: preset.iconColor ?? "",
    });
  };

  const settingsConfigErrorField = (
    <FormField
      control={form.control}
      name="settingsConfig"
      render={() => (
        <FormItem className="space-y-0">
          <FormMessage />
        </FormItem>
      )}
    />
  );

  return (
    <>
      <Form {...form}>
        <form
          id="provider-form"
          onSubmit={form.handleSubmit(handleSubmit)}
          className="space-y-6 glass rounded-xl p-6 border border-white/10"
        >
          {!initialData && (
            <ProviderPresetSelector
              selectedPresetId={selectedPresetId}
              groupedPresets={groupedPresets}
              categoryKeys={categoryKeys}
              presetCategoryLabels={presetCategoryLabels}
              onPresetChange={handlePresetChange}
              category={category}
            />
          )}

          <BasicFormFields form={form} />

          {appId === "claude" && (
            <ClaudeFormFields
              providerId={providerId}
              shouldShowApiKey={
                (category !== "cloud_provider" ||
                  hasApiKeyField(form.getValues("settingsConfig"))) &&
                shouldShowApiKey(form.getValues("settingsConfig"), isEditMode)
              }
              apiKey={apiKey}
              onApiKeyChange={handleApiKeyChange}
              category={category}
              shouldShowApiKeyLink={shouldShowClaudeApiKeyLink}
              websiteUrl={claudeWebsiteUrl}
              isPartner={isClaudePartner}
              partnerPromotionKey={claudePartnerPromotionKey}
              templateValueEntries={templateValueEntries}
              templateValues={templateValues}
              templatePresetName={templatePreset?.name || ""}
              onTemplateValueChange={handleTemplateValueChange}
              shouldShowSpeedTest={shouldShowSpeedTest}
              baseUrl={baseUrl}
              onBaseUrlChange={handleClaudeBaseUrlChange}
              isEndpointModalOpen={isEndpointModalOpen}
              onEndpointModalToggle={setIsEndpointModalOpen}
              onCustomEndpointsChange={
                isEditMode ? undefined : setDraftCustomEndpoints
              }
              autoSelect={endpointAutoSelect}
              onAutoSelectChange={setEndpointAutoSelect}
              showEndpointTools
              shouldShowModelSelector={category !== "official"}
              claudeModel={claudeModel}
              defaultHaikuModel={defaultHaikuModel}
              defaultSonnetModel={defaultSonnetModel}
              defaultOpusModel={defaultOpusModel}
              onModelChange={handleModelChange}
              speedTestEndpoints={speedTestEndpoints}
              apiFormat={localApiFormat}
              onApiFormatChange={handleApiFormatChange}
              apiKeyField={localApiKeyField}
              onApiKeyFieldChange={handleApiKeyFieldChange}
              isFullUrl={localIsFullUrl}
              onFullUrlChange={setLocalIsFullUrl}
            />
          )}

          {/* 配置编辑器 */}
          <>
            <CommonConfigEditor
              value={form.getValues("settingsConfig")}
              onChange={(value) => form.setValue("settingsConfig", value)}
              useCommonConfig={useCommonConfig}
              onCommonConfigToggle={handleCommonConfigToggle}
              commonConfigSnippet={commonConfigSnippet}
              onCommonConfigSnippetChange={handleCommonConfigSnippetChange}
              commonConfigError={commonConfigError}
              onEditClick={() => setIsCommonConfigModalOpen(true)}
              isModalOpen={isCommonConfigModalOpen}
              onModalClose={() => setIsCommonConfigModalOpen(false)}
              onExtract={handleClaudeExtract}
              isExtracting={isClaudeExtracting}
            />
            {settingsConfigErrorField}
          </>

          <ProviderAdvancedConfig
            testConfig={testConfig}
            pricingConfig={pricingConfig}
            onTestConfigChange={setTestConfig}
            onPricingConfigChange={setPricingConfig}
          />

          {showButtons && (
            <div className="flex justify-end gap-2">
              <Button variant="outline" type="button" onClick={onCancel}>
                {t("common.cancel")}
              </Button>
              <Button
                type="submit"
                disabled={isSubmitting || isConfirmSubmitting}
              >
                {submitLabel}
              </Button>
            </div>
          )}
        </form>
      </Form>

      <ConfirmDialog
        isOpen={showCommonConfigNotice}
        variant="info"
        title={t("confirm.commonConfig.title")}
        message={t("confirm.commonConfig.message")}
        confirmText={t("confirm.commonConfig.confirm")}
        onConfirm={() => void handleCommonConfigConfirm()}
        onCancel={() => void handleCommonConfigConfirm()}
      />

      <ConfirmDialog
        isOpen={softIssues !== null && softIssues.length > 0}
        variant="info"
        title={t("providerForm.softValidation.title", {
          defaultValue: "配置存在以下问题",
        })}
        message={
          (softIssues ?? []).map((issue) => `• ${issue}`).join("\n") +
          "\n\n" +
          t("providerForm.softValidation.hint", {
            defaultValue:
              "仍要保存吗？保存后切换此供应商时可能失败，可以之后再补全。",
          })
        }
        confirmText={t("providerForm.softValidation.saveAnyway", {
          defaultValue: "仍要保存",
        })}
        cancelText={t("common.cancel")}
        onConfirm={async () => {
          if (isConfirmSubmitting) return;
          const values = pendingFormValues;
          if (!values) {
            setSoftIssues(null);
            return;
          }
          setIsConfirmSubmitting(true);
          try {
            await performSubmit(values);
            setSoftIssues(null);
            setPendingFormValues(null);
          } catch (error) {
            console.error("[ProviderForm] soft-confirm submit failed:", error);
            // 保留确认框和 pending values，让用户可以重试或取消
          } finally {
            setIsConfirmSubmitting(false);
          }
        }}
        onCancel={() => {
          if (isConfirmSubmitting) return;
          setSoftIssues(null);
          setPendingFormValues(null);
        }}
      />
    </>
  );
}

export type ProviderFormValues = ProviderFormData & {
  presetId?: string;
  presetCategory?: ProviderCategory;
  isPartner?: boolean;
  meta?: ProviderMeta;
  providerKey?: string;
};
