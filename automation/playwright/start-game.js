#!/usr/bin/env node
const { chromium } = require('playwright');

const BASE_URL = process.env.BASE_URL || 'http://127.0.0.1:4001/index.htm';
const TIMEOUT_MS = Number(process.env.TIMEOUT_MS || 30000);
const HEADLESS = process.env.HEADLESS !== 'false';

async function waitForCanvas(page, label) {
  const canvas = page.locator('#glcanvas');
  await canvas.waitFor({ state: 'visible', timeout: TIMEOUT_MS });
  console.log(`${label}: canvas is visible`);
}

async function attachLogging(page, label) {
  page.on('console', (msg) => {
    const text = msg.text();
    console.log(`[${label} console:${msg.type()}] ${text}`);
  });

  page.on('pageerror', (error) => {
    console.error(`[${label} pageerror] ${error.message}`);
  });

  page.on('requestfailed', (request) => {
    console.error(
      `[${label} requestfailed] ${request.method()} ${request.url()} -> ${request.failure()?.errorText}`
    );
  });
}

async function saveDebugArtifacts(page, label) {
  const safeLabel = label.toLowerCase().replace(/[^a-z0-9]+/g, '-');
  await page.screenshot({ path: `${safeLabel}.png`, fullPage: true }).catch(() => {});
  await page.content().then((html) => require('fs').writeFileSync(`${safeLabel}.html`, html)).catch(() => {});
}

async function main() {
  const browser1 = await chromium.launch({ headless: HEADLESS });
  const browser2 = await chromium.launch({ headless: HEADLESS });

  const context1 = await browser1.newContext();
  const context2 = await browser2.newContext();
  const page1 = await context1.newPage();
  const page2 = await context2.newPage();

  await attachLogging(page1, 'host');
  await attachLogging(page2, 'guest');

  try {
    console.log(`Opening host page: ${BASE_URL}`);
    await page1.goto(BASE_URL, { waitUntil: 'domcontentloaded', timeout: TIMEOUT_MS });

    await page1.getByText('Play with a friend', { exact: true }).click({ timeout: TIMEOUT_MS });
    const inviteText = (await page1.locator('#room_id_display').textContent())?.trim();

    if (!inviteText) {
      throw new Error('Invite URL was not rendered after clicking Play with a friend');
    }

    let inviteUrl;
    try {
      inviteUrl = new URL(inviteText).toString();
    } catch (error) {
      throw new Error(`Invite URL is invalid: ${inviteText}`);
    }

    console.log(`Invite URL: ${inviteUrl}`);
    console.log(`Opening guest page: ${inviteUrl}`);
    await page2.goto(inviteUrl, { waitUntil: 'domcontentloaded', timeout: TIMEOUT_MS });

    await waitForCanvas(page2, 'guest');

    await page1.getByText('Copy Invite & Start Room', { exact: true }).click({ timeout: TIMEOUT_MS });
    await waitForCanvas(page1, 'host');

    await page1.waitForTimeout(5000);
    await page2.waitForTimeout(5000);

    console.log('Bugchess game launch flow completed in both headless browsers.');
  } catch (error) {
    console.error(`Automation failed: ${error.message}`);
    await Promise.all([
      saveDebugArtifacts(page1, 'host-debug'),
      saveDebugArtifacts(page2, 'guest-debug'),
    ]);
    process.exitCode = 1;
  } finally {
    await Promise.allSettled([
      context1.close(),
      context2.close(),
      browser1.close(),
      browser2.close(),
    ]);
  }
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
