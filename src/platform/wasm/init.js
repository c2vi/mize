

async function init_mize(config) {

	if ("mize" in window) {
		window.mize.set("self/config", config)
		return window.mize
	}

  let config_str = "";
  if typeof config == "string" {
    config_str = config

  } else if typeof config == "object" {
    config_str = JSON.stringify(config)

  } else {
    throw "Error: config passed to init_mize() is not of type 'object' or 'string'"
  }

  const { new_js_instance } = wasm_bindgen;

  await wasm_bindgen()

  window.mize = await new_js_instance(config)
  window.mize.mod = {}
  window.mize.init()

  return window.mize
}
