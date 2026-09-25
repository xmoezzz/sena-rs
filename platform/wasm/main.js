import init, { start_sena_from_directory } from "./pkg/pal_vm.js";

const input = document.getElementById("game-dir");
const nlsSelect = document.getElementById("nls-select");
const rescanButton = document.getElementById("rescan-button");
const statusLine = document.getElementById("status-line");
const errorBox = document.getElementById("error-box");
const grid = document.getElementById("game-grid");
const emptyCard = document.getElementById("empty-card");
const launchOverlay = document.getElementById("launch-overlay");
const launchText = document.getElementById("launch-text");
const libraryScreen = document.getElementById("library-screen");
const playerScreen = document.getElementById("player-screen");
const playerTitle = document.getElementById("player-title");
const exitButton = document.getElementById("exit-button");
const canvas = document.getElementById("sena-canvas");

let wasmInitialized = false;
let selectedFileList = null;
let selectedRootName = "";
let games = [];
let running = false;

function selectedNls() {
  return nlsSelect && nlsSelect.value ? nlsSelect.value : "sjis";
}

function nlsLabel(value) {
  switch (String(value || "").toLowerCase()) {
    case "gbk": return "GBK";
    case "utf-8":
    case "utf8": return "UTF-8";
    default: return "ShiftJIS";
  }
}

const filesByPath = new Map();
const filesByLowerPath = new Map();
const dirChildren = new Map();
let filesMetadata = [];

function setStatus(text) {
  statusLine.textContent = text;
}

function setError(error) {
  const text = error && error.stack ? error.stack : String(error);
  console.error(error);
  errorBox.textContent = text;
  errorBox.style.display = "block";
}

function clearError() {
  errorBox.textContent = "";
  errorBox.style.display = "none";
}

function showLaunching(text) {
  launchText.textContent = text;
  launchOverlay.style.display = "grid";
}

function hideLaunching() {
  launchOverlay.style.display = "none";
}

function normalizePath(path) {
  return String(path || "")
    .replaceAll("\\\\", "/")
    .replaceAll("\\", "/")
    .split("/")
    .filter((part) => part.length > 0 && part !== ".")
    .join("/");
}

function splitPath(path) {
  const normalized = normalizePath(path);
  return normalized ? normalized.split("/").filter(Boolean) : [];
}

