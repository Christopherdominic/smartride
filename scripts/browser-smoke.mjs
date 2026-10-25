// Node 22+ / Google Chrome. Drives the actual frontend against the persisted API.
import { spawn } from "node:child_process";
import { mkdtemp, writeFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
const origin = process.env.FRONTEND_URL || "http://localhost:3000";
const profile = await mkdtemp(join(tmpdir(), "smartride-chrome-"));
const chrome = spawn(
  process.env.CHROME_PATH || "/usr/bin/google-chrome",
  [
    "--headless",
    "--disable-gpu",
    "--no-sandbox",
    "--remote-debugging-port=9224",
    `--user-data-dir=${profile}`,
    "about:blank",
  ],
  { stdio: "ignore" },
);
const pause = (ms) => new Promise((r) => setTimeout(r, ms));
let ws;
try {
  let tab;
  for (let i = 0; i < 100; i++) {
    try {
      const tabs = await (await fetch("http://localhost:9224/json")).json();
      tab = tabs.find((t) => t.type === "page");
      if (tab) break;
    } catch {}
    await pause(100);
  }
  if (!tab) throw Error("Chrome CDP did not start");
  ws = new WebSocket(tab.webSocketDebuggerUrl);
  await new Promise((r, j) => {
    ws.onopen = r;
    ws.onerror = j;
  });
  let id = 0;
  const pending = new Map();
  ws.onmessage = (e) => {
    const m = JSON.parse(e.data);
    if (pending.has(m.id)) {
      const { resolve, reject } = pending.get(m.id);
      pending.delete(m.id);
      m.error ? reject(Error(JSON.stringify(m.error))) : resolve(m.result);
    }
  };
  function send(method, params = {}) {
    return new Promise((resolve, reject) => {
      const n = ++id;
      pending.set(n, { resolve, reject });
      ws.send(JSON.stringify({ id: n, method, params }));
    });
  }
  async function evaluate(expression) {
    const r = await send("Runtime.evaluate", {
      expression,
      awaitPromise: true,
      returnByValue: true,
    });
    if (r.exceptionDetails) throw Error(JSON.stringify(r.exceptionDetails));
    return r.result.value;
  }
  async function wait(expression) {
    for (let i = 0; i < 150; i++) {
      if (await evaluate(expression)) return;
      await pause(150);
    }
    throw Error(
      "Timed out: " +
        expression +
        "\n" +
        (await evaluate("document.body.innerText")),
    );
  }
  async function navigate(path) {
    await send("Page.navigate", { url: origin + path });
    await wait('document.readyState === "complete"');
  }
  async function click(text) {
    await wait(
      `(()=>{const b=[...document.querySelectorAll('button')].find(b=>b.textContent.includes(${JSON.stringify(text)}));return b && !b.disabled})()`,
    );
    await evaluate(
      `(()=>{const b=[...document.querySelectorAll('button')].find(b=>b.textContent.includes(${JSON.stringify(text)}));if(!b||b.disabled)throw Error('Button unavailable: '+${JSON.stringify(text)});b.click()})()`,
    );
  }
  async function screenshot(name, width, height) {
    await send("Emulation.setDeviceMetricsOverride", {
      width,
      height,
      deviceScaleFactor: 1,
      mobile: width < 600,
    });
    await pause(300);
    const shot = await send("Page.captureScreenshot", {
      format: "png",
      captureBeyondViewport: true,
    });
    await writeFile(join(tmpdir(), name), Buffer.from(shot.data, "base64"));
    const overflow = await evaluate(
      "document.documentElement.scrollWidth > innerWidth",
    );
    if (overflow) throw Error("Horizontal overflow at " + width);
  }
  await send("Page.enable");
  await send("Runtime.enable");
  await navigate("/");
  await wait('document.body.innerText.includes("Every delivery.")');
  await screenshot("smartride-desktop.png", 1440, 1000);
  await screenshot("smartride-mobile.png", 390, 844);
  await navigate("/demo");
  await wait('document.querySelector("form select") !== null');
  const live = await evaluate(
    `[...document.querySelectorAll('button')].some(b=>b.textContent.includes('Verify postcode'))`,
  );
  if (live) {
    if (!process.env.TEST_POSTCODE)
      throw Error(
        "Set TEST_POSTCODE to a verified code appropriate to the configured NIPOST key environment",
      );
    await evaluate(
      `(()=>{const i=document.querySelector('input[placeholder="AA-99-H77-BB-55"]');Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(i,${JSON.stringify(process.env.TEST_POSTCODE)});i.dispatchEvent(new Event('input',{bubbles:true}))})()`,
    );
  }
  await click(live ? "Verify postcode" : "Resolve postcode");
  if (process.env.EXPECT_INVALID_POSTCODE === "1") {
    if (!live)
      throw Error("Negative NIPOST check requires the NIPOST provider");
    await wait(
      `document.querySelector('[role="alert"]')?.textContent.includes('not found')`,
    );
    if (
      await evaluate(
        `document.body.innerText.includes('Postcode verified by NIPOST') || document.querySelector('.address-preview') !== null`,
      )
    )
      throw Error("Invalid code presented as verified");
    if (
      !(await evaluate(
        `document.querySelector('textarea[name="customer_address"]')?.required`,
      ))
    )
      throw Error("Customer-provided address must be required independently");
    await evaluate(
      `(()=>{const t=document.querySelector('textarea[name="customer_address"]');Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype,'value').set.call(t,'12 Sample Street, Demo City');t.dispatchEvent(new Event('input',{bubbles:true}))})()`,
    );
    await pause(100);
    if (
      !(await evaluate(
        `[...document.querySelectorAll('button')].find(b=>b.textContent.includes('Create delivery'))?.disabled`,
      ))
    )
      throw Error("Creation enabled without valid L1 verification");
    await screenshot("smartride-invalid-postcode-mobile.png", 390, 844);
    console.log(
      "PASS: public NIPOST invalid result displayed; no verified badge or fictional address; customer address required; delivery creation blocked",
    );
  } else {
    await wait('document.querySelector(".address-preview") !== null');
    await evaluate(
      `(()=>{const t=document.querySelector('textarea[name="customer_address"]');Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype,'value').set.call(t,'12 Sample Street, Demo District, Test City');t.dispatchEvent(new Event('input',{bubbles:true}))})()`,
    );
    await pause(100);
    await click("Create delivery");
    await wait(
      'location.pathname.includes("/dashboard/deliveries/") && document.querySelector(".delivery-actions select") !== null',
    );
    await wait(
      'document.querySelector(".delivery-actions select")?.options.length > 1',
    );
    await evaluate(
      `(()=>{const s=document.querySelector('.delivery-actions select');s.value=s.options[1].value;s.dispatchEvent(new Event('change',{bubbles:true}))})()`,
    );
    await click("Assign rider");
    await wait('document.body.innerText.includes("Mark parcel picked up")');
    await click("Mark parcel picked up");
    await wait('document.body.innerText.includes("Start delivery")');
    await click("Start delivery");
    await wait('document.body.innerText.includes("Generate demo OTP")');
    await click("Generate demo OTP");
    await wait('document.querySelector(".otp-display strong") !== null');
    await evaluate(
      `(()=>{const code=document.querySelector('.otp-display strong').textContent;const input=document.querySelector('input[autocomplete="one-time-code"]');Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(input,code);input.dispatchEvent(new Event('input',{bubbles:true}));input.dispatchEvent(new Event('change',{bubbles:true}))})()`,
    );
    await pause(200);
    await click("Confirm delivery");
    await wait('document.body.innerText.includes("Confirmed with OTP")');
    await screenshot("smartride-delivered-mobile.png", 390, 844);
    const timeline = await evaluate(
      '[...document.querySelectorAll(".timeline strong")].map(e=>e.textContent)',
    );
    if (
      timeline.join(",") !==
      "created,assigned,picked up,out for delivery,delivered"
    )
      throw Error("Unexpected timeline " + timeline);
    await navigate("/dashboard");
    await wait('document.querySelector("tbody tr") !== null');
    await screenshot("smartride-dashboard-mobile.png", 390, 844);
    await navigate("/rider");
    await wait(
      'document.querySelector(".rider-select select")?.options.length > 1',
    );
    console.log(
      "PASS: actual browser workflow, OTP proof and timeline; landing/dashboard/detail responsive at 390px; screenshots in /tmp/smartride-*.png",
    );
  }
} finally {
  ws?.close();
  chrome.kill();
  await pause(300);
  await rm(profile, { recursive: true, force: true }).catch(() => {});
}
