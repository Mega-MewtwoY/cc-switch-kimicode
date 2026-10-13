import { QueryClientProvider } from "@tanstack/react-query";
import { render } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ProviderCard } from "@/components/providers/ProviderCard";
import type { Provider } from "@/types";
import { createTestQueryClient } from "../utils/testQueryClient";

const subscriptionFooterProps = vi.hoisted(() => vi.fn());

vi.mock("@/components/providers/ProviderActions", () => ({
  ProviderActions: () => null,
}));
vi.mock("@/components/ProviderIcon", () => ({ ProviderIcon: () => null }));
vi.mock("@/components/UsageFooter", () => ({ default: () => null }));
vi.mock("@/components/SubscriptionQuotaFooter", () => ({
  default: (props: unknown) => {
    subscriptionFooterProps(props);
    return <div>subscription-quota-footer</div>;
  },
}));
vi.mock("@/components/CopilotQuotaFooter", () => ({ default: () => null }));
vi.mock("@/components/CodexOauthQuotaFooter", () => ({ default: () => null }));
vi.mock("@/components/XaiOauthQuotaFooter", () => ({ default: () => null }));
vi.mock("@/lib/query/failover", () => ({
  useProviderHealth: () => ({ data: undefined }),
}));
vi.mock("@/lib/query/queries", () => ({
  useUsageQuery: () => ({ data: undefined }),
}));

/** 带 OAuth 登录快照的官方条目（多账号场景：每个条目存各自的 token 束） */
const snapshotProvider = (id: string, name: string): Provider => ({
  id,
  name,
  category: "official",
  settingsConfig: {
    auth: {
      OPENAI_API_KEY: null,
      auth_mode: "chatgpt",
      last_refresh: "2026-09-20T00:00:00Z",
      tokens: {
        access_token: `access-${id}`,
        account_id: `acct-${id}`,
        refresh_token: `refresh-${id}`,
        id_token: `id-${id}`,
      },
    },
    config: "",
  },
  meta: {
    usage_script: {
      enabled: true,
      language: "javascript",
      code: "",
      templateType: "official_subscription",
      autoQueryInterval: 5,
    },
  },
});

function renderCard(provider: Provider, isCurrent: boolean) {
  return render(
    <QueryClientProvider client={createTestQueryClient()}>
      <ProviderCard
        provider={provider}
        appId="codex"
        isCurrent={isCurrent}
        presentation={{ chips: [], buttons: [] }}
        onEdit={vi.fn()}
        onDelete={vi.fn()}
        onConfigureUsage={vi.fn()}
        onOpenWebsite={vi.fn()}
        onDuplicate={vi.fn()}
      />
    </QueryClientProvider>,
  );
}

describe("ProviderCard codex 多账号订阅用量", () => {
  it("带 OAuth 快照的官方条目把 providerId 传给订阅用量 footer（非当前也展示）", () => {
    renderCard(snapshotProvider("official-x", "OpenAI Official X"), false);

    expect(subscriptionFooterProps).toHaveBeenCalledWith(
      expect.objectContaining({
        providerId: "official-x",
        isCurrent: false,
        autoQueryInterval: 5,
      }),
    );
  });

  it("快照缺 access_token 的条目不启用按条目查询（回退共享 live 路径）", () => {
    const provider = snapshotProvider("official-cli", "OpenAI Official");
    (
      provider.settingsConfig as { auth: Record<string, unknown> }
    ).auth.tokens = {};
    renderCard(provider, true);

    expect(subscriptionFooterProps).toHaveBeenCalledWith(
      expect.objectContaining({ providerId: undefined }),
    );
  });
});
