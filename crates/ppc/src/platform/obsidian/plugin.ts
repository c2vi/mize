import { Plugin, TFile, Vault, MetadataCache } from "obsidian";
import { id, i, init, InstaQLEntity } from "@instantdb/react";
import remarkParse from "remark-parse";
import remarkStringify from "remark-stringify";
import { unified } from "unified";
import { visit } from "unist-util-visit";
import { Root, Content, Heading, List, ListItem, Text } from "mdast";
import remarkFrontmatter from "remark-frontmatter";
import remarkGfm from "remark-gfm";
import wikiLinkPlugin from "@flowershow/remark-wiki-link";
import wasmBinary from "./rust_dist/ppc_bg.wasm";
import * as wasmNamespace from "./rust_dist/ppc";

const OTASK_PROP_NAMES = ["short", "outcome", "priority"];
const ACTION_PROP_NAMES = [];

const REMARK = unified()
  .use(remarkParse)
  .use(remarkFrontmatter)
  .use(remarkGfm)
  .use(remarkStringify, { bullet: "-" });

export default class PPCObsidianPlugin extends Plugin {
  async onload() {
    // init mize from wasm
    const wasmModule = await import("./rust_dist/ppc.js");
    await wasmModule.default({ module_or_path: wasmBinary });
    console.log("hoooo wasmModule:", wasmModule);
    wasmModule.initSync();

    //console.log("");
    //const buffer = Uint8Array.from(atob(rustPlugin), (c) => c.charCodeAt(0));
    wasmModule.obsidian_mize_entrypoint();

    // Listen for file modifications in the vault
    //this.registerEvent(
    //this.app.vault.on("modify", (file: TFile) => {
    //this.handleFileUpdate(file);
    //})
    //);

    //this.registerView(
    //"system-c2-app",
    //(leaf) => new AppReactView(leaf, this)
    //);

    this.addCommand({
      id: "app",
      name: "Open SystemC2 App page",
      callback: async () => {
        const leaf = this.app.workspace.getLeaf(true);
        await leaf.setViewState({
          type: "system-c2-app",
          active: true,
        });
        this.app.workspace.revealLeaf(leaf);
      },
    });

    this.addCommand({
      id: "full-parse",
      name: "Parse all otask files into the db",
      callback: async () => {
        await full_parse();
      },
    });

    //full_parse(this.db)
  }

  async handleFileUpdate(file: TFile) {
    if (!(file instanceof TFile) || file.extension !== "md") {
      return; // only care about markdown files
    }

    const metadata = this.app.metadataCache.getFileCache(file);

    // Example: check if the file has a specific tag
    if (metadata?.frontmatter?.tags?.includes("t/otask")) {
      this.parseOtask(file);
    }
  }

  async parseOtask(file: TFile) {
    if (!this.isLoggedIn()) {
      return;
    }

    const isRrootOtask = file.basename === "my-projects";

    if (isRrootOtask) {
      return;
    }
  }

  async isLoggedIn() {
    const user = this.db.getAuth();
    user;
  }

  onunload() {
    this.app.workspace.detachLeavesOfType("system-c2-app");
  }
}

let OTASK_PARSE_QUEUE = [];

let OTASK_LIST = [];

let DBG = false;

function dbg() {
  if (DBG) {
    console.log(arguments);
  }
}

function queue_otasks_from_list_item(listItem, otask_path, priority = 0) {
  OTASK_PARSE_QUEUE.push({
    listItem,
    otask_path,
    priority,
  });
}

function queue_otasks_from_file(file, otask_path, priority = 0) {
  OTASK_PARSE_QUEUE.push({
    file,
    otask_path,
    priority,
  });
}

// parsing helper functions

