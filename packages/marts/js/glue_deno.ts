import { Mize } from "./stub_deno.ts";

function argsToMessage(...args) {
  return args.map((arg) => JSON.stringify(arg)).join(" ");
}

globalThis.log = (...args) => {
  Deno.core.print(`[out]: ${argsToMessage(...args)}\n`, false);
};

globalThis.mize = new Mize();
