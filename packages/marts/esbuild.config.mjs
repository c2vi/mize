import * as esbuild from "npm:esbuild";

import { denoPlugin } from "jsr:@deno/esbuild-plugin";

await esbuild.build({
  plugins: [denoPlugin()],
  entryPoints: ["./habitica.ts"],
  outfile: "./deno_dist/habitica.js",
  bundle: true,
  format: "iife",
});
esbuild.stop();
