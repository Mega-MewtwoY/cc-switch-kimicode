import { describe, expect, it } from "vitest";
import { codexProviderPresets } from "./codexProviderPresets";

// Dedicated GLM presets are intentionally absent from this fork.
describe.each(["Zhipu GLM", "Zhipu GLM en"])("%s retired preset", (name) => {
  it("is not restored by upstream preset updates", () => {
    expect(
      codexProviderPresets.find((preset) => preset.name === name),
    ).toBeUndefined();
  });
});
