import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Keyboard, CheckCircle2, AlertTriangle, RefreshCw } from "lucide-react";
import { Button } from "../ui/Button";
import { commands, type NiriStatus } from "@/bindings";

export const NiriShortcutCard: React.FC = () => {
  const { t } = useTranslation();
  const [niriStatus, setNiriStatus] = useState<NiriStatus | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [feedback, setFeedback] = useState<{ type: "success" | "error"; text: string } | null>(null);

  const fetchStatus = async () => {
    try {
      const status = await commands.checkNiriStatus();
      setNiriStatus(status);
    } catch (e: any) {
      console.warn("Niri check failed:", e);
    }
  };

  useEffect(() => {
    fetchStatus();
  }, []);

  if (!niriStatus || !niriStatus.is_niri) {
    return null;
  }

  const handleInstall = async () => {
    setIsLoading(true);
    setFeedback(null);
    try {
      const res = await commands.installNiriShortcut("Ctrl+Space");
      if (res.status === "ok") {
        setNiriStatus(res.data);
        setFeedback({
          type: "success",
          text: t("settings.niri.installedSuccess", "Сочетание Ctrl+Space успешно привязано в Niri и конфиг перезагружен!"),
        });
      } else {
        setFeedback({
          type: "error",
          text: res.error,
        });
      }
    } catch (e: any) {
      setFeedback({
        type: "error",
        text: e?.message || String(e),
      });
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="p-4 rounded-xl border border-line bg-surface/80 flex flex-col gap-3">
      <div className="flex items-start justify-between gap-3">
        <div className="flex items-start gap-3">
          <div className="p-2 rounded-lg bg-logo-primary/10 text-logo-primary mt-0.5">
            <Keyboard className="w-5 h-5" />
          </div>
          <div className="space-y-1">
            <div className="flex items-center gap-2">
              <span className="font-medium text-sm text-text">
                {t("settings.niri.title", "Интеграция с Niri Wayland")}
              </span>
              {niriStatus.is_configured ? (
                <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[11px] font-medium bg-emerald-500/15 text-emerald-400 border border-emerald-500/30">
                  <CheckCircle2 className="w-3 h-3 text-emerald-400" />
                  {niriStatus.shortcut ?? "Ctrl+Space"} активен
                </span>
              ) : (
                <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[11px] font-medium bg-warning-wash text-warning border border-warning/30">
                  <AlertTriangle className="w-3 h-3 text-warning" />
                  Требуется привязка
                </span>
              )}
            </div>
            <p className="text-xs text-ink-3">
              {niriStatus.is_configured
                ? t(
                    "settings.niri.configuredDesc",
                    "Глобальный вызов Handy привязан к композитору Niri (my-binds.kdl) через spawn 'handy' '--toggle-transcription'.",
                  )
                : t(
                    "settings.niri.unconfiguredDesc",
                    "В Niri Wayland глобальные хоткеи обрабатываются композитором. Нажмите кнопку ниже, чтобы автоматически зарегистрировать Ctrl+Space.",
                  )}
            </p>
          </div>
        </div>

        <Button
          variant={niriStatus.is_configured ? "secondary" : "primary"}
          size="sm"
          onClick={handleInstall}
          disabled={isLoading}
          className="shrink-0"
        >
          {isLoading ? (
            <RefreshCw className="w-3.5 h-3.5 animate-spin mr-1" />
          ) : null}
          {niriStatus.is_configured
            ? t("settings.niri.updateBtn", "Обновить бинд")
            : t("settings.niri.installBtn", "Подключить Ctrl+Space")}
        </Button>
      </div>

      {feedback && (
        <div
          className={`p-2.5 rounded-lg text-xs flex items-center gap-2 ${
            feedback.type === "success"
              ? "bg-emerald-500/10 text-emerald-400 border border-emerald-500/20"
              : "bg-error-wash text-error border border-error/20"
          }`}
        >
          {feedback.type === "success" ? (
            <CheckCircle2 className="w-4 h-4 shrink-0" />
          ) : (
            <AlertTriangle className="w-4 h-4 shrink-0" />
          )}
          <span>{feedback.text}</span>
        </div>
      )}
    </div>
  );
};
