// D-164 (L3 smoke, docs/ci.md L3 Smoke): start the app through tauri-driver and check that the home
// heading is rendered. Node 24 standard library only (D-030): five WebDriver requests over fetch.
// ponytail: one heading only; screen operations (E-2) would re-compare a WebDriver client dependency.
import { spawn } from "node:child_process";
import { existsSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { setTimeout as sleep } from "node:timers/promises";

const EXPECTED_HEADING = "ホーム";
const DRIVER_URL = "http://127.0.0.1:4444";
const ELEMENT_KEY = "element-6066-11e4-a52e-4f735466cecf"; // W3C WebDriver §12.1 web element identifier
// One overall deadline; cleanup fits inside it (270 + 15 + 10, 5 seconds spare).
const TOTAL_MS = 300_000;
const WORK_MS = 270_000;
const DELETE_MS = 15_000;
const EXIT_WAIT_MS = 10_000;
const RETRY_MS = 500;
// The step reads this file for the verdict; the exit code is not used (Node can end with 127 after a PASS).
const RESULT_FILE = process.env.L3_SMOKE_RESULT;

const startedAt = Date.now();
const work = AbortSignal.timeout(WORK_MS);
let child;
let exited;
let driverExit;
// Only the exit event proves tauri-driver ended; an error event (spawn or kill failure) does not.
let driverExited = false;
let stage = "setup";

// A grandchild that holds a pipe or a hung cleanup must not keep the step running.
setTimeout(() => {
  console.error(`L3 smoke: FAIL: overall ${TOTAL_MS / 1000} s deadline passed (stage: ${stage})`);
  child?.kill();
  process.exit(1);
}, TOTAL_MS);

function requireFile(name) {
  const value = process.env[name];
  if (!value) throw new Error(`${name} is not set`);
  const path = resolve(value);
  if (!existsSync(path)) throw new Error(`${name} does not exist: ${path}`);
  return path;
}

async function webdriver(method, path, body, signal) {
  const response = await fetch(DRIVER_URL + path, {
    method,
    headers: body === undefined ? undefined : { "content-type": "application/json" },
    body: body === undefined ? undefined : JSON.stringify(body),
    signal,
  });
  const text = await response.text();
  if (!response.ok) {
    throw new Error(`${method} ${path}: HTTP ${response.status} ${text.slice(0, 500)}`);
  }
  return JSON.parse(text).value;
}

async function createSession(application, signal) {
  for (;;) {
    if (driverExit) throw new Error(`tauri-driver ended before the session: ${driverExit}`);
    try {
      return await webdriver(
        "POST",
        "/session",
        { capabilities: { alwaysMatch: { browserName: "wry", "tauri:options": { application } } } },
        signal,
      );
    } catch (error) {
      // Only "not listening yet" is retried; every other failure is the result.
      if (error?.cause?.code !== "ECONNREFUSED") throw error;
    }
    await sleep(RETRY_MS, undefined, { signal });
  }
}

let sessionId;
let code = 1;
let cleanedUp = false;
try {
  const application = requireFile("L3_SMOKE_APP");
  const nativeDriver = requireFile("L3_SMOKE_NATIVE_DRIVER");
  child = spawn("tauri-driver", ["--native-driver", nativeDriver], {
    stdio: ["ignore", "pipe", "pipe"],
  });
  child.stdout.on("data", (chunk) => process.stdout.write(chunk));
  child.stderr.on("data", (chunk) => process.stderr.write(chunk));
  child.on("error", (error) => {
    driverExit = `spawn error ${error.message}`;
  });
  exited = new Promise((resolveExit) => {
    child.on("exit", (exitCode, signal) => {
      driverExit = `exit code ${exitCode}, signal ${signal}`;
      driverExited = true;
      resolveExit();
    });
  });

  stage = "session creation";
  sessionId = (await createSession(application, work)).sessionId;
  stage = "implicit wait";
  const implicit = Math.max(0, WORK_MS - (Date.now() - startedAt));
  await webdriver("POST", `/session/${sessionId}/timeouts`, { implicit }, work);
  stage = "h1 search";
  const element = await webdriver(
    "POST",
    `/session/${sessionId}/element`,
    { using: "css selector", value: "h1" },
    work,
  );
  stage = "h1 text";
  const text = await webdriver(
    "GET",
    `/session/${sessionId}/element/${element[ELEMENT_KEY]}/text`,
    undefined,
    work,
  );
  console.log(`L3 smoke: heading text: ${text}`);
  if (text === EXPECTED_HEADING) {
    console.log(`L3 smoke: heading matched (${Math.round((Date.now() - startedAt) / 1000)} s)`);
    code = 0;
  } else {
    console.error(
      `L3 smoke: FAIL: heading text ${JSON.stringify(text)} is not ${JSON.stringify(EXPECTED_HEADING)}`,
    );
  }
} catch (error) {
  if (work.aborted) {
    console.error(`L3 smoke: FAIL: ${stage} timed out at the ${WORK_MS / 1000} s work deadline`);
  } else {
    const cause = error?.cause?.code ? ` (${error.cause.code})` : "";
    console.error(`L3 smoke: FAIL: ${stage}: ${error?.message ?? error}${cause}`);
  }
} finally {
  stage = "cleanup";
  let sessionDeleted = sessionId === undefined;
  if (!sessionDeleted) {
    try {
      await webdriver("DELETE", `/session/${sessionId}`, undefined, AbortSignal.timeout(DELETE_MS));
      sessionDeleted = true;
    } catch (error) {
      console.error(`L3 smoke: DELETE /session failed: ${error?.message ?? error}`);
    }
  }
  if (child && !driverExit) {
    // tauri-driver 2.1.0 puts itself and the native driver in a kill-on-close Job Object on Windows.
    child.kill();
    await Promise.race([exited, sleep(EXIT_WAIT_MS)]);
    if (!driverExited)
      console.error(`L3 smoke: tauri-driver did not exit within ${EXIT_WAIT_MS / 1000} s`);
  }
  cleanedUp = sessionDeleted && driverExited;
  if (code === 0 && !cleanedUp) console.error("L3 smoke: FAIL: cleanup did not finish");
}
if (code === 0 && cleanedUp) writeFileSync(RESULT_FILE, "PASS");
process.exit(code);
