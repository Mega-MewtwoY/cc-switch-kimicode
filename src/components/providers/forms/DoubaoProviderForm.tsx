import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import type { ProviderFormProps } from "./ProviderForm";

export function DoubaoProviderForm({
  submitLabel,
  onSubmit,
  onCancel,
  onSubmittingChange,
  onSubmitReadyChange,
  initialData,
  showButtons = true,
}: ProviderFormProps) {
  const [name, setName] = useState(initialData?.name ?? "豆包");
  const [websiteUrl, setWebsiteUrl] = useState(
    initialData?.websiteUrl ?? "https://www.doubao.com",
  );
  const [notes, setNotes] = useState(initialData?.notes ?? "");
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => onSubmitReadyChange?.(true), [onSubmitReadyChange]);
  useEffect(
    () => onSubmittingChange?.(submitting),
    [onSubmittingChange, submitting],
  );

  const submit = async (event: React.FormEvent) => {
    event.preventDefault();
    setSubmitting(true);
    try {
      await onSubmit({
        name: name.trim() || "豆包",
        websiteUrl: websiteUrl.trim() || "https://www.doubao.com",
        notes: notes.trim() || undefined,
        icon: "doubao",
        settingsConfig: JSON.stringify(initialData?.settingsConfig ?? {}),
      });
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <form onSubmit={submit} className="space-y-5">
      <div className="space-y-2">
        <Label htmlFor="doubao-name">显示名称</Label>
        <Input
          id="doubao-name"
          value={name}
          onChange={(event) => setName(event.target.value)}
        />
      </div>
      <div className="space-y-2">
        <Label htmlFor="doubao-website">额度页面</Label>
        <Input
          id="doubao-website"
          value={websiteUrl}
          onChange={(event) => setWebsiteUrl(event.target.value)}
        />
      </div>
      <div className="space-y-2">
        <Label htmlFor="doubao-notes">备注（可选）</Label>
        <Textarea
          id="doubao-notes"
          value={notes}
          onChange={(event) => setNotes(event.target.value)}
          rows={3}
        />
      </div>
      <p className="text-xs text-muted-foreground">
        登录态、套餐与额度由豆包桌面端管理；CC Switch 只允许修改显示信息。
      </p>
      {showButtons && (
        <div className="flex justify-end gap-2">
          <Button type="button" variant="outline" onClick={onCancel}>
            取消
          </Button>
          <Button type="submit" disabled={submitting}>
            {submitLabel}
          </Button>
        </div>
      )}
    </form>
  );
}
