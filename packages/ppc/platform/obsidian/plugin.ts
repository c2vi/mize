import { Plugin, TFile } from "obsidian";
import wasmBinary from "./rust_dist/ppc_bg.wasm";
import * as wasmNamespace from "./rust_dist/ppc";
import { c2vi_obsidian_canvas_patch } from "@ppc/marts";
import { task_obsidian_otask_parse } from "@ppc/marts";

export default class PPCObsidianPlugin extends Plugin {
  async onload() {
    // init mize from wasm
    const wasmModule = await import("./rust_dist/ppc.js");
    await wasmModule.default({ module_or_path: wasmBinary });
    wasmModule.initSync();
    wasmModule.obsidian_mize_entrypoint();

    const mize = {};
    mize.get_part_native = (name) => this;

    // c2vi part which patches canvas movement
    c2vi_obsidian_canvas_patch(mize);

    task_obsidian_otask_parse(mize);

    this.addCommand({
      id: "app",
      name: "Open ppc app main page",
      callback: async () => {
        const leaf = this.app.workspace.getLeaf(true);
        await leaf.setViewState({
          type: "ppc-app",
          active: true,
        });
        this.app.workspace.revealLeaf(leaf);
      },
    });
  }

  onunload() {
    this.app.workspace.detachLeavesOfType("ppc-app");
  }
}
