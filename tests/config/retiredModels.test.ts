import { describe, expect, it } from "vitest";
import { parse } from "smol-toml";
import { APP_IDS, MCP_APP_IDS, SKILLS_APP_IDS } from "@/config/appConfig";
import { providerPresets as claudeProviderPresets } from "@/config/claudeProviderPresets";
import { claudeDesktopProviderPresets } from "@/config/claudeDesktopProviderPresets";
import { codexProviderPresets } from "@/config/codexProviderPresets";
import { openclawProviderPresets } from "@/config/openclawProviderPresets";
import { opencodeProviderPresets } from "@/config/opencodeProviderPresets";
import { hermesProviderPresets } from "@/config/hermesProviderPresets";
import { piProviderPresets } from "@/config/piProviderPresets";

import { KNOWN_APP_TYPES } from "@/types/usage";

describe("retired integrations", () => {
  it("keeps historical ZCode usage filterable", () => {
    expect(KNOWN_APP_TYPES).toContain("zcode");
  });
  it("does not expose ZCode as a configurable application", () => {
    for (const ids of [APP_IDS, MCP_APP_IDS, SKILLS_APP_IDS]) {
      expect(ids).not.toContain("zcode");
    }
  });
  it.each(
    Object.entries({
      claudeProviderPresets,
      claudeDesktopProviderPresets,
      codexProviderPresets,
      openclawProviderPresets,
      opencodeProviderPresets,
      hermesProviderPresets,
      piProviderPresets,
    }),
  )("%s contains no GLM model or dedicated provider", (_name, presets) => {
    expect(JSON.stringify(presets)).not.toMatch(/glm|zcode|zhipu/i);
  });
});

// Removing a retired model must leave the preset's default selectable.
describe("replacement cloud preset defaults", () => {
  it.each([
    "Baseten",
    "Nebius Token Factory",
    "Hugging Face",
    "Ollama Cloud",
    "DigitalOcean",
    "Fireworks AI",
  ])("keeps %s defaults in the available model catalogs", (name) => {
    const codex = codexProviderPresets.find((preset) => preset.name === name)!;
    const defaultModel = parse(codex.config).model;
    expect(codex.modelCatalog?.map((model) => model.model)).toContain(
      defaultModel,
    );

    const hermes = hermesProviderPresets.find(
      (preset) => preset.name === name,
    )!;
    expect(hermes.settingsConfig.models?.map((model) => model.id)).toContain(
      hermes.suggestedDefaults?.model?.default,
    );

    const openclaw = openclawProviderPresets.find(
      (preset) => preset.name === name,
    )!;
    const primary = openclaw.suggestedDefaults!.model!.primary!;
    expect(openclaw.settingsConfig.models?.map((model) => model.id)).toContain(
      primary.slice(primary.indexOf("/") + 1),
    );
    expect(openclaw.suggestedDefaults?.modelCatalog).toHaveProperty(primary);
  });

  it.each(["Baseten", "Nebius Token Factory", "Hugging Face"])(
    "keeps %s Claude model routes on the retained provider namespace",
    (name) => {
      const claude = claudeProviderPresets.find(
        (preset) => preset.name === name,
      )!;
      const env = (claude.settingsConfig as { env: Record<string, string> })
        .env;
      for (const [key, model] of Object.entries(env)) {
        if (
          key === "ANTHROPIC_MODEL" ||
          /^ANTHROPIC_DEFAULT_.*_MODEL$/.test(key)
        ) {
          expect(model).toMatch(/^(moonshotai|deepseek-ai)\/\S+$/);
        }
      }
      const desktop = claudeDesktopProviderPresets.find(
        (preset) => preset.name === name,
      )!;
      for (const route of desktop.modelRoutes ?? []) {
        expect(route.upstreamModel).toMatch(/^(moonshotai|deepseek-ai)\/\S+$/);
      }
    },
  );
});