async function full_parse() {
  console.log("full parse.......................... yay");
  let otasks = [];
  OTASK_PARSE_QUEUE = [];
  OTASK_LIST = [];

  const root_file = app.vault.getAbstractFileByPath("task/my-projects.md");

  const content = await app.vault.read(root_file);
  const list = await getListUnderHeading(content, "root");

  for (const entry of list) {
    queue_otasks_from_list_item(entry, []);
  }

  // go through the queue of otaskParseTasks
  // using a queue instead of recursive calls, to make it more debuggable...
  do {
    const task = OTASK_PARSE_QUEUE.shift();
    dbg("task", task);
    if (task.file) {
      const otask = await otasks_from_file(
        task.file,
        task.otask_path,
        task.priority,
      );
      dbg("otask", otask);
      if (otask) {
        OTASK_LIST.push(otask);
      }
    } else {
      const otask = await otasks_from_list_item(
        task.listItem,
        task.otask_path,
        task.priority,
      );
      dbg("otask", otask);
      if (otask) {
        OTASK_LIST.push(otask);
      }
    }
  } while (OTASK_PARSE_QUEUE.length > 0);

  console.log("otasks::", OTASK_LIST);

  render_sub0_file(OTASK_LIST);
}

async function render_sub0_file(otasks) {
  const sub0_file = app.vault.getAbstractFileByPath("private/sub0-tasks.md");
  if (!(sub0_file instanceof TFile)) {
    console.error(
      "SystemC2: sub0_file not found:",
      "private/sub0-tasks.md",
      sub0_file,
    );
    return;
  }

  //render sub0
  const sub0_otasks = otasks.filter(
    (otask) => otask.priority == 0 && !otask.is_file,
  );
  console.log("sub0 otasks...", sub0_otasks);

  let rendered_otasks_string = "";

  for (const otask of sub0_otasks) {
    const path_part = otask.path.length > 0 ? otask.path.join(": ") + ": " : "";

    let content_part = otask.content ? otask.content + "\n" : "";
    content_part = content_part
      .split("\n")
      .map((line) => "\t" + line)
      .join("\n"); // add a \t in front of every otask.content line

    rendered_otasks_string += `- ${path_part}${otask.longName}\n${content_part}\n`;
  }
  //console.log("newContent...", rendered_otasks_string)

  await replaceHeading(sub0_file, "sub0", rendered_otasks_string);

  //render sub-1
}

async function replaceHeading(
  file: TFile,
  heading: string,
  newContent: string,
) {
  const data = await this.app.vault.read(file);

  // Parse the Markdown into an AST
  const tree = REMARK.parse(data) as Root;

  // Split new content into AST nodes
  const newContentAst = REMARK.parse(newContent) as Root;

  const children = tree.children;
  const newChildren: Content[] = [];
  let insideTarget = false;
  let targetLevel = 0;

  for (let i = 0; i < children.length; i++) {
    const node = children[i];

    if (node.type === "heading") {
      const headingNode = node as Heading;
      const text = headingNode.children
        .filter((c) => c.type === "text")
        .map((c: any) => c.value)
        .join("");

      // If we encounter our target heading
      if (text.trim() === heading) {
        insideTarget = true;
        targetLevel = headingNode.depth;
        newChildren.push(node); // Keep the heading itself
        // Insert new content right after
        newChildren.push(...newContentAst.children);
        continue;
      }

      // If we’re currently inside target section and see a heading of same or higher level
      if (insideTarget && headingNode.depth <= targetLevel) {
        insideTarget = false;
      }
    }

    // If we’re not inside the target section, keep the node
    if (!insideTarget) {
      newChildren.push(node);
    }
  }

  const newTree: Root = { ...tree, children: newChildren };

  // Convert AST back to Markdown
  const newMarkdown = await REMARK.stringify(newTree);

  await this.app.vault.modify(file, newMarkdown);
}