function hashString(s) {
  let h = 2166136261;
  for (let i = 0; i < s.length; i += 1) {
    h ^= s.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return (h >>> 0).toString(16);
}

function rootNameFromFileList(fileList) {
  for (const file of fileList) {
    const rel = normalizePath(file.webkitRelativePath || file.name);
    const parts = splitPath(rel);
    if (parts.length > 0) return parts[0];
  }
  return "Selected Folder";
}

function relativeInsideSelectedRoot(file) {
  const rel = normalizePath(file.webkitRelativePath || file.name);
  const parts = splitPath(rel);
  if (parts.length <= 1) return file.name || rel;
  return parts.slice(1).join("/");
}

function hasRootFile(entries, fileName) {
  const needle = fileName.toLowerCase();
  return entries.some((entry) => {
    const parts = splitPath(entry.gamePath.toLowerCase());
    return parts.length === 1 && parts[0] === needle;
  });
}

function looksLikeGameRoot(entries) {
  return hasRootFile(entries, "Scene.pck") ||
    hasRootFile(entries, "Gameexe.ini") ||
    hasRootFile(entries, "Gameexe.dat") ||
    hasRootFile(entries, "data.pac") ||
    hasRootFile(entries, "SYSTEM.INI") ||
    hasRootFile(entries, "archive.dat");
}

function titleForGameRoot(fallback, entries) {
  if (hasRootFile(entries, "Gameexe.ini")) return fallback;
  if (hasRootFile(entries, "Gameexe.dat")) return fallback;
  if (hasRootFile(entries, "Scene.pck")) return fallback;
  return fallback;
}

function rememberLastPlayed(id) {
  try {
    localStorage.setItem(`sena.lastPlayed.${id}`, String(Date.now()));
  } catch (_) {
    // best-effort
  }
}

function getLastPlayed(id) {
  try {
    return Number(localStorage.getItem(`sena.lastPlayed.${id}`) || "0") || 0;
  } catch (_) {
    return 0;
  }
}

async function buildGamesFromFileList(fileList) {
  const rootName = rootNameFromFileList(fileList);
  selectedRootName = rootName;

  const rootEntries = [];
  for (const file of fileList) {
    const rootRelativePath = relativeInsideSelectedRoot(file);
    if (!rootRelativePath) continue;
    rootEntries.push({ file, gamePath: rootRelativePath });
  }

  if (looksLikeGameRoot(rootEntries)) {
    const id = hashString(`${rootName}:/`);
    const title = titleForGameRoot(rootName, rootEntries);
    return [{
      id,
      title,
      rootPath: rootName,
      entries: rootEntries,
      lastPlayed: getLastPlayed(id),
      nls: selectedNls(),
    }];
  }

  const groups = new Map();
  for (const entry of rootEntries) {
    const parts = splitPath(entry.gamePath);
    if (parts.length < 2) continue;
    const groupName = parts[0];
    const gamePath = parts.slice(1).join("/");
    if (!gamePath) continue;

    if (!groups.has(groupName)) groups.set(groupName, []);
    groups.get(groupName).push({ file: entry.file, gamePath });
  }

  const out = [];
  for (const [groupName, entries] of groups.entries()) {
    if (!looksLikeGameRoot(entries)) continue;

    const id = hashString(`${rootName}:${groupName}`);
    const title = titleForGameRoot(groupName, entries);

    out.push({
      id,
      title,
      rootPath: `${rootName}/${groupName}`,
      entries,
      lastPlayed: getLastPlayed(id),
      nls: selectedNls(),
    });
  }

  out.sort((a, b) => {
    if (a.lastPlayed !== b.lastPlayed) return b.lastPlayed - a.lastPlayed;
    return a.title.localeCompare(b.title);
  });

  return out;
}

function renderLibrary() {
  grid.innerHTML = "";
  emptyCard.style.display = games.length === 0 ? "block" : "none";

  for (const game of games) {
    const tile = document.createElement("article");
    tile.className = "game-tile";

    const poster = document.createElement("div");
    poster.className = "poster";

    const posterTitle = document.createElement("div");
    posterTitle.className = "poster-title";
    posterTitle.textContent = game.title;
    poster.append(posterTitle);

    const title = document.createElement("div");
    title.className = "game-title";
    title.textContent = game.title;

    const path = document.createElement("div");
    path.className = "game-path";
    path.textContent = game.rootPath;

    const meta = document.createElement("div");
    meta.className = "game-meta";
    meta.textContent = `${game.entries.length} file(s) · ${nlsLabel(game.nls)}`;

    const actions = document.createElement("div");
    actions.className = "tile-actions";

    const play = document.createElement("button");
    play.textContent = "Play";
    play.addEventListener("click", () => launchGame(game));

    const grow = document.createElement("div");
    grow.className = "grow";

    const nls = document.createElement("select");
    nls.className = "tile-nls";
    for (const [value, label] of [["sjis", "ShiftJIS"], ["gbk", "GBK"], ["utf-8", "UTF-8"]]) {
      const option = document.createElement("option");
      option.value = value;
      option.textContent = label;
      if (value === game.nls) option.selected = true;
      nls.append(option);
    }
    nls.addEventListener("change", () => {
      game.nls = selectedNlsValue(nls.value);
      renderLibrary();
    });

    const remove = document.createElement("button");
    remove.className = "danger";
    remove.textContent = "Remove";
    remove.addEventListener("click", () => {
      games = games.filter((x) => x.id !== game.id);
      renderLibrary();
      setStatus(games.length === 0 ? "No games in library." : `${games.length} game(s) in library.`);
    });

    actions.append(play, grow, nls, remove);
    tile.append(poster, title, path, meta, actions);
    grid.append(tile);
  }
}

function selectedNlsValue(value) {
  const normalized = String(value || "").toLowerCase();
  return normalized === "gbk" || normalized === "utf-8" || normalized === "utf8" ? (normalized === "utf8" ? "utf-8" : normalized) : "sjis";
}

async function scanCurrentSelection() {
  clearError();

  if (!selectedFileList || selectedFileList.length === 0) {
    games = [];
    renderLibrary();
    setStatus("No folder selected.");
    rescanButton.disabled = true;
    return;
  }

  setStatus(`Scanning ${selectedFileList.length} file(s)...`);
  await new Promise((resolve) => setTimeout(resolve, 0));

  games = await buildGamesFromFileList(selectedFileList);
  renderLibrary();
  rescanButton.disabled = false;

  if (games.length === 0) {
    setStatus(`No valid Sena game root found under ${selectedRootName}.`);
  } else {
    setStatus(`${games.length} game(s) found under ${selectedRootName}.`);
  }
}

function parentDirOf(path) {
  const i = path.lastIndexOf("/");
  return i < 0 ? "" : path.slice(0, i);
}

function baseNameOf(path) {
  const i = path.lastIndexOf("/");
  return i < 0 ? path : path.slice(i + 1);
}

function registerDir(path) {
  const parts = splitPath(path);
  let parent = "";

  for (let i = 0; i + 1 < parts.length; i += 1) {
    const child = parts[i];
    if (!dirChildren.has(parent)) {
      dirChildren.set(parent, new Set());
    }
    dirChildren.get(parent).add(child);
    parent = parent ? `${parent}/${child}` : child;
  }

  const fileParent = parentDirOf(path);
  const fileBase = baseNameOf(path);
  if (!dirChildren.has(fileParent)) {
    dirChildren.set(fileParent, new Set());
  }
  dirChildren.get(fileParent).add(fileBase);
}

function registerGameEntries(game) {
  filesByPath.clear();
  filesByLowerPath.clear();
  dirChildren.clear();
  filesMetadata = [];

  for (const entry of game.entries) {
    const path = normalizePath(entry.gamePath);
    if (!path) continue;

    const lower = path.toLowerCase();
    if (filesByLowerPath.has(lower)) {
      const other = filesByLowerPath.get(lower).__senaPath;
      throw new Error(`case-insensitive path conflict: ${other} vs ${path}`);
    }

    entry.file.__senaPath = path;
    filesByPath.set(path, entry.file);
    filesByLowerPath.set(lower, entry.file);
    registerDir(path);
    filesMetadata.push({ path, size: entry.file.size, lastModified: entry.file.lastModified || 0 });
  }

  console.log("sena wasm registered files:", filesMetadata.length);
  console.log("sena wasm file sample:", filesMetadata.slice(0, 50).map((f) => `${f.path} (${f.size})`));
  console.log("sena wasm has Scene.pck:", globalThis.senaFileExists("Scene.pck"));
  console.log("sena wasm has Gameexe.ini:", globalThis.senaFileExists("Gameexe.ini"));
  console.log("sena wasm has data.pac:", globalThis.senaFileExists("data.pac"));
  console.log("sena wasm has SYSTEM.INI:", globalThis.senaFileExists("SYSTEM.INI"));

  const palRoot = globalThis.senaFileExists("data.pac") ||
    globalThis.senaFileExists("SYSTEM.INI") ||
    globalThis.senaFileExists("archive.dat") ||
    globalThis.senaFileExists("data/archive.dat");
  const legacyRoot = globalThis.senaFileExists("Scene.pck") ||
    globalThis.senaFileExists("Gameexe.ini") ||
    globalThis.senaFileExists("Gameexe.dat");
  if (!palRoot && !legacyRoot) {
    throw new Error("selected directory does not look like a PAL or legacy Sena game root");
  }

  return filesMetadata;
}

function resolveSenaFile(path) {
  const normalized = normalizePath(path);

  let file = filesByPath.get(normalized);
  if (file) return file;

  file = filesByLowerPath.get(normalized.toLowerCase());
  return file || null;
}

function readFileSynchronously(file) {
  const url = URL.createObjectURL(file);
  try {
    const xhr = new XMLHttpRequest();
    xhr.open("GET", url, false);
    xhr.overrideMimeType("text/plain; charset=x-user-defined");
    xhr.send(null);

    if (xhr.status !== 200 && xhr.status !== 0) {
      throw new Error(`Sena wasm file read failed: HTTP ${xhr.status}`);
    }

    const text = xhr.responseText || "";
    const out = new Uint8Array(text.length);
    for (let i = 0; i < text.length; i += 1) {
      out[i] = text.charCodeAt(i) & 0xff;
    }
    return out;
  } finally {
    URL.revokeObjectURL(url);
  }
}

globalThis.senaFileExists = function senaFileExists(path) {
  return resolveSenaFile(path) !== null;
};

globalThis.senaFileSize = function senaFileSize(path) {
  const file = resolveSenaFile(path);
  return file ? file.size : -1;
};

globalThis.senaReadFile = function senaReadFile(path) {
  const file = resolveSenaFile(path);
  if (!file) {
    throw new Error(`Sena file not found: ${path}`);
  }
  return readFileSynchronously(file);
};

globalThis.senaReadRange = function senaReadRange(path, offset, len) {
  const file = resolveSenaFile(path);
  if (!file) {
    throw new Error(`Sena file not found: ${path}`);
  }
  const start = Math.max(0, Number(offset) || 0);
  const end = Math.min(file.size, start + Math.max(0, Number(len) || 0));
  return readFileSynchronously(file.slice(start, end));
};

globalThis.senaListDir = function senaListDir(path) {
  const normalized = normalizePath(path).replace(/\/$/, "");
  const children = dirChildren.get(normalized);
  return children ? Array.from(children) : [];
};

globalThis.senaKnownFileCount = function senaKnownFileCount() {
  return filesByPath.size;
};

async function ensureWasmInitialized() {
  if (wasmInitialized) return;
  showLaunching("Loading wasm…");
  await init();
  wasmInitialized = true;
}

async function launchGame(game) {
  if (running) return;

  try {
    clearError();
    running = true;
    showLaunching("Registering files…");

    await ensureWasmInitialized();
    const files = registerGameEntries(game);

    rememberLastPlayed(game.id);
    game.lastPlayed = Date.now();

    libraryScreen.style.display = "none";
    playerScreen.style.display = "block";
    playerTitle.textContent = game.title;
    canvas.focus();

    showLaunching("Launching…");
    await start_sena_from_directory("sena-canvas", JSON.stringify(files), selectedNlsValue(game.nls));

    hideLaunching();
    setStatus("Running.");
  } catch (error) {
    running = false;
    playerScreen.style.display = "none";
    libraryScreen.style.display = "flex";
    hideLaunching();
    setError(error);
    setStatus("sena failed to start.");
  }
}

function exitPlayer() {
  window.location.reload();
}

input.addEventListener("change", async () => {
  selectedFileList = input.files;
  await scanCurrentSelection();
});

rescanButton.addEventListener("click", async () => {
  await scanCurrentSelection();
});

exitButton.addEventListener("click", exitPlayer);

window.addEventListener("resize", () => {
  if (playerScreen.style.display === "block") {
    canvas.focus();
  }
});

renderLibrary();
