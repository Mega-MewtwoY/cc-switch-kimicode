import { invoke } from "@tauri-apps/api/core";
import type { SubscriptionQuota } from "@/types/subscription";

export const subscriptionApi = {
  // providerId（仅 codex）：按供应商条目查询——当前条目用 live 凭据，
  // 其他条目用各自的 auth 快照，多个 OpenAI 账号可同时查各自的额度。
  getQuota: (tool: string, providerId?: string): Promise<SubscriptionQuota> =>
    invoke("get_subscription_quota", { tool, providerId }),
  getCodexOauthQuota: (accountId: string | null): Promise<SubscriptionQuota> =>
    invoke("get_codex_oauth_quota", { accountId }),
  getXaiOauthQuota: (accountId: string | null): Promise<SubscriptionQuota> =>
    invoke("get_xai_oauth_quota", { accountId }),
  getCodingPlanQuota: (
    baseUrl: string,
    apiKey: string,
    // 火山方舟用账号 AK/SK 签名查询用量；其他供应商不传。
    accessKeyId?: string,
    secretAccessKey?: string,
    // 智谱团队版（zhipu_team）靠显式标识路由（base_url 与个人版相同无法区分）。
    codingPlanProvider?: string,
    teamOrganizationId?: string,
    teamProjectId?: string,
  ): Promise<SubscriptionQuota> =>
    invoke("get_coding_plan_quota", {
      baseUrl,
      apiKey,
      accessKeyId,
      secretAccessKey,
      codingPlanProvider,
      teamOrganizationId,
      teamProjectId,
    }),
  getBalance: (
    baseUrl: string,
    apiKey: string,
  ): Promise<import("@/types").UsageResult> =>
    invoke("get_balance", { baseUrl, apiKey }),
};