async function otasks_from_list_item(listItem, otask_path, priority = 0) {
  // the main otask this listItem is about
  let otask = {
    description: [],
    actions: [],
    path: [],
    priority,
  };
  dbg("SystemC2: otasks_from_list_item", listItem, otask_path);

  if (!listItem.children[0].type === "paragraph") {
    dbg(
      "listItem[0] of a otask item is not a paragraph... the otask:",
      listItem,
    );
    return null;
  }
  let longName = nodeToString(listItem.children[0]);

  dbg("longName:", longName);

  // check for [[anotherotask]], call otasks_from_file, return those
  const match = longName.match(/\[\[([^\]]+)\]\]/);
  if (match) {
    const file = app.metadataCache.getFirstLinkpathDest(match[1], "");
    if (file) {
      const metadata = app.metadataCache.getFileCache(file);

      if (metadata?.frontmatter?.tags?.includes("t/otask")) {
        queue_otasks_from_file(file, otask_path);
        return null;
      } else {
        dbg(
          "SystemC2: listItem with Link to '",
          match[1],
          "', which is not an otask",
          longName,
        );
      }
    }
  }

  //check for [x] and remove from name
  if (longName.startsWith("[ ") || longName.startsWith("[x")) {
    longName = longName.slice(4);
  }

  //check for "ot:" and remove from name
  if (longName.startsWith("ot:")) {
    longName = longName.slice(4);
  }

  otask.longName = longName;
  otask.short = longName;
  otask.path = otask_path;
  let new_otask_path = [...otask_path, otask.short];

  // go through sub list items
  // 	- if it's a prop for the otask... handle that
  // 	- if it's an "ac:" handle that
  // 	- if it's "ot: " handle as another otask -> recursive call
  // 	- if it's "[ ]" handle as another otask -> recursive call
  // 	- if it's a link to a otask file... handle as another otask -> call otasks_from_file()
  // 	- else add it as description item list to otask
  if (!listItem.children || listItem.children.length < 2) {
    return otask;
  }

  // add content as text to otask
  if (otask.longName == "hosting updates") {
    console.log("listItem... ", listItem.children[1]);
    let hii = REMARK.stringify(listItem.children[1]);
    console.log("text...", hii);
  }
  let content_text = REMARK.stringify(listItem.children[1]);
  otask.content = content_text;

  for (const subListItem of listItem.children[1].children) {
    let text = nodeToString(subListItem.children[0]);

    // check prop
    if (check_otask_prop_from_list_item(subListItem, otask)) {
      new_otask_path = [...otask_path, otask.short]; // have to redo it here, in case short changed from aprop
      continue; // don't do any other processing on this sub-listItem
    }

    // remove "[ ]"
    if (text.startsWith("[ ") || text.startsWith("[x")) {
      text = text.slice(4);
    }

    // check ac
    if (text.startsWith("ac:")) {
      otask.actions = [
        ...otask.actions,
        action_from_list_item(subListItem, new_otask_path, otask.priority),
      ];

      continue; // don't do any other processing on this sub-listItem
    }

    // check ot
    if (text.startsWith("ot:")) {
      queue_otasks_from_list_item(subListItem, new_otask_path, otask.priority);

      continue; // don't do any other processing on this sub-listItem
    }

    // check [ ]
    if (text.startsWith("[ ] ") || text.startsWith("[x] ")) {
      queue_otasks_from_list_item(subListItem, new_otask_path, otask.priority);

      continue; // don't do any other processing on this sub-listItem
    }

    // check link to otask file
    const match = text.match(/\[\[([^\]]+)\]\]/);
    if (match) {
      const file = app.metadataCache.getFirstLinkpathDest(match[1], "");
      if (file) {
        const metadata = app.metadataCache.getFileCache(file);

        if (metadata?.frontmatter?.tags?.includes("t/otask")) {
          queue_otasks_from_file(file, new_otask_path, otask.priority);
          continue;
        } else {
          dbg(
            "SystemC2: listItem with Link to '",
            match[1],
            "', which is not an otask",
            longName,
          );
        }

        continue; // don't do any other processing on this sub-listItem
      }
    }

    // add as description
    const desc = REMARK.stringify(subListItem);
    otask.description.push(desc);
  }

  return otask;
}

