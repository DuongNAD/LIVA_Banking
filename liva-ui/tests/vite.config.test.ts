import { describe, expect, it } from "vitest";
import { vendorChunkName } from "../vite.config";

describe("vendorChunkName", () => {
  it("routes vue dependencies to vendor-vue chunk", () => {
    expect(vendorChunkName("C:/repo/node_modules/vue/dist/vue.esm-bundler.js")).toBe(
      "vendor-vue"
    );
    expect(vendorChunkName("C:/repo/node_modules/@vue/runtime-core/index.js")).toBe(
      "vendor-vue"
    );
  });

  it("routes other dependencies to vendor chunk", () => {
    expect(vendorChunkName("C:/repo/node_modules/pinia/dist/pinia.mjs")).toBe("vendor");
    expect(vendorChunkName("C:/repo/node_modules/msgpackr/pack.js")).toBe("vendor");
  });

  it("leaves application source unbundled into vendor chunks", () => {
    expect(vendorChunkName("C:/repo/src/BankingApp.vue")).toBeUndefined();
    expect(vendorChunkName("C:/repo/src/components/banking/BankCard.vue")).toBeUndefined();
  });
});
