import { useTranslation } from "react-i18next";
import { FormLabel } from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type {
  ResponsesCompactMode,
  UpstreamApiStyle,
  UpstreamPathConfig,
} from "@/types";

interface UpstreamPathStrategyFieldsProps {
  value: UpstreamPathConfig;
  onChange: (value: UpstreamPathConfig) => void;
  showChatPath?: boolean;
}

export function UpstreamPathStrategyFields({
  value,
  onChange,
  showChatPath = true,
}: UpstreamPathStrategyFieldsProps) {
  const { t } = useTranslation();

  const style = value.upstreamApiStyle ?? "standard_openai";
  const compactMode = value.responsesCompactMode ?? "synthetic";

  const update = (patch: Partial<UpstreamPathConfig>) => {
    onChange({ ...value, ...patch });
  };

  return (
    <div className="space-y-3 rounded-lg border border-border-default/60 p-4 bg-muted/20">
      <div className="space-y-2">
        <FormLabel>
          {t("providerForm.upstreamPathStrategy", {
            defaultValue: "上游路径策略",
          })}
        </FormLabel>
        <Select
          value={style}
          onValueChange={(next) =>
            update({ upstreamApiStyle: next as UpstreamApiStyle })
          }
        >
          <SelectTrigger>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="standard_openai">
              {t("providerForm.upstreamPathStandard", {
                defaultValue: "standard_openai",
              })}
            </SelectItem>
            <SelectItem value="prefixed_openai">
              {t("providerForm.upstreamPathPrefixed", {
                defaultValue: "prefixed_openai",
              })}
            </SelectItem>
            <SelectItem value="custom">
              {t("providerForm.upstreamPathCustom", {
                defaultValue: "custom",
              })}
            </SelectItem>
          </SelectContent>
        </Select>
        <p className="text-xs text-muted-foreground">
          {t("providerForm.upstreamPathStrategyHint", {
            defaultValue:
              "通过逻辑端点模板拼接上游 URL，避免继续依赖 base_url + endpoint 的固定假设。",
          })}
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div className="space-y-2">
          <FormLabel>
            {t("providerForm.upstreamPrefix", {
              defaultValue: "上游前缀",
            })}
          </FormLabel>
          <Input
            value={value.upstreamPrefix ?? ""}
            onChange={(e) => update({ upstreamPrefix: e.target.value })}
            placeholder="/openai"
          />
        </div>

        <div className="space-y-2">
          <FormLabel>
            {t("providerForm.openaiBasePath", {
              defaultValue: "OpenAI Base Path",
            })}
          </FormLabel>
          <Input
            value={value.openaiBasePath ?? ""}
            onChange={(e) => update({ openaiBasePath: e.target.value })}
            placeholder="/openai"
          />
        </div>

        <div className="space-y-2">
          <FormLabel>
            {t("providerForm.responsesPath", {
              defaultValue: "Responses Path",
            })}
          </FormLabel>
          <Input
            value={value.responsesPath ?? ""}
            onChange={(e) => update({ responsesPath: e.target.value })}
            placeholder="/v1/responses"
          />
        </div>

        <div className="space-y-2">
          <FormLabel>
            {t("providerForm.responsesCompactPath", {
              defaultValue: "Responses Compact Path",
            })}
          </FormLabel>
          <Input
            value={value.responsesCompactPath ?? ""}
            onChange={(e) => update({ responsesCompactPath: e.target.value })}
            placeholder="/v1/responses/compact"
          />
        </div>

        <div className="space-y-2">
          <FormLabel>
            {t("providerForm.responsesCompactMode", {
              defaultValue: "Responses Compact Mode",
            })}
          </FormLabel>
          <Select
            value={compactMode}
            onValueChange={(next) =>
              update({ responsesCompactMode: next as ResponsesCompactMode })
            }
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="native">
                {t("providerForm.responsesCompactModeNative", {
                  defaultValue: "native (prefer upstream)",
                })}
              </SelectItem>
              <SelectItem value="synthetic">
                {t("providerForm.responsesCompactModeSynthetic", {
                  defaultValue: "synthetic (local fallback)",
                })}
              </SelectItem>
              <SelectItem value="disabled">
                {t("providerForm.responsesCompactModeDisabled", {
                  defaultValue: "disabled",
                })}
              </SelectItem>
            </SelectContent>
          </Select>
        </div>

        {showChatPath && (
          <div className="space-y-2 md:col-span-2">
            <FormLabel>
              {t("providerForm.chatCompletionsPath", {
                defaultValue: "Chat Completions Path",
              })}
            </FormLabel>
            <Input
              value={value.chatCompletionsPath ?? ""}
              onChange={(e) => update({ chatCompletionsPath: e.target.value })}
              placeholder="/v1/chat/completions"
            />
          </div>
        )}
      </div>
    </div>
  );
}