async function otasks_from_file(file: TFile, otask_path, priority = 0) {
  dbg("SystemC2: otasks_from_file", file, otask_path);

  // the main otask this file is about
  let otask = {
    description: [],
    actions: [],
    path: [],
    priority,
    is_file: true,
  };

  otask.longName = file.basename;
  otask.short = otask.longName;
  otask.path = otask_path;
  let new_otask_path = [...otask_path, otask.short];

  const content = await app.vault.read(file);

  // parse listItems at the top... for props
  const topListItems = getListItemsBeforeFirstHeading(content);
  for (const topListItem of topListItems) {
    const text = nodeToString(topListItem);

    // check for prop
    if (check_otask_prop_from_list_item(topListItem, otask)) {
      new_otask_path = [...otask_path, otask.short];
      continue; // don't do any other processing on this sub-listItem
    }
  }

  // parse all the sub sections
  const SUB_COUNT = 10;
  for (let sub_number = 0; sub_number < SUB_COUNT; sub_number++) {
    let listItems = await getListUnderHeading(content, "sub" + sub_number);

    // "sub" without a number should have priority 5
    if (sub_number == 5) {
      listItems = listItems.concat(await getListUnderHeading(content, "sub"));
    }

    //dbg(`sub section ${sub_number} has items:`, listItems)

    for (const listItem of listItems) {
      // check if listItem is an action...
      let text = nodeToString(listItem.children[0]);
      if (
        text.startsWith("ac:") ||
        text.startsWith("[ ] ac:") ||
        text.startsWith("[x] ac:")
      ) {
        const action = action_from_list_item(listItem, otask_path);
        otask.actions.push(action);
      }

      queue_otasks_from_list_item(
        listItem,
        new_otask_path,
        otask.priority + sub_number,
      );
    }
  }

  return otask;
}

function action_from_list_item(listItem, otask_path) {
  dbg("SystemC2: action_from_list_item", listItem, otask_path);
  let action = {};
  let text = nodeToString(listItem.children[0]).slice(4);

  if (text.startsWith("[")) {
    text = text.slice(4);
  }

  action.name = text;

  return action;
}

function check_otask_prop_from_list_item(listItem, otask) {
  let text = nodeToString(listItem.children[0]);

  for (const prop_name of OTASK_PROP_NAMES) {
    if (text.startsWith(prop_name + ":")) {
      const value = text.slice(prop_name.length + 2);

      if (prop_name == "priority") {
        // parse priority as int
        otask[prop_name] = parseInt(value);
      } else if (prop_name == "outcome") {
        // add sub items to it
        let fullText = REMARK.stringify(listItem).slice(prop_name.length + 4);
        otask[prop_name] = fullText;
      } else {
        otask[prop_name] = value;
      }

      return true;
    }
  }
  return false;
}

// Extract plain text from a node
function nodeToString(node: any): string {
  let result = "";
  visit(node, "text", (textNode: Text) => {
    result += textNode.value;
  });
  return result.trim();
}

async function getListUnderHeading(
  markdown: string,
  headingText: string,
): string[] {
  const tree = REMARK.parse(markdown);
  //dbg("tree", tree)

  let capture = false;
  let items: string[] = [];

  for (let i = 0; i < tree.children.length; i++) {
    const node: Content = tree.children[i];

    // Detect the target heading
    if (
      node.type === "heading" &&
      nodeToString(node).toLowerCase() === headingText.toLowerCase()
    ) {
      capture = true;
      continue;
    }

    // Stop if another heading of the same or higher level comes
    if (capture && node.type === "heading") {
      break;
    }

    // Collect list items if capturing
    if (capture && node.type === "list") {
      for (const li of (node as List).children) {
        if (li.type === "listItem") {
          items.push(li);
        }
      }
    }
  }

  return items;
}

function getListItemsBeforeFirstHeading(markdown: string) {
  const tree = REMARK.parse(markdown);

  let items = [];

  for (const node of tree.children) {
    if (node.type === "heading") {
      // stop at first heading
      break;
    }
    if (node.type === "list") {
      for (const li of (node as List).children) {
        if (li.type === "listItem") {
          items.push(li);
        }
      }
    }
  }

  return items;
}
