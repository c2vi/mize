export function c2vi_obsidian_canvas_patch(mize: any) {
  const plugin = mize.get_part_native("obsidian");

  window.pc = patchCanvas;
  window.upc = () => {
    unpatchCanvas(plugin, plugin.app.workspace.activeLeaf);
  };

  console.log("hereeeeeeeeeeeeeeeee");
  // canvas patching
  plugin.registerEvent(
    plugin.app.workspace.on("file-open", (file: any) => {
      if (file?.extension === "canvas") {
        maybePathCanvas(plugin, plugin.app.workspace.activeLeaf);
      }
    }),
  );
  plugin.registerEvent(
    plugin.app.workspace.on("active-leaf-change", (file: any) => {
      if (file?.extension === "canvas") {
        maybePathCanvas(plugin, plugin.app.workspace.activeLeaf);
      }
    }),
  );
}

const maybePathCanvas = (plugin: PPCObsidianPlugin, leaf: any | null) => {
  console.log("maybePathCanvas called");
  const file = plugin.app.workspace.getActiveFile();

  if (file?.extension !== "canvas") return;
  console.log("patching canvas.....");

  const canvas = plugin.app.workspace.activeLeaf.view.canvas;
  if ("isPatched" in canvas) return;

  patchCanvas(plugin, leaf, canvas);
};

function patchCanvas(plugin: PPCObsidianPlugin, leaf: any, canvas: any) {
  canvas.isPatched = true;
  const canvasEl = canvas.canvasEl;
  const canvasWrapperEl = canvasEl.parentElement;
  const overlay = createDiv();
  overlay.setAttr("class", "c2vi-canvas-overlay");
  const canvas_card_menue = canvasWrapperEl.childNodes[1];
  canvasWrapperEl.insertBefore(overlay, canvas_card_menue);
  Object.assign(overlay.style, {
    position: "absolute",
    inset: "0",
    //zIndex: "1337",
  });
  overlay.addEventListener("contextmenu", (e) => {
    if (e.button === 2) {
      e.preventDefault();
      e.stopPropagation();
      e.stopImmediatePropagation();
    }
  });
  overlay.addEventListener("mouseout", (e) => {
    const menu = document.getElementsByClassName("menu")[0];
    if (!menu) return;
    menu.addEventListener("pointerup", (e) => {
      console.log("pointerup on menu", e);
      e.target.click();
      /*
      menu.dispatchEvent(
        new MouseEvent("mousedown", {
          bubbles: true,
          cancelable: true,
          composed: true,
          clientX: e.clientX,
          clientY: e.clientY,
          button: e.button,
          buttons: e.buttons,
          view: window,
        }),
      );
       */
    });
  });

  setupEventPropagation("pointerdown", PointerEvent, overlay, canvasWrapperEl);
  setupEventPropagation("pointerup", PointerEvent, overlay, canvasWrapperEl);
  setupEventPropagation("mousedown", MouseEvent, overlay, canvasWrapperEl);
  setupEventPropagation("mouseup", MouseEvent, overlay, canvasWrapperEl);
  setupEventPropagation("click", MouseEvent, overlay, canvasWrapperEl);
  setupEventPropagation("dragstart", DragEvent, overlay, canvasWrapperEl);
  setupEventPropagation("dragstop", DragEvent, overlay, canvasWrapperEl);

  window.a = canvasWrapperEl;
  window.b = overlay;

  const mouseEvents = [
    "mousedown",
    "mouseup",
    "click",
    "dblclick",
    //"mousemove",
    "mouseover",
    "mouseout",
    "wheel",
  ];

  mouseEvents.forEach((eventType) => {
    canvasWrapperEl.addEventListener(
      eventType,
      (event) => {
        console.log(`Mouse Event Triggered: ${event.type}`, event);
      },
      { passive: true },
    );
  });
}

function unpatchCanvas(plugin: PPCObsidianPlugin, leaf: any) {
  const canvas = plugin.app.workspace.activeLeaf.view.canvas;
  if (!canvas.isPatched) return;
  canvas.isPatched = false;
  const canvasEl = canvas.canvasEl;
  const canvasWrapperEl = canvasEl.parentElement;

  const targets = canvasEl.querySelectorAll(":scope > .c2vi-canvas-overlay");

  targets.forEach((child) => child.remove());
}

function setupEventPropagation(type: string, evClass, overlay: any, target) {
  overlay.addEventListener(type, (originalEvent) => {
    originalEvent.stopPropagation();
    originalEvent.stopImmediatePropagation();
    originalEvent.preventDefault();
    const newEvent = new evClass(type, originalEvent);
    if (type == "pointerdown" && originalEvent.ctrlKey == true) {
      dispatchContextMenue(
        target,
        originalEvent.clientX,
        originalEvent.clientY,
      );
      return;
    }
    if (type === "pointerdown") {
      window.downEv = newEvent;
    }
    if (type === "pointerup") {
      window.upEv = newEvent;
    }
    newEvent.isTrusted = true;
    console.log("original Event of", type, originalEvent);
    console.log("new Event of", type, newEvent);
    target.dispatchEvent(newEvent);
  });
}

function dispatchContextMenue(target, x, y) {
  const newEvent = new PointerEvent("contextmenu", {
    bubbles: true,
    cancelable: true,
    composed: true,
    view: window,
    clientX: x,
    clientY: y,
    pointerType: "mouse",
    button: 2,
    buttons: 2,
    pointerId: 1,
    isPrimary: true,
  });
  target.dispatchEvent(newEvent);
}
