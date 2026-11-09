import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Cpu, Zap, CheckCircle2, AlertCircle } from "lucide-react";
import { SettingContainer } from "../ui/SettingContainer";
import { Switch } from "../ui/Switch";
import { commands, type NpuStatus } from "@/bindings";
import { useSettings } from "../../hooks/useSettings";

interface NpuAcceleratorToggleProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const NpuAcceleratorToggle: React.FC<NpuAcceleratorToggleProps> = ({
  descriptionMode = "tooltip",
  grouped = false,
}) => {
  const { t } = useTranslation();
  const { refreshSettings } = useSettings();
  const [npuStatus, setNpuStatus] = useState<NpuStatus | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  const fetchStatus = async () => {
    try {
      const res = await commands.getNpuStatus();
      if (res.status === "ok") {
        setNpuStatus(res.data);
      } else {
        setErrorMsg(res.error);
      }
    } catch (e: any) {
      setErrorMsg(e?.message || String(e));
    }
  };

  useEffect(() => {
    fetchStatus();
  }, []);

  const handleToggle = async (enabled: boolean) => {
    setIsLoading(true);
    setErrorMsg(null);
    try {
      const res = await commands.setNpuEnabled(enabled);
      if (res.status === "ok") {
        setNpuStatus(res.data);
        await refreshSettings();
      } else {
        setErrorMsg(res.error);
      }
    } catch (e: any) {
      setErrorMsg(e?.message || String(e));
    } finally {
      setIsLoading(false);
    }
  };

  const isEnabled = npuStatus?.is_enabled ?? false;
  const isAvailable = npuStatus?.is_available ?? false;
  const modelFound = npuStatus?.model_found ?? false;

  return (
    <div className="flex flex-col gap-2">
      <SettingContainer
        title={t("settings.advanced.npu.title", "Ускорение на Intel NPU")}
        description={t(
          "settings.advanced.npu.description",
          "Использовать встроенный нейронный процессор Intel NPU (OpenVINO GenAI) для мгновенной расшифровки без нагрузки на CPU/GPU",
        )}
        descriptionMode={descriptionMode}
        grouped={grouped}
        disabled={!isAvailable && !isEnabled}
      >
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-1.5 text-xs">
            {isEnabled ? (
              <span className="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full font-medium bg-emerald-500/15 text-emerald-400 border border-emerald-500/30">
                <Zap className="w-3 h-3 text-emerald-400" />
                Intel NPU 3720
              </span>
            ) : (
              <span className="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full font-medium bg-wash text-ink-3 border border-line">
                <Cpu className="w-3 h-3 text-ink-3" />
                CPU
              </span>
            )}
          </div>
          <Switch
            checked={isEnabled}
            onChange={handleToggle}
            disabled={isLoading || (!isAvailable && !isEnabled)}
            ariaLabel={t("settings.advanced.npu.title", "Ускорение на Intel NPU")}
          />
        </div>
      </SettingContainer>

      {/* Sub-status line for model and runtime verification */}
      {isEnabled && (
        <div className="px-4 py-2 -mt-1 text-xs rounded-b-lg bg-surface/50 border-x border-b border-line flex flex-wrap items-center justify-between gap-2 text-ink-2">
          <div className="flex items-center gap-1.5">
            <CheckCircle2 className="w-3.5 h-3.5 text-emerald-400 shrink-0" />
            <span>
              {modelFound
                ? t("settings.advanced.npu.modelReady", "Модель Whisper Large v3 Turbo INT8 оптимизирована для NPU")
                : t("settings.advanced.npu.modelMissing", "Модель не найдена в /var/lib/ollama-models/asr/models/ov")}
            </span>
          </div>
          {npuStatus?.model_path && (
            <span className="text-[10px] text-ink-4 font-mono truncate max-w-[280px]">
              {npuStatus.model_path}
            </span>
          )}
        </div>
      )}

      {errorMsg && (
        <div className="flex items-center gap-2 p-2.5 text-xs text-error bg-error-wash rounded-lg border border-error/20">
          <AlertCircle className="w-4 h-4 shrink-0" />
          <span>{errorMsg}</span>
        </div>
      )}
    </div>
  );
};
