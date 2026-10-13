import { describe, expect, it } from "vitest";
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
