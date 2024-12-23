

async function init_mize(config) {

	if ("mize" in window) {
		window.mize.set("self/config", config)
		return window.mize
	}


   const { JsInstance } = wasm_bindgen;

   await wasm_bindgen()

   window.mize = await new JsInstance(config)

	return window.mize
}
